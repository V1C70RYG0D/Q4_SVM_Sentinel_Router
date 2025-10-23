use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

/// Feature vector for MEV threat detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureVector {
    // Base transaction metadata
    pub slot: u64,
    pub compute_unit_limit: u32,
    pub compute_unit_price: u64,
    pub jito_tip_lamports: u64,

    // DEX details
    pub is_dex_swap: bool,
    pub input_amount: f64,
    pub output_amount: f64,
    pub price_impact_bps: f64,

    // Market context (from Pyth)
    pub oracle_price: f64,
    pub oracle_confidence: f64,

    // Pattern indicators
    pub has_swap_triplet: bool,
    pub is_potential_sandwich_victim: bool,
    pub is_potential_front_run: bool,
    pub is_potential_back_run: bool,

    // Wide sandwich indicators
    pub recent_swaps_same_pair: u32,
    pub recent_swaps_same_actor: u32,

    // Jito tip context
    pub tip_percentile_vs_recent: f32,

    // Time-based features
    pub time_since_last_slot_ms: u64,
}

impl Default for FeatureVector {
    fn default() -> Self {
        Self {
            slot: 0,
            compute_unit_limit: 0,
            compute_unit_price: 0,
            jito_tip_lamports: 0,
            is_dex_swap: false,
            input_amount: 0.0,
            output_amount: 0.0,
            price_impact_bps: 0.0,
            oracle_price: 0.0,
            oracle_confidence: 0.0,
            has_swap_triplet: false,
            is_potential_sandwich_victim: false,
            is_potential_front_run: false,
            is_potential_back_run: false,
            recent_swaps_same_pair: 0,
            recent_swaps_same_actor: 0,
            tip_percentile_vs_recent: 0.0,
            time_since_last_slot_ms: 0,
        }
    }
}

impl FeatureVector {
    /// Convert to array for model inference
    pub fn to_array(&self) -> Vec<f32> {
        vec![
            self.slot as f32,
            self.compute_unit_limit as f32,
            self.compute_unit_price as f32,
            self.jito_tip_lamports as f32,
            if self.is_dex_swap { 1.0 } else { 0.0 },
            self.input_amount as f32,
            self.output_amount as f32,
            self.price_impact_bps as f32,
            self.oracle_price as f32,
            self.oracle_confidence as f32,
            if self.has_swap_triplet { 1.0 } else { 0.0 },
            if self.is_potential_sandwich_victim {
                1.0
            } else {
                0.0
            },
            if self.is_potential_front_run {
                1.0
            } else {
                0.0
            },
            if self.is_potential_back_run { 1.0 } else { 0.0 },
            self.recent_swaps_same_pair as f32,
            self.recent_swaps_same_actor as f32,
            self.tip_percentile_vs_recent,
            self.time_since_last_slot_ms as f32,
        ]
    }

    pub fn feature_count() -> usize {
        18
    }
}

/// Feature extractor with stateful tracking for pattern detection
pub struct FeatureExtractor {
    recent_swaps: Vec<SwapRecord>,
    max_history: usize,
}

#[derive(Debug, Clone)]
struct SwapRecord {
    slot: u64,
    actor: Pubkey,
    token_pair: (Pubkey, Pubkey),
    #[allow(dead_code)] // May be used for profitability checks in future
    amount: u64,
}

impl FeatureExtractor {
    pub fn new() -> Self {
        Self {
            recent_swaps: Vec::new(),
            max_history: 1000,
        }
    }

    /// Extract features from transaction data
    pub fn extract(&mut self, tx_data: &TransactionData) -> FeatureVector {
        let features = FeatureVector {
            slot: tx_data.slot,
            compute_unit_limit: tx_data.compute_unit_limit,
            compute_unit_price: tx_data.compute_unit_price,
            jito_tip_lamports: tx_data.jito_tip_lamports,
            has_swap_triplet: self.detect_swap_triplet(tx_data),
            recent_swaps_same_pair: self.count_recent_swaps_same_pair(tx_data),
            recent_swaps_same_actor: self.count_recent_swaps_same_actor(tx_data),
            ..Default::default()
        };

        // Update history
        self.update_history(tx_data);

        features
    }

