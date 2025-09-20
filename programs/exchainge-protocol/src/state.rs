use anchor_lang::prelude::*;

/// Data listing for marketplace
#[account]
pub struct DataListing {
    pub provider: Pubkey,                    // Data owner (immutable)
    pub title: String,                       // Max 100 chars, validated
    pub description: String,                 // Max 500 chars, optional
    pub price_usdc: u64,                    // Price in USDC microunits (min 1)
    pub license_type: LicenseType,          // Enum, validated
    pub usage_rights: UsageRights,          // Struct with permissions
    pub content_hash: String,               // IPFS/Arweave hash, validated format
    pub verification_status: bool,          // True after SP1 verification
    pub commitment: [u8; 32],               // Data commitment from SP1
    pub royalty_bps: u16,                   // Max 5000 (50%), validated
    pub max_owners: Option<u32>,            // Max 10000, validated
    pub license_duration: Option<i64>,      // Max 10 years from creation
    pub created_at: i64,                    // Immutable timestamp
    pub updated_at: i64,                    // Last modification
    pub is_active: bool,                    // Available for purchase
    pub total_sales: u32,                   // Number of licenses sold
    pub revenue_earned: u64,                // Total USDC earned
    pub bump: u8,                           // PDA bump for security
}

impl DataListing {
    // Account discriminator (8) + provider (32) + title (4 + 100) + description (4 + 500) +
    // price_usdc (8) + license_type (1) + usage_rights (7) + content_hash (4 + 100) +
    // verification_status (1) + commitment (32) + royalty_bps (2) + max_owners (1 + 4) +
    // license_duration (1 + 8) + created_at (8) + updated_at (8) + is_active (1) +
    // total_sales (4) + revenue_earned (8) + bump (1)
    pub const LEN: usize = 8 + 32 + 4 + 100 + 4 + 500 + 8 + 1 + 7 + 4 + 100 + 1 + 32 + 2 + 1 + 4 + 1 + 8 + 8 + 8 + 1 + 4 + 8 + 1;
    
    pub fn validate_update(&self, new_price: Option<u64>, _current_time: i64) -> bool {
        // Can only update if no sales yet or price increase only
        if self.total_sales > 0 {
            if let Some(price) = new_price {
                return price >= self.price_usdc; // Only allow price increases
            }
        }
        true
    }
    
    pub fn is_expired(&self, current_time: i64) -> bool {
        if let Some(duration) = self.license_duration {
            return current_time > duration;
        }
        false
    }
}

/// Hardware verification record with tamper protection
#[account]
pub struct HardwareVerification {
    pub listing_id: Pubkey,                 // Parent listing (immutable)
    pub data_commitment: [u8; 32],          // Hash of verified data (immutable)
    pub vendor: HardwareVendor,             // DJI, NVIDIA, etc.
    pub device_id: String,                  // Device identifier (max 64 chars)
    pub verification_score: u8,             // 0-100 confidence (min 60)
    pub physics_verified: bool,             // Passed physics checks
    pub anti_synthesis_score: u8,           // ML confidence (min 60)
    pub verifier_pubkey: Pubkey,            // Who submitted proof (immutable)
    pub sp1_proof_hash: [u8; 32],          // Hash of SP1 proof (immutable)
    pub public_values_hash: [u8; 32],      // Hash of public values (immutable)
    pub verified_at: i64,                   // Timestamp (immutable)
    pub nonce: u64,                         // Anti-replay protection
    pub bump: u8,                           // PDA bump for security
}

impl HardwareVerification {
    // Account discriminator (8) + listing_id (32) + data_commitment (32) + vendor (1) +
    // device_id (4 + 64) + verification_score (1) + physics_verified (1) +
    // anti_synthesis_score (1) + verifier_pubkey (32) + sp1_proof_hash (32) +
    // public_values_hash (32) + verified_at (8) + nonce (8) + bump (1)
    pub const LEN: usize = 8 + 32 + 32 + 1 + 4 + 64 + 1 + 1 + 1 + 32 + 32 + 32 + 8 + 8 + 1;
    
    pub fn is_valid_score(&self) -> bool {
        self.verification_score >= MIN_VERIFICATION_THRESHOLD &&
        self.anti_synthesis_score >= MIN_VERIFICATION_THRESHOLD
    }
    
    pub fn is_recent(&self, current_time: i64) -> bool {
        current_time - self.verified_at <= MAX_VERIFICATION_AGE
    }
}

