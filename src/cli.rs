use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "qrcrypt")]
#[command(about = "Secure storage of crypto wallet seed phrases in encrypted QR codes")]
#[command(version = "0.1.0")]
#[command(author = "QRCrypt Team")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Encrypt and generate QR code for a secret
    Encrypt {
        /// Output file path for the QR code
        #[arg(short, long)]
        output: PathBuf,

        /// Secret text to encrypt (if not provided, will prompt)
        #[arg(short, long)]
        secret: Option<String>,

        /// Read secret from file instead of input
        #[arg(short, long, conflicts_with = "secret")]
        file: Option<PathBuf>,

        /// QR code scale (pixels per module)
        #[arg(long, default_value = "8")]
        scale: u32,

        /// QR code border width (modules)
        #[arg(long, default_value = "4")]
        border: u32,

        /// Skip BIP39 word validation (allow invalid words)
        #[arg(long)]
        skip_word_check: bool,
    },

    /// Decrypt a QR code and display the secret
    Decrypt {
        /// JSON file containing encrypted data or path to read QR data
        #[arg(short, long)]
        input: Option<PathBuf>,

        /// JSON string containing encrypted data
        #[arg(short, long, conflicts_with = "input")]
        data: Option<String>,

        /// Scan QR code from webcam
        #[arg(long, conflicts_with_all = ["input", "data"])]
        scan_qr: bool,

        /// Save decrypted output to file
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Split secret using Shamir's Secret Sharing and generate QR codes
    Split {
        /// Number of shares required to reconstruct the secret (threshold)
        #[arg(short, long)]
        threshold: u8,

        /// Total number of shares to generate
        #[arg(short = 'n', long)]
        total: u8,

        /// Output directory for QR code files
        #[arg(short, long)]
        output_dir: PathBuf,

        /// Prefix for output filenames
        #[arg(long, default_value = "share")]
        prefix: String,

        /// Secret text to split (if not provided, will prompt)
        #[arg(short, long)]
        secret: Option<String>,

        /// Read secret from file instead of input
        #[arg(short, long, conflicts_with = "secret")]
        file: Option<PathBuf>,

        /// QR code scale (pixels per module)
        #[arg(long, default_value = "8")]
        scale: u32,

        /// QR code border width (modules)
        #[arg(long, default_value = "4")]
        border: u32,

        /// Generate info file with reconstruction instructions
        #[arg(long)]
        info: bool,

        /// Skip BIP39 word validation (allow invalid words)
        #[arg(long)]
        skip_word_check: bool,
    },

    /// Reconstruct secret from Shamir shares
    Reconstruct {
        /// Paths to share files (JSON format)
        #[arg(short, long)]
        shares: Vec<PathBuf>,

        /// JSON strings containing share data
        #[arg(short, long, conflicts_with = "shares")]
        data: Vec<String>,

        /// Scan QR codes from webcam (will scan until enough shares collected)
        #[arg(long, conflicts_with_all = ["shares", "data"])]
        scan_qr: bool,

        /// Maximum number of QR codes to scan (optional, will auto-stop when sufficient)
        #[arg(long, requires = "scan_qr")]
        max_scans: Option<u8>,

        /// Save reconstructed output to file
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Validate Shamir shares without reconstructing
    Validate {
        /// Paths to share files (JSON format)
        #[arg(short, long)]
        shares: Vec<PathBuf>,

        /// JSON strings containing share data
        #[arg(short, long, conflicts_with = "shares")]
        data: Vec<String>,

        /// Scan QR codes from webcam for validation
        #[arg(long, conflicts_with_all = ["shares", "data"])]
        scan_qr: bool,

        /// Number of QR codes to scan for validation
        #[arg(long, requires = "scan_qr")]
        count: Option<u8>,
    },

    /// Generate example seed phrase for testing
    Example {
        /// Type of example to generate
        #[arg(short, long, default_value = "bip39")]
        example_type: String,

        /// Number of words (12, 15, 18, 21, 24 for BIP39)
        #[arg(short, long, default_value = "12")]
        words: u8,
    },

    /// Validate a BIP39 seed phrase for typos and correctness
    ValidatePhrase {
        /// Seed phrase to validate (if not provided, will prompt)
        #[arg(short, long)]
        phrase: Option<String>,

        /// Read phrase from file instead of input
        #[arg(short, long, conflicts_with = "phrase")]
        file: Option<PathBuf>,

        /// Skip BIP39 checksum validation (only check words)
        #[arg(long)]
        skip_checksum: bool,
    },

    /// Create plausible deniability QR with decoy and hidden data
    DecoyEncrypt {
        /// Output file path for the layered QR code
        #[arg(short, long)]
        output: PathBuf,

        /// Real secret data to hide (if not provided, will prompt)
        #[arg(long)]
        real_secret: Option<String>,

        /// Read real secret from file instead of input
        #[arg(long, conflicts_with = "real_secret")]
        real_file: Option<PathBuf>,

        /// Decoy data (if not provided, will generate automatically)
        #[arg(long)]
        decoy_secret: Option<String>,

        /// Read decoy data from file
        #[arg(long, conflicts_with = "decoy_secret")]
        decoy_file: Option<PathBuf>,

        /// Type of decoy to generate: lowvalue, empty, random
        #[arg(long, default_value = "lowvalue")]
        decoy_type: String,

        /// Number of words for auto-generated decoy (12, 15, 18, 21, 24)
        #[arg(long, default_value = "12")]
        decoy_words: u8,

        /// QR code scale (pixels per module)
        #[arg(long, default_value = "8")]
        scale: u32,

        /// QR code border width (modules)
        #[arg(long, default_value = "4")]
        border: u32,

        /// Skip BIP39 word validation for real secret
        #[arg(long)]
        skip_word_check: bool,
    },
}

impl Cli {
    pub fn parse_args() -> Self {
        Self::parse()
    }

    pub fn validate(&self) -> Result<(), String> {
        match &self.command {
            Commands::Encrypt { scale, border, .. } => {
                if *scale == 0 {
                    return Err("Scale must be greater than 0".to_string());
                }
                if *border > 20 {
                    return Err("Border must be 20 or less".to_string());
                }
            }
            Commands::Split { threshold, total, scale, border, .. } => {
                if *threshold == 0 || *total == 0 {
                    return Err("Threshold and total must be greater than 0".to_string());
                }
                if *threshold > *total {
                    return Err("Threshold cannot be greater than total shares".to_string());
                }
                // Note: total is u8, so max value is already 255
                if *scale == 0 {
                    return Err("Scale must be greater than 0".to_string());
                }
                if *border > 20 {
                    return Err("Border must be 20 or less".to_string());
                }
            }
            Commands::Reconstruct { shares, data, scan_qr, .. } => {
                if !scan_qr && shares.is_empty() && data.is_empty() {
                    return Err("Must provide either share files, share data, or --scan-qr".to_string());
                }
            }
            Commands::Validate { shares, data, scan_qr, .. } => {
                if !scan_qr && shares.is_empty() && data.is_empty() {
                    return Err("Must provide either share files, share data, or --scan-qr".to_string());
                }
            }
            Commands::Example { words, .. } => {
                if ![12, 15, 18, 21, 24].contains(words) {
                    return Err("Word count must be 12, 15, 18, 21, or 24".to_string());
                }
            }
            _ => {}
        }
        Ok(())
    }
}