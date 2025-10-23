// Transaction feature extraction module
use crate::features_enhanced::FeatureVector;
use sentinel_core::Result;
use solana_sdk::instruction::CompiledInstruction;
use solana_sdk::transaction::Transaction;

/// Extract features from a signed Solana transaction
pub fn extract_from_transaction(transaction: &Transaction) -> Result<FeatureVector> {
    let mut features = FeatureVector::default();

    // Extract compute budget instructions
    for instruction in &transaction.message.instructions {
        if let Some((compute_units, price)) = parse_compute_budget(instruction) {
            if compute_units > 0 {
                features.compute_unit_limit = compute_units;
            }
            if price > 0 {
                features.compute_unit_price = price;
            }
        }
    }

    // Check for DEX swap patterns
    features.is_dex_swap = is_dex_transaction(transaction);

    // Default safe values
    features.oracle_confidence = 0.95;
    features.tip_percentile_vs_recent = 50.0;

    Ok(features)
}

fn parse_compute_budget(instruction: &CompiledInstruction) -> Option<(u32, u64)> {
    // Compute Budget Program ID: ComputeBudget111111111111111111111111111111
    // Simplified parsing - in production, use proper deserialization
    if instruction.data.len() >= 5 {
        let discriminator = instruction.data[0];
        match discriminator {
            2 => {
                // SetComputeUnitLimit
                let units = u32::from_le_bytes([
                    instruction.data[1],
                    instruction.data[2],
                    instruction.data[3],
                    instruction.data[4],
                ]);
                Some((units, 0))
            }
            3 => {
                // SetComputeUnitPrice
                if instruction.data.len() >= 9 {
                    let price = u64::from_le_bytes([
                        instruction.data[1],
                        instruction.data[2],
                        instruction.data[3],
                        instruction.data[4],
                        instruction.data[5],
                        instruction.data[6],
                        instruction.data[7],
                        instruction.data[8],
                    ]);
                    Some((0, price))
                } else {
                    None
                }
            }
            _ => None,
        }
    } else {
        None
    }
}

fn is_dex_transaction(transaction: &Transaction) -> bool {
    // Check if transaction interacts with known DEX programs
    let known_dex_programs = [
        "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8", // Raydium
        "9W959DqEETiGZocYWCQPaJ6sBmUzgfxXfqGeTEdp3aQP", // Orca
        "JUP4Fb2cqiRUcaTHdrPC8h2gNsA2ETXiPDD33WcGuJB",  // Jupiter
    ];

    transaction
        .message
        .account_keys
        .iter()
        .any(|key| known_dex_programs.iter().any(|dex| key.to_string() == *dex))
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_sdk::message::Message;
    use solana_sdk::signature::Keypair;
    use solana_sdk::signer::Signer;
    #[allow(deprecated)]
    use solana_sdk::system_instruction;

    #[test]
    fn test_extract_from_simple_transaction() {
        let payer = Keypair::new();
        let to = Keypair::new();

        let instruction = system_instruction::transfer(&payer.pubkey(), &to.pubkey(), 1000);

        let message = Message::new(&[instruction], Some(&payer.pubkey()));
        let transaction = Transaction::new_unsigned(message);

        let features = extract_from_transaction(&transaction).unwrap();
        assert!(!features.is_dex_swap);
    }
}
