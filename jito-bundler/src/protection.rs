use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};
use std::str::FromStr;

/// jitodontfront protection marker
pub struct JitoDontFrontMarker;

impl JitoDontFrontMarker {
    /// The official jitodontfront marker public key
    pub fn pubkey() -> Pubkey {
        Pubkey::from_str("jitodontfront111111111111111111111111111111")
            .expect("Valid jitodontfront pubkey")
    }

    /// Add jitodontfront protection to an instruction
    /// This marks the transaction as requiring index-0 placement in bundle
    pub fn add_to_instruction(instruction: &mut Instruction) {
        let marker_pubkey = Self::pubkey();

        // Add as read-only account if not already present
        if !instruction
            .accounts
            .iter()
            .any(|acc| acc.pubkey == marker_pubkey)
        {
            instruction.accounts.push(AccountMeta {
                pubkey: marker_pubkey,
                is_signer: false,
                is_writable: false,
            });
        }
    }

    /// Check if instruction has jitodontfront protection
    pub fn is_protected(instruction: &Instruction) -> bool {
        let marker_pubkey = Self::pubkey();
        instruction
            .accounts
            .iter()
            .any(|acc| acc.pubkey == marker_pubkey)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[allow(deprecated)]
    use solana_sdk::system_instruction;

    #[test]
    fn test_add_protection_marker() {
        let from = Pubkey::new_unique();
        let to = Pubkey::new_unique();
        let mut ix = system_instruction::transfer(&from, &to, 1000);

        assert!(!JitoDontFrontMarker::is_protected(&ix));

        JitoDontFrontMarker::add_to_instruction(&mut ix);

        assert!(JitoDontFrontMarker::is_protected(&ix));
    }

    #[test]
    fn test_marker_pubkey() {
        let pubkey = JitoDontFrontMarker::pubkey();
        assert_eq!(
            pubkey.to_string(),
            "jitodontfront111111111111111111111111111111"
        );
    }
}
