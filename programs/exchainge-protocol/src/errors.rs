use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    // === INPUT VALIDATION ERRORS (6000-6099) ===
    #[msg("Title cannot be empty")]
    EmptyTitle = 6000,
    
    #[msg("Title exceeds maximum length of 100 characters")]
    TitleTooLong = 6001,
    
    #[msg("Description exceeds maximum length of 500 characters")]
    DescriptionTooLong = 6002,
    
    #[msg("Price must be at least 0.001 USDC")]
    InvalidPrice = 6003,
    
    #[msg("Royalty cannot exceed 50%")]
    InvalidRoyalty = 6004,
    
    #[msg("Max owners cannot exceed 10,000")]
    InvalidMaxOwners = 6005,
    
    #[msg("License duration cannot exceed 10 years")]
    InvalidLicenseDuration = 6006,
    
    #[msg("Content hash format is invalid")]
    InvalidContentHash = 6007,
    
    #[msg("Device ID exceeds maximum length")]
    DeviceIdTooLong = 6008,

    // === SP1 VERIFICATION ERRORS (6100-6199) ===
    #[msg("SP1 proof is invalid or malformed")]
    InvalidSP1Proof = 6100,
    
    #[msg("Public values format is incorrect")]
    InvalidPublicValues = 6101,
    
    #[msg("Data commitment does not match proof")]
    CommitmentMismatch = 6102,
    
    #[msg("Verification score below minimum threshold of 60")]
    InsufficientVerification = 6103,
    
    #[msg("Verification is too old, must be within 24 hours")]
    VerificationExpired = 6104,
    
    #[msg("Anti-synthesis score too low")]
    SyntheticDataDetected = 6105,

    // === AUTHORIZATION ERRORS (6200-6299) ===
    #[msg("Only the data provider can perform this action")]
    UnauthorizedProvider = 6200,
    
    #[msg("Access denied - invalid permissions")]
    UnauthorizedAccess = 6201,
    
    #[msg("Only license owner can perform this action")]
    UnauthorizedLicenseHolder = 6202,
    
    #[msg("Admin privileges required")]
    UnauthorizedAdmin = 6203,

    // === LISTING STATE ERRORS (6300-6399) ===
    #[msg("Listing is not active for purchase")]
    ListingInactive = 6300,
    
    #[msg("Data must be verified before purchase")]
    DataNotVerified = 6301,
    
    #[msg("Listing ID does not match")]
    InvalidListingId = 6302,
    
    #[msg("Listing has expired")]
    ListingExpired = 6303,
    
    #[msg("Cannot modify listing after sales")]
    ListingImmutable = 6304,

    // === PAYMENT ERRORS (6400-6499) ===
    #[msg("Payment amount is insufficient")]
    InsufficientPayment = 6400,
    
    #[msg("Token mint does not match USDC")]
    InvalidTokenMint = 6401,
    
    #[msg("Insufficient token balance")]
    InsufficientBalance = 6402,
    
    #[msg("Payment transfer failed")]
    PaymentFailed = 6403,

    // === LICENSE ERRORS (6500-6599) ===
    #[msg("Exclusive license already sold")]
    ExclusiveLicenseAlreadySold = 6500,
    
    #[msg("Maximum number of owners reached")]
    MaxOwnersReached = 6501,
    
    #[msg("License ID does not match")]
    InvalidLicenseId = 6502,
    
    #[msg("License has expired")]
    LicenseExpired = 6503,
    
    #[msg("Usage limit exceeded")]
    UsageLimitExceeded = 6504,
    
    #[msg("License has been revoked")]
    LicenseRevoked = 6505,
    
    #[msg("License is not transferable")]
    TransferNotAllowed = 6506,
    
    #[msg("Maximum transfers exceeded")]
    MaxTransfersReached = 6507,

    // === RATE LIMITING ERRORS (6600-6699) ===
    #[msg("Too many listings for this provider")]
    RateLimitExceeded = 6600,
    
    #[msg("Purchase cooldown period active")]
    PurchaseCooldown = 6601,
    
    #[msg("Too many purchases for this buyer")]
    BuyerLimitExceeded = 6602,

    // === BUSINESS LOGIC ERRORS (6700-6799) ===
    #[msg("Cannot purchase your own listing")]
    SelfPurchaseProhibited = 6700,
    
    #[msg("Duplicate purchase attempted")]
    DuplicatePurchase = 6701,
    
    #[msg("Feature not available for this license type")]
    FeatureNotAvailable = 6702,

    // === SYSTEM ERRORS (6800-6899) ===
    #[msg("Arithmetic overflow detected")]
    ArithmeticOverflow = 6800,
    
    #[msg("Clock timestamp unavailable")]
    ClockUnavailable = 6801,
    
    #[msg("Account initialization failed")]
    InitializationFailed = 6802,
    
    #[msg("PDA derivation failed")]
    InvalidPDA = 6803,
}