    fn detect_swap_triplet(&self, tx_data: &TransactionData) -> bool {
        // Check for sandwich attack pattern:
        // 1. Find a recent swap from an actor (potential front-run)
        // 2. Current transaction could be victim
        // 3. Look for a matching back-run from same actor

        if let Some(ref victim_swap) = tx_data.swap_details {
            // Look for potential front-run (same token pair, different actor, recent slot)
            let potential_front_runs: Vec<&SwapRecord> = self
                .recent_swaps
                .iter()
                .filter(|s| {
                    s.slot <= tx_data.slot
                        && s.slot >= tx_data.slot.saturating_sub(2) // Within 2 slots
                        && s.token_pair == (victim_swap.input_mint, victim_swap.output_mint)
                        && s.actor != tx_data.fee_payer // Different actor
                })
                .collect();

            // For each potential front-run, look for matching back-run from same actor
            for front_run in potential_front_runs {
                let has_back_run = self.recent_swaps.iter().any(|s| {
                    s.actor == front_run.actor
                        && s.slot >= tx_data.slot
                        && s.slot <= tx_data.slot + 2
                        && s.token_pair == (victim_swap.output_mint, victim_swap.input_mint)
                });

                if has_back_run {
                    return true;
                }
            }
        }

        false
    }

    fn count_recent_swaps_same_pair(&self, tx_data: &TransactionData) -> u32 {
        if let Some(ref swap) = tx_data.swap_details {
            self.recent_swaps
                .iter()
                .filter(|s| {
                    s.token_pair == (swap.input_mint, swap.output_mint)
                        && s.slot >= tx_data.slot.saturating_sub(100)
                })
                .count() as u32
        } else {
            0
        }
    }

    fn count_recent_swaps_same_actor(&self, tx_data: &TransactionData) -> u32 {
        self.recent_swaps
            .iter()
            .filter(|s| s.actor == tx_data.fee_payer && s.slot >= tx_data.slot.saturating_sub(100))
            .count() as u32
    }

    fn update_history(&mut self, tx_data: &TransactionData) {
        if let Some(ref swap) = tx_data.swap_details {
            self.recent_swaps.push(SwapRecord {
                slot: tx_data.slot,
                actor: tx_data.fee_payer,
                token_pair: (swap.input_mint, swap.output_mint),
                amount: swap.amount,
            });

            // Keep only recent history
            if self.recent_swaps.len() > self.max_history {
                self.recent_swaps
                    .drain(0..self.recent_swaps.len() - self.max_history);
            }
        }
    }

    /// Extract features from an Intent (for API service)
    pub fn extract_from_intent(
        &mut self,
        intent: &sentinel_core::Intent,
        user_pubkey: &Pubkey,
    ) -> FeatureVector {
        let mut features = FeatureVector {
            is_dex_swap: true,
            ..Default::default()
        };

        // Extract swap details
        if let Some(swap_details) = &intent.swap_details {
            features.input_amount = swap_details.amount as f64;
            features.price_impact_bps = (intent.constraints.max_slippage_bps as f64).min(1000.0);

            // Check history for patterns (if we have swap records)
            let swap_data = TransactionData {
                slot: 0, // Will be filled by real-time data
                fee_payer: *user_pubkey,
                compute_unit_limit: intent.fee_preferences.max_priority_fee_lamports as u32,
                compute_unit_price: 0,
                jito_tip_lamports: intent.fee_preferences.max_jito_tip_lamports,
                swap_details: Some(SwapDetailsData {
                    input_mint: swap_details.input_mint,
                    output_mint: swap_details.output_mint,
                    amount: swap_details.amount,
                }),
            };

            features.recent_swaps_same_pair = self.count_recent_swaps_same_pair(&swap_data);
            features.recent_swaps_same_actor = self.count_recent_swaps_same_actor(&swap_data);
        }

        // Set tip context
        features.jito_tip_lamports = intent.fee_preferences.max_jito_tip_lamports;
        features.tip_percentile_vs_recent = 50.0; // Default to median

        features
    }
}

impl Default for FeatureExtractor {
    fn default() -> Self {
        Self::new()
    }
}

/// Raw transaction data for feature extraction
#[derive(Debug, Clone)]
pub struct TransactionData {
    pub slot: u64,
    pub fee_payer: Pubkey,
    pub compute_unit_limit: u32,
    pub compute_unit_price: u64,
    pub jito_tip_lamports: u64,
    pub swap_details: Option<SwapDetailsData>,
}

#[derive(Debug, Clone)]
pub struct SwapDetailsData {
    pub input_mint: Pubkey,
    pub output_mint: Pubkey,
    pub amount: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_vector_to_array() {
        let features = FeatureVector::default();
        let array = features.to_array();
        assert_eq!(array.len(), FeatureVector::feature_count());
    }

    #[test]
    fn test_feature_extractor() {
        let mut extractor = FeatureExtractor::new();
        let tx_data = TransactionData {
            slot: 1000,
            fee_payer: Pubkey::new_unique(),
            compute_unit_limit: 200000,
            compute_unit_price: 1000,
            jito_tip_lamports: 10000,
            swap_details: None,
        };
        let features = extractor.extract(&tx_data);
        assert_eq!(features.slot, 1000);
    }
}
