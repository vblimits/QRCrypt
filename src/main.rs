mod error;
mod crypto;
mod shamir;
mod qr;
mod cli;
mod utils;

use std::fs;
use qrcode::EcLevel;

use crate::cli::{Cli, Commands};
use crate::crypto::{Crypto, SecretData};
use crate::shamir::ShamirSecretSharing;
use crate::qr::QRGenerator;
use crate::error::Result;
use crate::utils::*;

fn main() {
    let cli = Cli::parse_args();
    
    if let Err(e) = cli.validate() {
        print_error(&format!("Invalid arguments: {}", e));
        std::process::exit(1);
    }
    
    if let Err(e) = run_command(cli.command) {
        print_error(&format!("Error: {}", e));
        std::process::exit(1);
    }
}

fn run_command(command: Commands) -> Result<()> {
    match command {
        Commands::Encrypt { output, secret, file, scale, border } => {
            handle_encrypt(output, secret, file, scale, border)
        }
        Commands::Decrypt { input, data, output } => {
            handle_decrypt(input, data, output)
        }
        Commands::Split { threshold, total, output_dir, prefix, secret, file, scale, border, info } => {
            handle_split(threshold, total, output_dir, prefix, secret, file, scale, border, info)
        }
        Commands::Reconstruct { shares, data, output } => {
            handle_reconstruct(shares, data, output)
        }
        Commands::Validate { shares, data } => {
            handle_validate(shares, data)
        }
        Commands::Example { example_type, words } => {
            handle_example(example_type, words)
        }
    }
}

fn handle_encrypt(
    output: std::path::PathBuf,
    secret: Option<String>,
    file: Option<std::path::PathBuf>,
    scale: u32,
    border: u32,
) -> Result<()> {
    // Get secret data
    let secret_text = if let Some(file_path) = file {
        read_file_content(&file_path)?
    } else if let Some(text) = secret {
        text
    } else {
        read_secret_input("Enter secret to encrypt: ")?
    };

    if secret_text.trim().is_empty() {
        return Err(crate::error::QRCryptError::InvalidInput("Secret cannot be empty".to_string()));
    }

    // Validate if it looks like a seed phrase
    if let Err(_) = validate_seed_phrase(&secret_text) {
        print_warning("Input doesn't appear to be a valid seed phrase format");
    }

    // Get password
    let password = read_password("Enter encryption password: ")?;
    if password.trim().is_empty() {
        return Err(crate::error::QRCryptError::InvalidInput("Password cannot be empty".to_string()));
    }

    // Encrypt the data
    let crypto = Crypto::new();
    let secret_data = SecretData::new(secret_text);
    let encrypted_data = crypto.encrypt(&secret_data, &password)?;

    // Generate QR code
    let error_correction = if encrypted_data.ciphertext.len() > 1000 {
        EcLevel::L // Low error correction for larger data
    } else {
        EcLevel::M // Medium error correction
    };
    
    let qr_generator = QRGenerator::with_settings(error_correction, scale, border);
    qr_generator.save_encrypted_qr(&encrypted_data, &output)?;

    print_success(&format!("Encrypted QR code saved to: {}", output.display()));
    
    // Optionally save JSON data as well
    let json_output = output.with_extension("json");
    write_file_content(&json_output, &serde_json::to_string_pretty(&encrypted_data)?)?;
    print_info(&format!("JSON data also saved to: {}", json_output.display()));

    Ok(())
}

fn handle_decrypt(
    input: Option<std::path::PathBuf>,
    data: Option<String>,
    output: Option<std::path::PathBuf>,
) -> Result<()> {
    // Load encrypted data
    let encrypted_data = if let Some(input_path) = input {
        load_encrypted_from_file(&input_path)?
    } else if let Some(data_str) = data {
        load_encrypted_from_data(&data_str)?
    } else {
        return Err(crate::error::QRCryptError::InvalidInput("Must provide either input file or data string".to_string()));
    };

    // Get password
    let password = read_password("Enter decryption password: ")?;

    // Decrypt the data
    let crypto = Crypto::new();
    let decrypted_data = crypto.decrypt(&encrypted_data, &password)?;

    // Output the result
    if let Some(output_path) = output {
        write_file_content(&output_path, &decrypted_data.data)?;
        print_success(&format!("Decrypted data saved to: {}", output_path.display()));
    } else {
        println!("\nDecrypted secret:");
        println!("{}", decrypted_data.data);
    }

    Ok(())
}

