use thiserror::Error;

#[derive(Error, Debug)]
pub enum QRCryptError {
    #[error("Encryption error: {0}")]
    Encryption(String),

    #[error("Decryption error: {0}")]
    Decryption(String),

    #[error("QR code generation error: {0}")]
    QRGeneration(String),

    #[error("QR code parsing error: {0}")]
    QRParsing(String),

    #[error("Shamir's Secret Sharing error: {0}")]
    ShamirError(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Base64 decode error: {0}")]
    Base64Decode(#[from] base64::DecodeError),

    #[error("Hex decode error: {0}")]
    HexDecode(#[from] hex::FromHexError),
}

pub type Result<T> = std::result::Result<T, QRCryptError>;
