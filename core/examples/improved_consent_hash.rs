// RECOMMENDED FIX for Intent consent hash
// This file demonstrates the improved implementation using SHA256
// To apply: Replace the compute_consent_hash method in crates/core/src/intent.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use solana_sdk::pubkey::Pubkey;

/// Improved Intent implementation with cryptographic consent hash
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImprovedIntent {
    pub intent_id: String,
    pub user_public_key: Pubkey,
    pub intent_type: IntentType,
    pub swap_details: Option<SwapDetails>,
    pub constraints: Constraints,
    pub fee_preferences: FeePreferences,
    pub consent: ConsentBlock,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum IntentType {
    Swap,
    Transfer,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapDetails {
    pub input_mint: Pubkey,
    pub output_mint: Pubkey,
    pub amount: u64,
    pub mode: SwapMode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SwapMode {
    ExactIn,
    ExactOut,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constraints {
    pub slippage_bps: u16,
    pub deadline: DateTime<Utc>,
    pub allow_partial_fills: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeePreferences {
    pub max_priority_fee_lamports: u64,
    pub max_jito_tip_lamports: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsentBlock {
    pub consent_to_protective_routing: bool,
    pub router_fee_bps: u16,
    pub consent_hash: String,
}

impl ImprovedIntent {
    /// Compute cryptographically secure anti-tamper consent hash
    /// Uses SHA256 over all non-consent fields
    pub fn compute_consent_hash(&self) -> String {
        let mut hasher = Sha256::new();

        // Hash intent_id
        hasher.update(self.intent_id.as_bytes());

        // Hash user public key
        hasher.update(self.user_public_key.to_bytes());

        // Hash intent type
        let intent_type_str = match &self.intent_type {
            IntentType::Swap => "SWAP",
            IntentType::Transfer => "TRANSFER",
            IntentType::Custom(name) => name.as_str(),
        };
        hasher.update(intent_type_str.as_bytes());

        // Hash swap details if present
        if let Some(swap) = &self.swap_details {
            hasher.update(swap.input_mint.to_bytes());
            hasher.update(swap.output_mint.to_bytes());
            hasher.update(swap.amount.to_le_bytes());

            let mode_str = match swap.mode {
                SwapMode::ExactIn => "EXACT_IN",
                SwapMode::ExactOut => "EXACT_OUT",
            };
            hasher.update(mode_str.as_bytes());
        } else {
            // Hash a marker for None to prevent collision
            hasher.update(b"NO_SWAP");
        }

        // Hash constraints
        hasher.update(self.constraints.slippage_bps.to_le_bytes());
        hasher.update(self.constraints.deadline.timestamp().to_le_bytes());
        hasher.update([self.constraints.allow_partial_fills as u8]);

        // Hash fee preferences
        hasher.update(self.fee_preferences.max_priority_fee_lamports.to_le_bytes());
        hasher.update(self.fee_preferences.max_jito_tip_lamports.to_le_bytes());

        // Hash timestamp
        hasher.update(self.timestamp.timestamp().to_le_bytes());

        // DO NOT hash consent block (this is what we're protecting)

        // Return hex-encoded hash
        hex::encode(hasher.finalize())
    }

    /// Validate the intent with improved hash checking
    pub fn validate(&self) -> Result<(), String> {
        // Check deadline is in the future
        if self.constraints.deadline <= Utc::now() {
            return Err("Deadline must be in the future".to_string());
        }

        // Check slippage is reasonable
        if self.constraints.slippage_bps > 10000 {
            return Err("Slippage must be <= 10000 bps (100%)".to_string());
        }

        // Verify consent
        if !self.consent.consent_to_protective_routing {
            return Err("User must consent to protective routing".to_string());
        }

        // Verify consent hash (CRITICAL SECURITY CHECK)
        let computed_hash = self.compute_consent_hash();
        if computed_hash != self.consent.consent_hash {
            return Err(format!(
                "Consent hash mismatch - possible tampering detected. Expected: {}, Got: {}",
                computed_hash, self.consent.consent_hash
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_intent() -> ImprovedIntent {
        ImprovedIntent {
            intent_id: "test-123".to_string(),
            user_public_key: Pubkey::new_unique(),
            intent_type: IntentType::Swap,
            swap_details: Some(SwapDetails {
                input_mint: Pubkey::new_unique(),
                output_mint: Pubkey::new_unique(),
                amount: 1_000_000,
                mode: SwapMode::ExactIn,
            }),
            constraints: Constraints {
                slippage_bps: 50,
                deadline: Utc::now() + chrono::Duration::hours(1),
                allow_partial_fills: false,
            },
            fee_preferences: FeePreferences {
                max_priority_fee_lamports: 100_000,
                max_jito_tip_lamports: 50_000,
            },
            consent: ConsentBlock {
                consent_to_protective_routing: true,
                router_fee_bps: 10,
                consent_hash: String::new(),
            },
            timestamp: Utc::now(),
        }
    }

    #[test]
    fn test_sha256_hash_consistency() {
        let intent = create_test_intent();
        let hash1 = intent.compute_consent_hash();
        let hash2 = intent.compute_consent_hash();

        assert_eq!(hash1, hash2, "Hash should be consistent");
        assert_eq!(hash1.len(), 64, "SHA256 hex should be 64 characters");

        println!("✅ SHA256 hash: {}", hash1);
    }

    #[test]
    fn test_hash_detects_intent_id_tampering() {
        let mut intent = create_test_intent();
        let original_hash = intent.compute_consent_hash();
        intent.consent.consent_hash = original_hash.clone();

        // Tamper with intent_id
        intent.intent_id = "tampered-id".to_string();

        let new_hash = intent.compute_consent_hash();
        assert_ne!(original_hash, new_hash, "Hash should change with intent_id");
        assert!(intent.validate().is_err(), "Validation should fail");
    }

    #[test]
    fn test_hash_detects_swap_amount_tampering() {
        let mut intent = create_test_intent();
        let original_hash = intent.compute_consent_hash();
        intent.consent.consent_hash = original_hash.clone();

        // Tamper with swap amount
        if let Some(swap) = &mut intent.swap_details {
            swap.amount = 999_999_999;
        }

        let new_hash = intent.compute_consent_hash();
        assert_ne!(original_hash, new_hash, "Hash should change with amount");
        assert!(intent.validate().is_err(), "Validation should fail");
    }

    #[test]
    fn test_hash_detects_slippage_tampering() {
        let mut intent = create_test_intent();
        let original_hash = intent.compute_consent_hash();
        intent.consent.consent_hash = original_hash.clone();

        // Tamper with slippage
        intent.constraints.slippage_bps = 5000; // Increase to 50%

        let new_hash = intent.compute_consent_hash();
        assert_ne!(original_hash, new_hash, "Hash should change with slippage");
        assert!(intent.validate().is_err(), "Validation should fail");
    }

    #[test]
    fn test_hash_detects_fee_tampering() {
        let mut intent = create_test_intent();
        let original_hash = intent.compute_consent_hash();
        intent.consent.consent_hash = original_hash.clone();

        // Tamper with fees
        intent.fee_preferences.max_priority_fee_lamports = 9_999_999;

        let new_hash = intent.compute_consent_hash();
        assert_ne!(original_hash, new_hash, "Hash should change with fees");
        assert!(intent.validate().is_err(), "Validation should fail");
    }

    #[test]
    fn test_hash_ignores_consent_block() {
        let mut intent = create_test_intent();
        let hash1 = intent.compute_consent_hash();

        // Change consent fields (should NOT affect hash)
        intent.consent.router_fee_bps = 20;
        let hash2 = intent.compute_consent_hash();

        assert_eq!(hash1, hash2, "Hash should ignore consent block fields");
    }

    #[test]
    fn test_valid_intent_passes() {
        let mut intent = create_test_intent();
        let correct_hash = intent.compute_consent_hash();
        intent.consent.consent_hash = correct_hash;

        assert!(intent.validate().is_ok(), "Valid intent should pass");
    }

    #[test]
    fn test_hash_different_for_different_intents() {
        let intent1 = create_test_intent();
        let intent2 = create_test_intent();

        let hash1 = intent1.compute_consent_hash();
        let hash2 = intent2.compute_consent_hash();

        // Should differ because of random pubkeys and timestamps
        assert_ne!(
            hash1, hash2,
            "Different intents should have different hashes"
        );
    }
}

/* ========================================
   MIGRATION GUIDE
   ========================================

To apply this fix to the existing codebase:

1. Add sha2 and hex dependencies to crates/core/Cargo.toml:
   ```toml
   [dependencies]
   sha2 = "0.10"
   hex = "0.4"
   ```

2. Replace the compute_consent_hash method in crates/core/src/intent.rs:

   CURRENT (WEAK):
   ```rust
   fn compute_consent_hash(&self) -> String {
       use std::collections::hash_map::DefaultHasher;
       use std::hash::{Hash, Hasher};

       let mut hasher = DefaultHasher::new();
       self.intent_id.hash(&mut hasher);
       self.user_public_key.to_string().hash(&mut hasher);
       format!("{:x}", hasher.finish())
   }
   ```

   REPLACE WITH (STRONG):
   ```rust
   fn compute_consent_hash(&self) -> String {
       use sha2::{Sha256, Digest};

       let mut hasher = Sha256::new();

       // Hash intent_id
       hasher.update(self.intent_id.as_bytes());

       // Hash user public key
       hasher.update(self.user_public_key.to_bytes());

       // Hash intent type
       let intent_type_str = match &self.intent_type {
           IntentType::Swap => "SWAP",
           IntentType::Transfer => "TRANSFER",
           IntentType::Custom(name) => name.as_str(),
       };
       hasher.update(intent_type_str.as_bytes());

       // Hash swap details if present
       if let Some(swap) = &self.swap_details {
           hasher.update(swap.input_mint.to_bytes());
           hasher.update(swap.output_mint.to_bytes());
           hasher.update(&swap.amount.to_le_bytes());

           let mode_str = match swap.mode {
               SwapMode::ExactIn => "EXACT_IN",
               SwapMode::ExactOut => "EXACT_OUT",
           };
           hasher.update(mode_str.as_bytes());
       } else {
           hasher.update(b"NO_SWAP");
       }

       // Hash constraints
       hasher.update(&self.constraints.slippage_bps.to_le_bytes());
       hasher.update(&self.constraints.deadline.timestamp().to_le_bytes());
       hasher.update(&[self.constraints.allow_partial_fills as u8]);

       // Hash fee preferences
       hasher.update(&self.fee_preferences.max_priority_fee_lamports.to_le_bytes());
       hasher.update(&self.fee_preferences.max_jito_tip_lamports.to_le_bytes());

       // Hash timestamp
       hasher.update(&self.timestamp.timestamp().to_le_bytes());

       // Return hex-encoded hash
       hex::encode(hasher.finalize())
   }
   ```

3. Run tests:
   ```bash
   cargo test --package sentinel-core
   ```

4. Update frontend to compute matching hash before submission:
   ```typescript
   import { sha256 } from 'js-sha256';

   function computeConsentHash(intent: Intent): string {
       const hashInput = [
           intent.intentId,
           intent.userPublicKey,
           intent.intentType,
           // ... include all non-consent fields
       ].join('|');

       return sha256(hashInput);
   }
   ```

5. Test with actual intent submission to verify hash matching

========================================
*/

fn main() {
    // This is a reference implementation file
    // Run tests with: cargo test --example improved_consent_hash
    println!("This file demonstrates the improved consent hash implementation.");
    println!("The actual implementation is in crates/core/src/intent.rs");
}
