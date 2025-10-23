//! Durable Nonce Management for Replay Protection
//!
//! Provides functionality to create, manage, and use durable nonce accounts
//! for replay protection. Durable nonces allow transactions to remain valid
//! indefinitely until executed, unlike recent_blockhash which expires in ~90 seconds.
//!
//! NOTE: This is infrastructure/interface code for durable nonce support.
//! For production use, transactions currently use recent_blockhash with 150-slot validity.
//! Full durable nonce integration with Solana 2.0 APIs coming in future updates.

use solana_sdk::{hash::Hash, pubkey::Pubkey};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

/// Manages durable nonce accounts for replay protection
#[derive(Clone)]
pub struct NonceManager {
    nonce_accounts: Arc<RwLock<HashMap<Pubkey, NonceAccountInfo>>>,
    rpc_endpoint: String,
}

/// Information about a nonce account
#[derive(Debug, Clone)]
pub struct NonceAccountInfo {
    pub address: Pubkey,
    pub current_nonce: Hash,
    pub authority: Pubkey,
    pub lamports: u64,
    pub last_updated: i64,
}

impl NonceManager {
    /// Create a new nonce manager
    pub fn new(rpc_endpoint: String) -> Self {
        info!(
            "✅ NonceManager initialized with endpoint: {}",
            rpc_endpoint
        );
        info!("   Using recent_blockhash for replay protection (150-slot validity)");
        info!("   Durable nonce infrastructure ready - full integration coming soon");

        Self {
            nonce_accounts: Arc::new(RwLock::new(HashMap::new())),
            rpc_endpoint,
        }
    }

    /// Check if nonce management is available
    pub fn is_available(&self) -> bool {
        // Infrastructure is in place for nonce management
        true
    }

    /// Get RPC endpoint
    pub fn endpoint(&self) -> &str {
        &self.rpc_endpoint
    }

    /// List managed nonce accounts
    pub async fn list_nonce_accounts(&self) -> Vec<NonceAccountInfo> {
        let cache = self.nonce_accounts.read().await;
        cache.values().cloned().collect()
    }

    /// Add a nonce account to the cache
    pub async fn add_nonce_account(&self, info: NonceAccountInfo) {
        let mut cache = self.nonce_accounts.write().await;
        info!("Adding nonce account {} to cache", info.address);
        cache.insert(info.address, info);
    }

    /// Remove a nonce account from the cache
    pub async fn remove_nonce_account(&self, address: &Pubkey) {
        let mut cache = self.nonce_accounts.write().await;
        cache.remove(address);
    }

    /// Get a specific nonce account from the cache
    pub async fn get_nonce_account(&self, address: &Pubkey) -> Option<NonceAccountInfo> {
        let cache = self.nonce_accounts.read().await;
        cache.get(address).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nonce_manager_creation() {
        let manager = NonceManager::new("https://api.devnet.solana.com".to_string());
        assert!(manager.is_available());
        assert_eq!(manager.endpoint(), "https://api.devnet.solana.com");
    }

    #[tokio::test]
    async fn test_list_empty_nonces() {
        let manager = NonceManager::new("https://api.devnet.solana.com".to_string());
        let nonces = manager.list_nonce_accounts().await;
        assert_eq!(nonces.len(), 0);
    }

    #[tokio::test]
    async fn test_add_nonce_account() {
        let manager = NonceManager::new("https://api.devnet.solana.com".to_string());

        let info = NonceAccountInfo {
            address: Pubkey::new_unique(),
            current_nonce: Hash::new_unique(),
            authority: Pubkey::new_unique(),
            lamports: 1000000,
            last_updated: 1234567890,
        };

        manager.add_nonce_account(info.clone()).await;

        let nonces = manager.list_nonce_accounts().await;
        assert_eq!(nonces.len(), 1);
        assert_eq!(nonces[0].address, info.address);
    }

    #[tokio::test]
    async fn test_remove_nonce_account() {
        let manager = NonceManager::new("https://api.devnet.solana.com".to_string());

        let info = NonceAccountInfo {
            address: Pubkey::new_unique(),
            current_nonce: Hash::new_unique(),
            authority: Pubkey::new_unique(),
            lamports: 1000000,
            last_updated: 1234567890,
        };

        manager.add_nonce_account(info.clone()).await;
        assert_eq!(manager.list_nonce_accounts().await.len(), 1);

        manager.remove_nonce_account(&info.address).await;
        assert_eq!(manager.list_nonce_accounts().await.len(), 0);
    }

    #[tokio::test]
    async fn test_get_nonce_account() {
        let manager = NonceManager::new("https://api.devnet.solana.com".to_string());

        let info = NonceAccountInfo {
            address: Pubkey::new_unique(),
            current_nonce: Hash::new_unique(),
            authority: Pubkey::new_unique(),
            lamports: 1000000,
            last_updated: 1234567890,
        };

        manager.add_nonce_account(info.clone()).await;

        let retrieved = manager.get_nonce_account(&info.address).await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().address, info.address);

        let not_found = manager.get_nonce_account(&Pubkey::new_unique()).await;
        assert!(not_found.is_none());
    }
}
