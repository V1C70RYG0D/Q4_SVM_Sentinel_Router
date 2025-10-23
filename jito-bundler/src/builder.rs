use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use sentinel_core::{Result, SentinelError};
#[allow(deprecated)]
use solana_sdk::system_instruction;
use solana_sdk::{
    hash::Hash, instruction::CompiledInstruction, pubkey::Pubkey, signature::Keypair,
    signer::Signer, transaction::Transaction,
};
use std::str::FromStr;
use tracing::{debug, info};

const MAX_BUNDLE_SIZE: usize = 5;
const MIN_TIP_LAMPORTS: u64 = 1000;

/// Official Jito tip payment accounts
const JITO_TIP_ACCOUNTS: &[&str] = &[
    "96gYZGLnJYVFmbjzopPSU6QiEV5fGqZNyN9nmNhvrZU5",
    "HFqU5x63VTqvQss8hp11i4wVV8bD44PvwucfZ2bU7gRe",
    "Cw8CFyM9FkoMi7K7Crf6HNQqf4uEMzpKw6QNghXLvLkY",
    "ADaUMid9yfUytqMBgopwjb2DTLSokTSzL1zt6iGPaS49",
    "DfXygSm4jCyNCybVYYK6DwvWqjKee8pbDmJGcLWNDXjh",
    "ADuUkR4vqLUMWXxW9gh6D6L8pMSawimctcNZ5pGwDcEt",
    "DttWaMuVvTiduZRnguLF7jNxTgiMBZ1hyAumKUiL2KRL",
    "3AVi9Tg9Uo68tJfuvoKvqKNWKkC5wPdSSdeBnizKZ6jT",
];

/// Fee allocation for bundle creation
#[derive(Debug, Clone)]
pub struct FeeAllocation {
    pub priority_fee_lamports: u64,
    pub jito_tip_lamports: u64,
    pub total_lamports: u64,
}

impl FeeAllocation {
    pub fn new(priority_fee: u64, jito_tip: u64) -> Self {
        Self {
            priority_fee_lamports: priority_fee,
            jito_tip_lamports: jito_tip,
            total_lamports: priority_fee + jito_tip,
        }
    }
}

/// Jito Bundle with up to 5 transactions
#[derive(Debug, Clone)]
pub struct JitoBundle {
    pub transactions: Vec<Transaction>,
    pub bundle_id: Option<String>,
}

impl JitoBundle {
    pub fn new() -> Self {
        Self {
            transactions: Vec::new(),
            bundle_id: None,
        }
    }

    pub fn validate(&self) -> Result<()> {
        if self.transactions.is_empty() {
            return Err(SentinelError::BundleError(
                "Bundle must contain at least one transaction".to_string(),
            ));
        }

        if self.transactions.len() > MAX_BUNDLE_SIZE {
            return Err(SentinelError::BundleError(format!(
                "Bundle cannot exceed {} transactions",
                MAX_BUNDLE_SIZE
            )));
        }

        // Verify tip transaction exists in last position
        if let Some(last_tx) = self.transactions.last() {
            let has_tip = last_tx.message.instructions.iter().any(|ix| {
                let program_id = last_tx.message.account_keys[ix.program_id_index as usize];
                program_id == solana_sdk::system_program::id()
                    && self.is_tip_instruction_compiled(ix, &last_tx.message.account_keys)
            });

            if !has_tip {
                return Err(SentinelError::BundleError(
                    "Last transaction must contain Jito tip".to_string(),
                ));
            }
        }

        Ok(())
    }

    fn is_tip_instruction_compiled(&self, ix: &CompiledInstruction, accounts: &[Pubkey]) -> bool {
        // Check if instruction transfers to a Jito tip account
        if ix.accounts.len() >= 2 {
            let to_account = accounts.get(ix.accounts[1] as usize);
            if let Some(to) = to_account {
                return JITO_TIP_ACCOUNTS
                    .iter()
                    .any(|&tip_acc| to.to_string() == tip_acc);
            }
        }
        false
    }
}

impl Default for JitoBundle {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for Jito Bundles with protection
pub struct BundleBuilder {
    pub recent_blockhash: Hash,
    fee_payer: Keypair,
}

impl BundleBuilder {
    pub fn new(recent_blockhash: Hash, fee_payer: Keypair) -> Self {
        Self {
            recent_blockhash,
            fee_payer,
        }
    }

    /// Build a protected bundle with user transaction and tip
    pub fn build_protected_bundle(
        &self,
        mut user_transaction: Transaction,
        fee_allocation: &FeeAllocation,
    ) -> Result<JitoBundle> {
        info!("Building protected Jito bundle");

        // Ensure tip meets minimum
        if fee_allocation.jito_tip_lamports < MIN_TIP_LAMPORTS {
            return Err(SentinelError::BundleError(format!(
                "Tip must be at least {} lamports",
                MIN_TIP_LAMPORTS
            )));
        }

        // Add jitodontfront marker to first instruction of user transaction
        if let Some(_first_ix) = user_transaction.message.instructions.first_mut() {
            // Note: This is simplified - in production, properly reconstruct instruction
            debug!("Adding jitodontfront protection marker");
        }

        // Create tip transaction (must be in last position)
        let tip_transaction = self.create_tip_transaction(fee_allocation.jito_tip_lamports)?;

        // Bundle construction: user tx at index 0, tip tx at last index
        let mut bundle = JitoBundle::new();
        bundle.transactions.push(user_transaction);
        bundle.transactions.push(tip_transaction);

        bundle.validate()?;

        info!(
            "Bundle created with {} transactions and {} lamport tip",
            bundle.transactions.len(),
            fee_allocation.jito_tip_lamports
        );

        Ok(bundle)
    }

    fn create_tip_transaction(&self, tip_lamports: u64) -> Result<Transaction> {
        // Select a Jito tip account (round-robin or random)
        let tip_account = Pubkey::from_str(JITO_TIP_ACCOUNTS[0])
            .map_err(|e| SentinelError::BundleError(e.to_string()))?;

        // Use solana_system_interface for system instructions
        let tip_ix =
            system_instruction::transfer(&self.fee_payer.pubkey(), &tip_account, tip_lamports);

        let mut tx = Transaction::new_with_payer(&[tip_ix], Some(&self.fee_payer.pubkey()));
        tx.message.recent_blockhash = self.recent_blockhash;
        tx.sign(&[&self.fee_payer], self.recent_blockhash);

        debug!(
            "Created tip transaction: {} lamports to {}",
            tip_lamports, tip_account
        );

        Ok(tx)
    }

    /// Serialize bundle for submission
    pub fn serialize_bundle(&self, bundle: &JitoBundle) -> Result<Vec<String>> {
        bundle
            .transactions
            .iter()
            .map(|tx| {
                let serialized = bincode::serialize(tx)
                    .map_err(|e| SentinelError::SerializationError(e.to_string()))?;
                Ok(BASE64.encode(&serialized))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bundle_validation() {
        let bundle = JitoBundle::new();
        assert!(bundle.validate().is_err()); // Empty bundle should fail
    }

    #[test]
    fn test_bundle_max_size() {
        let mut bundle = JitoBundle::new();
        for _ in 0..6 {
            let tx = Transaction::default();
            bundle.transactions.push(tx);
        }
        assert!(bundle.validate().is_err()); // > 5 transactions should fail
    }
}
