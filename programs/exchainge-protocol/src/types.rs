use anchor_lang::prelude::*;

/// Maximum number of sensor readings we persist on-chain for a single verification.
pub const MAX_SENSOR_READINGS: usize = 50;

/// Hardware signatures must be submitted within this window (in seconds).
pub const HARDWARE_SIGNATURE_MAX_AGE: u64 = 300; // 5 minutes

/// Maximum number of verification records an oracle can create inside a rolling window.
pub const MAX_VERIFICATIONS_PER_DAY: u16 = 144; // Every 10 minutes on average

/// Core listing data structure with streamlined licensing for the MVP.
#[account]
#[derive(InitSpace)]
pub struct Listing {
    /// Unique identifier for this listing
    pub id: Pubkey,

    /// Human-readable title (max 100 chars for cost control)
    #[max_len(100)]
    pub title: String,

    /// Price denominated in the smallest USDC unit
    pub price_usdc: u64,

    /// Simplified license bucket for MVP
    pub license_type: LicenseType,

    /// Content hash (IPFS CID or SHA256) for data integrity
    #[max_len(200)]
    pub hash: String,

    /// Usage rights metadata shared with buyers
    pub usage_rights: UsageRights,

    /// Royalty percentage in basis points (0-10000 = 0-100%)
    pub royalty_bps: u16,

    /// Original creator/owner of the dataset
    pub original_owner: Pubkey,

    /// Current owners/viewers depending on license type
    #[max_len(1000)]
    pub current_owners: Vec<Pubkey>,

    /// Maximum number of allowed owners (None = unlimited)
    pub max_owners: Option<u32>,

    /// License duration in days (None = perpetual)
    pub license_duration_days: Option<u32>,

    /// Whether this listing is active for new purchases
    pub is_active: bool,

    /// Timestamp when listing was created
    pub created_at: i64,

    /// Total number of purchases made
    pub total_purchases: u64,

    /// Total revenue generated from this listing
    pub total_revenue: u64,
}

/// Comprehensive purchase record with licensing details
#[account]
#[derive(InitSpace)]
pub struct PurchaseRecord {
    pub listing_id: Pubkey,
    pub buyer: Pubkey,
    pub purchase_price: u64,
    pub royalty_paid: u64,
    pub purchased_at: i64,
    pub license_expires_at: Option<i64>,
    pub usage_rights: UsageRights,
    pub can_resell: bool,
    pub resale_consent_required: bool,
}

/// Consent request for resale permissions
#[account]
#[derive(InitSpace)]
pub struct ConsentRequest {
    pub listing_id: Pubkey,
    pub requester: Pubkey,
    pub original_owner: Pubkey,
    pub is_approved: bool,
    pub requested_at: i64,
    pub decided_at: Option<i64>,
}

/// Anti-piracy proof record for authenticity verification
#[account]
#[derive(InitSpace)]
pub struct ProofRecord {
    pub listing_id: Pubkey,
    pub submitter: Pubkey,
    #[max_len(200)]
    pub proof_hash: String,
    pub proof_type: ProofType,
    pub submitted_at: i64,
    pub is_verified: bool,
    pub verification_score: u8,
}

/// Registry gating hardware oracle onboarding.
#[account]
#[derive(InitSpace)]
pub struct OracleRegistry {
    pub authority: Pubkey,
    #[max_len(64)]
    pub allowed_operators: Vec<Pubkey>,
    pub max_verifications_per_day: u16,
}

/// Hardware oracle registration for drone/robot data verification
#[account]
#[derive(InitSpace)]
pub struct HardwareOracle {
    #[max_len(64)]
    pub hardware_id: String,
    pub hardware_type: HardwareType,
    pub operator: Pubkey,
    pub public_key: [u8; 32],
    #[max_len(200)]
    pub certification_hash: String,
    pub trusted_hardware: TrustedHardware,
    pub is_active: bool,
    pub registered_at: i64,
    pub total_verifications: u64,
    /// Tracks the last day (UTC) we updated the daily counter to apply rate limits.
    pub last_verification_day: i64,
    pub verifications_today: u16,
}