fn handle_split(
    threshold: u8,
    total: u8,
    output_dir: std::path::PathBuf,
    prefix: String,
    secret: Option<String>,
    file: Option<std::path::PathBuf>,
    scale: u32,
    border: u32,
    info: bool,
) -> Result<()> {
    // Get secret data
    let secret_text = if let Some(file_path) = file {
        read_file_content(&file_path)?
    } else if let Some(text) = secret {
        text
    } else {
        read_secret_input("Enter secret to split: ")?
    };

    if secret_text.trim().is_empty() {
        return Err(crate::error::QRCryptError::InvalidInput("Secret cannot be empty".to_string()));
    }

    // Validate if it looks like a seed phrase
    validate_seed_phrase(&secret_text)?;

    // Create output directory
    fs::create_dir_all(&output_dir)?;

    // Split the secret using Shamir's Secret Sharing
    let shares = ShamirSecretSharing::split_secret(&secret_text, threshold, total)?;

    // Generate QR codes
    let error_correction = if secret_text.len() > 1000 {
        EcLevel::L // Low error correction for larger data
    } else {
        EcLevel::M // Medium error correction
    };
    
    let qr_generator = QRGenerator::with_settings(error_correction, scale, border);
    let file_names = qr_generator.save_shamir_qrs(&shares, &output_dir, &prefix)?;

    print_success(&format!("Generated {} share QR codes in: {}", shares.len(), output_dir.display()));
    
    for file_name in &file_names {
        print_info(&format!("- {}", file_name));
    }

    // Save JSON shares as well
    for (i, share) in shares.iter().enumerate() {
        let json_filename = format!("{}_{}_of_{}_share_{}.json", 
            prefix, i + 1, shares.len(), share.share_id);
        let json_path = output_dir.join(&json_filename);
        write_file_content(&json_path, &serde_json::to_string_pretty(share)?)?;
    }

    // Generate info file if requested
    if info {
        let info_content = qr_generator.generate_info_text(&shares);
        let info_path = output_dir.join(format!("{}_info.txt", prefix));
        write_file_content(&info_path, &info_content)?;
        print_info(&format!("Info file saved: {}", info_path.display()));
    }

    println!("\n{}", qr_generator.generate_info_text(&shares));

    Ok(())
}

fn handle_reconstruct(
    shares: Vec<std::path::PathBuf>,
    data: Vec<String>,
    output: Option<std::path::PathBuf>,
) -> Result<()> {
    // Load shares
    let share_data = if !shares.is_empty() {
        load_shares_from_files(&shares)?
    } else if !data.is_empty() {
        load_shares_from_data(&data)?
    } else {
        return Err(crate::error::QRCryptError::InvalidInput("No share data provided".to_string()));
    };

    // Validate shares
    ShamirSecretSharing::validate_shares(&share_data)?;
    print_info(&format!("Loaded {} valid shares", share_data.len()));

    // Reconstruct the secret
    let reconstructed_secret = ShamirSecretSharing::reconstruct_secret(&share_data)?;

    // Output the result
    if let Some(output_path) = output {
        write_file_content(&output_path, &reconstructed_secret)?;
        print_success(&format!("Reconstructed secret saved to: {}", output_path.display()));
    } else {
        println!("\nReconstructed secret:");
        println!("{}", reconstructed_secret);
    }

    Ok(())
}

fn handle_validate(
    shares: Vec<std::path::PathBuf>,
    data: Vec<String>,
) -> Result<()> {
    // Load shares
    let share_data = if !shares.is_empty() {
        load_shares_from_files(&shares)?
    } else if !data.is_empty() {
        load_shares_from_data(&data)?
    } else {
        return Err(crate::error::QRCryptError::InvalidInput("No share data provided".to_string()));
    };

    // Validate shares
    ShamirSecretSharing::validate_shares(&share_data)?;

    if !share_data.is_empty() {
        let first_share = &share_data[0];
        print_success(&format!("✅ All {} shares are valid!", share_data.len()));
        print_info(&format!(
            "Threshold: {} of {} shares required for reconstruction",
            first_share.threshold,
            first_share.total_shares
        ));
        
        if share_data.len() >= first_share.threshold as usize {
            print_success("✅ You have enough shares to reconstruct the secret!");
        } else {
            print_warning(&format!(
                "⚠️  You need {} more shares to reconstruct the secret",
                first_share.threshold as usize - share_data.len()
            ));
        }
    }

    Ok(())
}

fn handle_example(example_type: String, words: u8) -> Result<()> {
    match example_type.as_str() {
        "bip39" | "seed" | "mnemonic" => {
            let example_seed = generate_example_seed(words)?;
            println!("Example {}-word seed phrase:", words);
            println!("{}", example_seed);
            print_warning("⚠️  This is for testing only! Never use example phrases for real wallets.");
        }
        _ => {
            return Err(crate::error::QRCryptError::InvalidInput(format!("Unknown example type: {}", example_type)));
        }
    }
    
    Ok(())
}