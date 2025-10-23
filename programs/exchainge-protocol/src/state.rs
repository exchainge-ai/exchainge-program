use anchor_lang::prelude::*;

// String length constraints
pub const MAX_URI_LENGTH: usize = 200;
pub const MAX_HASH_LENGTH: usize = 64;
pub const MAX_INTERNAL_KEY_LENGTH: usize = 64;

// TODO: Confirm final fee before mainnet.
pub const PLATFORM_FEE_BPS: u64 = 500; // 5% fee
pub const BPS_DENOMINATOR: u64 = 10000;

// Minimum price prevents spam listings.
pub const MIN_PRICE_LAMPORTS: u64 = 100_000; // 0.0001 SOL

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, InitSpace)]
pub enum VerifierType {
    Metadata,
    AiAgent,
    ZkProof, // TODO: Implement SP1 verification.
    HardwareAttestation, // TODO: Implement device signature verification.
}

impl Default for VerifierType {
    fn default() -> Self {
        VerifierType::Metadata
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, InitSpace)]
pub enum LicenseType {
    MIT,
    Commercial,
    ResearchOnly,
    Custom,
}

impl Default for LicenseType {
    fn default() -> Self {
        LicenseType::MIT
    }
}

#[account]
#[derive(InitSpace)]
pub struct Dataset {
    pub owner: Pubkey,
    #[max_len(MAX_INTERNAL_KEY_LENGTH)]
    pub internal_key: String,
    #[max_len(MAX_URI_LENGTH)]
    pub metadata_uri: String,
    #[max_len(MAX_HASH_LENGTH)]
    pub data_hash: String,
    pub verified: bool,
    pub verifier_type: VerifierType,
    pub verification_score: u8,
    pub license_type: LicenseType,
    pub price_lamports: u64,
    pub seller_revenue: u64,
    pub purchase_count: u64,
    pub created_at: i64,
    pub updated_at: i64,
    pub bump: u8,
}

// One purchase per buyer-dataset pair prevents double-purchase.
#[account]
#[derive(InitSpace)]
pub struct Purchase {
    pub buyer: Pubkey,
    pub dataset: Pubkey,
    pub amount_paid: u64,
    pub platform_fee: u64,
    pub purchased_at: i64,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct PlatformConfig {
    pub authority: Pubkey,
    pub treasury: Pubkey,
    pub fee_bps: u64,
    pub paused: bool,
    pub total_platform_revenue: u64,
    pub total_datasets: u64,
    pub total_purchases: u64,
    pub bump: u8,
}