/// Hardware data verification record
#[account]
#[derive(InitSpace)]
pub struct HardwareVerification {
    pub listing_id: Pubkey,
    pub oracle_id: Pubkey,
    #[max_len(200)]
    pub data_hash: String,
    pub hardware_signature: [u8; 64],
    pub timestamp: i64,
    #[max_len(100)]
    pub location_hash: Option<String>,
    #[max_len(50)]
    pub sensor_readings: Vec<SensorReading>,
    pub verified_at: i64,
    pub is_verified: bool,
    pub verification_score: u8,
}

/// Streaming metadata used to monetize real-time feeds.
#[account]
#[derive(InitSpace)]
pub struct RealTimeDataStream {
    pub hardware_id: Pubkey,
    pub stream_hash: [u8; 32],
    pub samples_per_second: u16,
    pub cryptographic_proof: StreamProof,
}

/// Simplified license catalogue for the MVP.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, InitSpace)]
pub enum LicenseType {
    /// Read-only access, optimized for AI teams needing reproducibility.
    ViewOnly,
    /// Exclusive rights with resale/transfer gating.
    Exclusive,
}

/// Usage rights exposed to buyers. Keep flexible for roadmap without exploding state.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, InitSpace)]
pub struct UsageRights {
    pub commercial_use: bool,
    pub ai_training_allowed: bool,
    pub derivative_works_allowed: bool,
    pub redistribution_allowed: bool,
    pub consent_required: bool,
}

/// Supported proof primitives in the MVP.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, InitSpace)]
pub enum ProofType {
    ZKProofOfPossession,
    MerkleRoot,
    DigitalSignature,
    PhysicsConsistency,
    Custom,
}

/// Trusted hardware families we whitelist up-front.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, InitSpace)]
pub enum TrustedHardware {
    DJIDrone {
        #[max_len(64)]
        serial: String,
        firmware_hash: [u8; 32],
    },
    NvidiaJetson {
        #[max_len(64)]
        device_id: String,
        silicon_id: [u8; 16],
    },
    QualcommRb5 {
        #[max_len(64)]
        device_id: String,
        firmware_hash: [u8; 32],
    },
    Other,
}

/// Types of hardware that can serve as oracles
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, InitSpace)]
pub enum HardwareType {
    Drone,
    Robot,
    IoTSensor,
    Satellite,
    WeatherStation,
    Vehicle,
    Custom,
}

/// Individual sensor reading from hardware
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, InitSpace)]
pub struct SensorReading {
    #[max_len(32)]
    pub sensor_type: String,
    pub value: f64,
    #[max_len(16)]
    pub unit: String,
    pub is_calibrated: bool,
}

/// Lightweight proof container for streaming feeds.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, InitSpace)]
pub struct StreamProof {
    pub proof_type: ProofType,
    pub proof_hash: [u8; 32],
    pub submitted_at: i64,
}

// ================================
// EVENTS
// ================================

#[event]
pub struct ListingCreated {
    pub listing_id: Pubkey,
    pub original_owner: Pubkey,
    pub title: String,
    pub price_usdc: u64,
    pub license_type: LicenseType,
}

#[event]
pub struct DataPurchased {
    pub listing_id: Pubkey,
    pub buyer: Pubkey,
    pub price: u64,
    pub royalty_paid: u64,
    pub license_type: LicenseType,
}

#[event]
pub struct ResaleConsentRequested {
    pub listing_id: Pubkey,
    pub requester: Pubkey,
}

#[event]
pub struct ResaleConsentDecided {
    pub listing_id: Pubkey,
    pub requester: Pubkey,
    pub approved: bool,
}

#[event]
pub struct ProofSubmitted {
    pub listing_id: Pubkey,
    pub submitter: Pubkey,
    pub proof_hash: String,
    pub proof_type: ProofType,
}

#[event]
pub struct ListingDeactivated {
    pub listing_id: Pubkey,
    pub authority: Pubkey,
    pub reason: String,
}

