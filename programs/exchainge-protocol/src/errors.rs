use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Platform is paused")]
    PlatformPaused = 6000,
    #[msg("Unauthorized: only platform authority can perform this action")]
    Unauthorized = 6001,

    #[msg("URI must be non-empty and within allowed length")]
    InvalidUri = 6010,
    #[msg("Content hash must be valid SHA256 with 64 hex characters")]
    InvalidContentHash = 6011,
    #[msg("Internal key must be non-empty and within allowed length")]
    InvalidInternalKey = 6012,
    #[msg("Verification score must be between 0 and 100")]
    InvalidScore = 6013,
    #[msg("Verification score too low, must be greater than 50")]
    LowScore = 6014,
    #[msg("Price must meet minimum to prevent spam")]
    PriceTooLow = 6015,
    #[msg("Price exceeds maximum allowed value")]
    PriceTooHigh = 6016,
    #[msg("Dataset with this internal key already exists for this owner")]
    DuplicateDataset = 6017,

    #[msg("Dataset must be verified before purchase")]
    DatasetNotVerified = 6030,
    #[msg("Cannot purchase your own dataset")]
    SelfPurchaseNotAllowed = 6031,
    #[msg("You already purchased this dataset")]
    AlreadyPurchased = 6032,
    #[msg("Insufficient funds for purchase")]
    InsufficientFunds = 6033,
    #[msg("Payment transfer failed")]
    PaymentFailed = 6034,

    #[msg("Arithmetic overflow detected")]
    MathOverflow = 6050,
    #[msg("Arithmetic underflow detected")]
    MathUnderflow = 6051,
    #[msg("Division by zero")]
    DivisionByZero = 6052,

    #[msg("Only dataset owner can update or delete")]
    NotDatasetOwner = 6060,
    #[msg("Purchase record not found, access denied")]
    NoPurchaseRecord = 6061,
}