/// License token with comprehensive access control
#[account]
pub struct LicenseToken {
    pub listing_id: Pubkey,                 // Source dataset (immutable)
    pub owner: Pubkey,                      // Current license holder
    pub original_buyer: Pubkey,             // First buyer (immutable)
    pub license_type: LicenseType,          // Inherited from listing (immutable)
    pub usage_rights: UsageRights,          // Inherited permissions (immutable)
    pub purchase_price: u64,                // Amount paid (immutable)
    pub purchase_date: i64,                 // When purchased (immutable)
    pub expiration: Option<i64>,            // When expires (if any)
    pub access_count: u32,                  // Times data accessed
    pub last_access: i64,                   // Last access timestamp
    pub usage_limit: Option<u32>,           // Max accesses allowed
    pub is_transferable: bool,              // Can be resold
    pub transfer_count: u8,                 // Number of transfers (max 3)
    pub is_revoked: bool,                   // Emergency revocation flag
    pub bump: u8,                           // PDA bump for security
}

impl LicenseToken {
    // Account discriminator (8) + listing_id (32) + owner (32) + original_buyer (32) +
    // license_type (1) + usage_rights (7) + purchase_price (8) + purchase_date (8) +
    // expiration (1 + 8) + access_count (4) + last_access (8) + usage_limit (1 + 4) +
    // is_transferable (1) + transfer_count (1) + is_revoked (1) + bump (1)
    pub const LEN: usize = 8 + 32 + 32 + 32 + 1 + 7 + 8 + 8 + 1 + 8 + 4 + 8 + 1 + 4 + 1 + 1 + 1 + 1;
    
    pub fn can_access(&self, current_time: i64) -> bool {
        if self.is_revoked {
            return false;
        }
        
        if let Some(expiry) = self.expiration {
            if current_time > expiry {
                return false;
            }
        }
        
        if let Some(limit) = self.usage_limit {
            if self.access_count >= limit {
                return false;
            }
        }
        
        true
    }
    
    pub fn can_transfer(&self) -> bool {
        self.is_transferable && !self.is_revoked && self.transfer_count < MAX_TRANSFERS
    }
}

/// License types matching frontend specification
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq)]
pub enum LicenseType {
    ViewOnly,              // Read-only access
    ViewOnlyShared,        // Shared read access  
    SharedOwnership,       // Multiple owners allowed
    Exclusive,             // Single owner only
    TransferableExclusive, // Exclusive but can transfer
}

/// Hardware vendor enumeration
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq)]
pub enum HardwareVendor {
    DJI,
    NVIDIA,
    Qualcomm,
    Custom,
}

/// Access types for data usage
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq)]
pub enum AccessType {
    Download,     // Full file download
    Stream,       // Real-time streaming
    API,          // API endpoint access
    Compute,      // Compute-to-data
}

/// Usage rights structure matching frontend specification
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct UsageRights {
    pub commercial_use: bool,               // Can use commercially
    pub derivative_works_allowed: bool,     // Can create derivatives
    pub redistribution_allowed: bool,       // Can redistribute
    pub attribution_required: bool,         // Must credit original
    pub consent_required: bool,             // Need seller approval for resale
    pub ai_training_allowed: bool,          // Can use for AI training
    pub geographic_restrictions: bool,      // Has location limits
}

/// Security and validation constants for enterprise use
pub const PLATFORM_FEE_BPS: u64 = 300; // 3% platform fee
pub const MIN_VERIFICATION_THRESHOLD: u8 = 60; // Minimum verification score
pub const MAX_TITLE_LENGTH: usize = 100;
pub const MAX_DESCRIPTION_LENGTH: usize = 500;
pub const MAX_DEVICE_ID_LENGTH: usize = 64;
pub const MAX_CONTENT_HASH_LENGTH: usize = 100;

// Security limits
pub const MAX_ROYALTY_BPS: u16 = 5000; // Max 50% royalty
pub const MAX_OWNERS_LIMIT: u32 = 10000; // Max owners per listing
pub const MAX_LICENSE_DURATION: i64 = 10 * 365 * 24 * 60 * 60; // 10 years
pub const MAX_VERIFICATION_AGE: i64 = 24 * 60 * 60; // 24 hours
pub const MAX_TRANSFERS: u8 = 3; // Max license transfers
pub const MIN_PRICE_USDC: u64 = 1000; // Minimum 0.001 USDC

// Anti-spam and DoS protection
pub const MAX_LISTINGS_PER_PROVIDER: u32 = 1000;
pub const MAX_PURCHASES_PER_BUYER: u32 = 100;
pub const MIN_TIME_BETWEEN_PURCHASES: i64 = 60; // 1 minute cooldown

// Enterprise validation patterns
pub const IPFS_HASH_PREFIX: &str = "Qm";
pub const ARWEAVE_HASH_LENGTH: usize = 43;