#[event]
pub struct HardwareOracleRegistered {
    pub oracle_id: Pubkey,
    pub hardware_id: String,
    pub hardware_type: HardwareType,
    pub operator: Pubkey,
}

#[event]
pub struct HardwareDataVerified {
    pub listing_id: Pubkey,
    pub oracle_id: Pubkey,
    pub data_hash: String,
    pub hardware_type: HardwareType,
    pub verification_score: u8,
}

#[event]
pub struct HardwareOracleDeactivated {
    pub oracle_id: Pubkey,
    pub hardware_id: String,
    pub operator: Pubkey,
    pub reason: String,
}

#[event]
pub struct RegistryUpdated {
    pub authority: Pubkey,
    pub allowed_operator_count: u64,
}

// ================================
// ERROR HANDLING
// ================================

#[error_code]
pub enum ExchaingeError {
    #[msg("Title too long (max 100 characters)")]
    TitleTooLong,
    #[msg("Hash too long (max 200 characters)")]
    HashTooLong,
    #[msg("Invalid royalty percentage (max 100%)")]
    InvalidRoyalty,
    #[msg("Invalid price (must be greater than 0)")]
    InvalidPrice,
    #[msg("Invalid maximum owners (must be 1-1000)")]
    InvalidMaxOwners,
    #[msg("Invalid license duration (must be 1-3650 days)")]
    InvalidLicenseDuration,
    #[msg("Listing is not active")]
    ListingInactive,
    #[msg("Exclusive license already sold")]
    ExclusiveLicenseAlreadySold,
    #[msg("Maximum number of owners reached")]
    MaxOwnersReached,
    #[msg("License has expired")]
    LicenseExpired,
    #[msg("Invalid token mint (must be USDC)")]
    InvalidTokenMint,
    #[msg("Invalid token account owner")]
    InvalidTokenOwner,
    #[msg("Buyer already owns this data")]
    AlreadyOwned,
    #[msg("Resale not allowed for this license")]
    ResaleNotAllowed,
    #[msg("Not authorized to perform this action")]
    NotAuthorized,
    #[msg("Consent not required for this license type")]
    ConsentNotRequired,
    #[msg("Consent required but not provided")]
    ConsentRequired,
    #[msg("Consent not approved")]
    ConsentNotApproved,
    #[msg("Proof hash too long (max 200 characters)")]
    ProofHashTooLong,
    #[msg("Proof hash cannot be empty")]
    EmptyProofHash,
    #[msg("Reason too long (max 200 characters)")]
    ReasonTooLong,
    #[msg("Hardware ID too long (max 64 characters)")]
    HardwareIdTooLong,
    #[msg("Certification hash too long (max 200 characters)")]
    CertificationHashTooLong,
    #[msg("Hardware ID cannot be empty")]
    EmptyHardwareId,
    #[msg("Certification hash cannot be empty")]
    EmptyCertificationHash,
    #[msg("Data hash cannot be empty")]
    EmptyDataHash,
    #[msg("Hardware oracle is not active")]
    OracleInactive,
    #[msg("Too many sensor readings (max 50)")]
    TooManySensorReadings,
    #[msg("Timestamp too old (max 5 minutes)")]
    TimestampTooOld,
    #[msg("Location hash too long (max 100 characters)")]
    LocationHashTooLong,
    #[msg("Oracle operator not whitelisted")]
    OracleNotWhitelisted,
    #[msg("Invalid public key provided")]
    InvalidPublicKey,
    #[msg("Invalid signature payload")]
    InvalidSignature,
    #[msg("Signature verification failed")]
    SignatureVerificationFailed,
    #[msg("Hardware verification already exists for this payload")]
    DuplicateVerification,
    #[msg("Oracle has reached the daily verification quota")]
    RateLimitExceeded,
    #[msg("Operator whitelist exceeds supported length")]
    OperatorListTooLong,
    #[msg("Rate limit must be greater than zero")]
    InvalidRateLimit,
}
