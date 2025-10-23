use anchor_lang::prelude::*;

use crate::state::{LicenseType, VerifierType};

#[event]
pub struct PlatformInitialized {
    pub authority: Pubkey,
    pub treasury: Pubkey,
    pub fee_bps: u64,
}

#[event]
pub struct PlatformConfigUpdated {
    pub authority: Pubkey,
    pub treasury: Pubkey,
    pub fee_bps: u64,
    pub paused: bool,
}

#[event]
pub struct DatasetRegistered {
    pub dataset: Pubkey,
    pub owner: Pubkey,
    pub internal_key: String,
    pub data_hash: String,
    pub price_lamports: u64,
    pub verifier_type: VerifierType,
    pub verification_score: u8,
    pub license_type: LicenseType,
}

#[event]
pub struct DatasetUpdated {
    pub dataset: Pubkey,
    pub metadata_uri: String,
    pub price_lamports: u64,
}

#[event]
pub struct DatasetVerified {
    pub dataset: Pubkey,
    pub verifier_type: VerifierType,
    pub verification_score: u8,
}

#[event]
pub struct DatasetPurchased {
    pub purchase: Pubkey,
    pub dataset: Pubkey,
    pub buyer: Pubkey,
    pub seller: Pubkey,
    pub total_amount: u64,
    pub platform_fee: u64,
    pub seller_revenue: u64,
}

#[event]
pub struct AccessGranted {
    pub buyer: Pubkey,
    pub dataset: Pubkey,
    pub expires_at: i64,
}
