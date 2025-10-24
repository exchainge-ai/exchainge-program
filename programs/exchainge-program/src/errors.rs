use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Internal key is empty or too long (max 64 chars)")]
    InvalidInternalKey,

    #[msg("Dataset hash is invalid")]
    InvalidDatasetHash,

    #[msg("File key is empty or too long (max 100 chars)")]
    InvalidFileKey,

    #[msg("File size must be greater than 0")]
    InvalidFileSize,

    #[msg("Dataset ID must be greater than 0")]
    InvalidDatasetId,

    #[msg("Registry entry already exists for this key")]
    RegistryAlreadyExists,

    #[msg("Unauthorized: only owner can update or close")]
    Unauthorized,
}
