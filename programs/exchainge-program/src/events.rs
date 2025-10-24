use anchor_lang::prelude::*;

/// Event emitted when a dataset is registered with on-chain hash computation (trustless method)
#[event]
pub struct DatasetRegistered {
    pub dataset_id: u64,
    pub file_size: u64,
    pub file_key: String,
    pub derived_hash: [u8; 32],
    pub owner: Pubkey,
    pub timestamp: i64,
}

/// Event emitted when a pre-computed hash is registered (cheaper method)
#[event]
pub struct HashRegistered {
    pub internal_key: String,
    pub dataset_hash: [u8; 32],
    pub owner: Pubkey,
    pub timestamp: i64,
}

/// Event emitted when a registry entry is updated
#[event]
pub struct RegistryUpdated {
    pub registry_address: Pubkey,
    pub new_hash: [u8; 32],
    pub owner: Pubkey,
    pub timestamp: i64,
}

/// Event emitted when a registry is closed
#[event]
pub struct RegistryClosed {
    pub registry_address: Pubkey,
    pub owner: Pubkey,
    pub timestamp: i64,
}
