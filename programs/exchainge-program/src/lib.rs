#![allow(unexpected_cfgs)]

use anchor_lang::prelude::*;

pub mod state;
pub mod instructions;
pub mod errors;
pub mod events;

pub use state::*;
pub use instructions::*;
pub use events::*;

declare_id!("2yvGQ26fz2mvPnxDa2wcTf5Y88hr9sTSJpiZdFqMyQ4L");

/// ExchAInge Data Registry Program
///
/// Supports two methods for dataset registration:
/// 1. Trustless (on-chain hash): register_dataset - computes SHA-256 on-chain
/// 2. Cheaper (pre-computed): register_hash - stores client-provided hash
#[program]
pub mod exchainge_program {
    use super::*;

    /// Register dataset with on-chain SHA-256 computation (trustless method)
    ///
    /// Computes hash = SHA256(file_key:dataset_id:file_size)
    /// More expensive but fully trustless - no client computation needed
    ///
    /// # Arguments
    /// * `dataset_id` - Unique dataset identifier
    /// * `file_size` - File size in bytes
    /// * `file_key` - File key/identifier used in hash computation
    pub fn register_dataset(
        ctx: Context<RegisterDataset>,
        dataset_id: u64,
        file_size: u64,
        file_key: String,
    ) -> Result<()> {
        instructions::process_register_dataset(ctx, dataset_id, file_size, file_key)
    }

    /// Register with pre-computed hash (cheaper method)
    ///
    /// Client provides the hash - cheaper but requires trusting the client
    ///
    /// # Arguments
    /// * `internal_key` - Unique identifier for this registry entry
    /// * `dataset_hash` - Pre-computed SHA-256 hash (32 bytes)
    pub fn register_hash(
        ctx: Context<RegisterHash>,
        internal_key: String,
        dataset_hash: [u8; 32],
    ) -> Result<()> {
        instructions::process_register_hash(ctx, internal_key, dataset_hash)
    }

    /// Update hash for existing registry entry (owner only)
    ///
    /// # Arguments
    /// * `new_dataset_hash` - New hash to store
    pub fn update_hash(
        ctx: Context<UpdateHash>,
        new_dataset_hash: [u8; 32],
    ) -> Result<()> {
        instructions::process_update_hash(ctx, new_dataset_hash)
    }

    /// View/query registry data by account address (read-only)
    ///
    /// Returns registry information in logs. Clients can also fetch directly
    /// using Connection.getAccountInfo() for more efficient reads.
    pub fn view_hash(ctx: Context<ViewHash>) -> Result<()> {
        instructions::process_view_hash(ctx)
    }

    /// Close registry and reclaim rent (owner only)
    pub fn close_registry(ctx: Context<CloseRegistry>) -> Result<()> {
        instructions::process_close_registry(ctx)
    }
}
