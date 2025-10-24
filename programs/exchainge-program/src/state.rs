use anchor_lang::prelude::*;

/// Registry account storing dataset metadata and hash
/// Supports both trustless (on-chain computed) and pre-computed hash methods
#[account]
pub struct DataRegistry {
    /// Owner/creator of this registry entry
    pub owner: Pubkey,

    /// Unique identifier (internal_key or derived from dataset_id)
    pub internal_key: String,

    /// SHA-256 hash of the dataset (32 bytes)
    pub dataset_hash: [u8; 32],

    /// Optional: Original dataset_id (if using trustless method)
    pub dataset_id: Option<u64>,

    /// Optional: File size in bytes (if using trustless method)
    pub file_size: Option<u64>,

    /// Optional: File key used for hash derivation (if using trustless method)
    pub file_key: Option<String>,

    /// Timestamp when registered
    pub created_at: i64,

    /// PDA bump seed (if using PDA)
    pub bump: u8,
}

impl DataRegistry {
    /// Calculate space needed for account
    /// 8 (discriminator) + 32 (owner) + 4+64 (internal_key) + 32 (hash)
    /// + 1+8 (Option<dataset_id>) + 1+8 (Option<file_size>)
    /// + 1+4+100 (Option<file_key>) + 8 (created_at) + 1 (bump)
    pub const LEN: usize = 8 + 32 + 68 + 32 + 9 + 9 + 105 + 8 + 1;
}
