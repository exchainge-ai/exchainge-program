use anchor_lang::prelude::*;
use crate::state::{LicenseType, AccessType};

#[event]
pub struct ListingCreated {
    pub listing_id: Pubkey,
    pub provider: Pubkey,
    pub title: String,
    pub price_usdc: u64,
    pub license_type: LicenseType,
}

#[event]
pub struct DataVerified {
    pub listing_id: Pubkey,
    pub commitment: [u8; 32],
    pub verification_score: u8,
    pub verifier: Pubkey,
}

#[event]
pub struct LicensePurchased {
    pub listing_id: Pubkey,
    pub buyer: Pubkey,
    pub license_id: Pubkey,
    pub price: u64,
    pub platform_fee: u64,
}

#[event]
pub struct DataAccessed {
    pub license_id: Pubkey,
    pub user: Pubkey,
    pub access_type: AccessType,
    pub timestamp: i64,
}