mod cli;
mod crypto;
mod error;
mod qr;
mod scanner;
mod shamir;
mod utils;

use qrcode::EcLevel;
use std::fs;

use crate::cli::{Cli, Commands};
use crate::crypto::{Crypto, SecretData};
use crate::error::Result;
use crate::qr::QRGenerator;
use crate::scanner::QRScanner;
use crate::shamir::ShamirSecretSharing;
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
        Commands::Encrypt {
            output,
            secret,
            file,
            scale,
            border,
            skip_word_check,
            card,
            card_width,
            card_height,
        } => handle_encrypt(
            output,
            secret,
            file,
            scale,
            border,
            skip_word_check,
            card,
            card_width,
            card_height,
        ),
        Commands::Decrypt {
            input,
            data,
            scan_qr,
            output,
        } => handle_decrypt(input, data, scan_qr, output),
        Commands::Split {
            threshold,
            total,
            output_dir,
            prefix,
            secret,
            file,
            scale,
            border,
            info,
            skip_word_check,
            card,
            card_width,
            card_height,
        } => handle_split(
            threshold,
            total,
            output_dir,
            prefix,
            secret,
            file,
            scale,
            border,
            info,
            skip_word_check,
            card,
            card_width,
            card_height,
        ),
        Commands::Reconstruct {
            shares,
            data,
            scan_qr,
            max_scans,
            output,
        } => handle_reconstruct(shares, data, scan_qr, max_scans, output),
        Commands::Validate {
            shares,
            data,
            scan_qr,
            count,
        } => handle_validate(shares, data, scan_qr, count),
        Commands::Example {
            example_type,
            words,
        } => handle_example(example_type, words),
        Commands::ValidatePhrase {
            phrase,
            file,
            skip_checksum,
        } => handle_validate_phrase(phrase, file, skip_checksum),
        Commands::DecoyEncrypt {
            output,
            real_secret,
            real_file,
            decoy_secret,
            decoy_file,
            decoy_type,
            decoy_words,
            scale,
            border,
            skip_word_check,
        } => handle_decoy_encrypt(
            output,
            real_secret,
            real_file,
            decoy_secret,
            decoy_file,
            decoy_type,
            decoy_words,
            scale,
            border,
            skip_word_check,
        ),
    }
}

#[allow(clippy::too_many_arguments)]
fn handle_encrypt(
    output: std::path::PathBuf,
    secret: Option<String>,
    file: Option<std::path::PathBuf>,
    scale: u32,
    border: u32,
    skip_word_check: bool,
    card: bool,
    card_width: f32,
    card_height: f32,
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
        return Err(crate::error::QRCryptError::InvalidInput(
            "Secret cannot be empty".to_string(),
        ));
    }

    // Validate if it looks like a seed phrase (unless bypassed)
    if !skip_word_check {
        if let Err(e) = validate_seed_phrase(&secret_text) {
            print_warning(&format!("Seed phrase validation warning: {}", e));
            if !confirm_action("Continue anyway")? {
                return Err(crate::error::QRCryptError::InvalidInput(
                    "Seed phrase validation failed".to_string(),
                ));
            }
        }
    } else {
        print_warning("⚠️  Skipping BIP39 word validation (--skip-word-check flag used)");
    }

    // Get password
    let password = read_password("Enter encryption password: ")?;
    if password.trim().is_empty() {
        return Err(crate::error::QRCryptError::InvalidInput(
            "Password cannot be empty".to_string(),
        ));
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

    if card {
        // Generate card-optimized QR code with repository URL
        let json_data = serde_json::to_string(&encrypted_data)?;
        let repo_url = "https://github.com/vblimits/QRCrypt.git";
        qr_generator.save_card_qr(&json_data, repo_url, &output, card_width, card_height)?;
        print_success(&format!("Card QR code saved to: {}", output.display()));
        print_info(&format!(
            "Card dimensions: {:.1}cm x {:.1}cm",
            card_width, card_height
        ));
        print_info("Optimized for 300 DPI printing on stainless steel cards");
    } else {
        qr_generator.save_encrypted_qr(&encrypted_data, &output)?;
        print_success(&format!("Encrypted QR code saved to: {}", output.display()));
    }

    // Optionally save JSON data as well
    let json_output = output.with_extension("json");
    write_file_content(
        &json_output,
        &serde_json::to_string_pretty(&encrypted_data)?,
    )?;
    print_info(&format!(
        "JSON data also saved to: {}",
        json_output.display()
    ));

    Ok(())
}

fn handle_decrypt(
    input: Option<std::path::PathBuf>,
    data: Option<String>,
    scan_qr: bool,
    output: Option<std::path::PathBuf>,
) -> Result<()> {
    // First try to detect if we have layered data
    let is_layered = if let Some(ref input_path) = input {
        // Try to load as layered data first
        load_layered_from_file(input_path).is_ok()
    } else if let Some(ref data_str) = data {
        load_layered_from_data(data_str).is_ok()
    } else {
        false // For scan_qr, we'll detect dynamically
    };

    if is_layered || scan_qr {
        // Handle layered decryption
        let layered_data = if scan_qr {
            // For scanning, we need to detect the QR type dynamically
            // For now, try to scan as layered first
            match QRScanner::scan_layered_interactive() {
                Ok(data) => data,
                Err(_) => {
                    // Fall back to regular encrypted scan
                    print_info("QR doesn't contain layered data, trying regular decryption...");
                    return handle_regular_decrypt(input, data, scan_qr, output);
                }
            }
        } else if let Some(input_path) = input {
            load_layered_from_file(&input_path)?
        } else if let Some(data_str) = data {
            load_layered_from_data(&data_str)?
        } else {
            return Err(crate::error::QRCryptError::InvalidInput(
                "Must provide either input file, data string, or --scan-qr".to_string(),
            ));
        };

        // Get password
        let password = read_password("Enter password (will try both decoy and real layers): ")?;

        // Try to decrypt layered data
        let crypto = Crypto::new();
        match crypto.decrypt_layered(&layered_data, &password) {
            Ok((decrypted_data, is_real)) => {
                if is_real {
                    print_success("🔓 Successfully decrypted REAL secret data!");
                } else {
                    print_warning("🎭 Successfully decrypted DECOY data.");
                    if let Some(hint) = &layered_data.decoy_hint {
                        print_info(&format!("Decoy hint: {}", hint));
                    }
                }

                // Output the result
                if let Some(output_path) = output {
                    write_file_content(&output_path, &decrypted_data.data)?;
                    print_success(&format!(
                        "Decrypted data saved to: {}",
                        output_path.display()
                    ));
                } else {
                    println!("\nDecrypted secret:");
                    println!("{}", decrypted_data.data);
                }
            }
            Err(_) => {
                print_error("❌ Password doesn't match any layer (neither decoy nor real data)");
                return Err(crate::error::QRCryptError::Decryption(
                    "Invalid password for layered data".to_string(),
                ));
            }
        }
    } else {
        // Handle regular decryption
        return handle_regular_decrypt(input, data, scan_qr, output);
    }

    Ok(())
}

fn handle_regular_decrypt(
    input: Option<std::path::PathBuf>,
    data: Option<String>,
    scan_qr: bool,
    output: Option<std::path::PathBuf>,
) -> Result<()> {
    // Load encrypted data
    let encrypted_data = if scan_qr {
        QRScanner::scan_encrypted_interactive()?
    } else if let Some(input_path) = input {
        load_encrypted_from_file(&input_path)?
    } else if let Some(data_str) = data {
        load_encrypted_from_data(&data_str)?
    } else {
        return Err(crate::error::QRCryptError::InvalidInput(
            "Must provide either input file, data string, or --scan-qr".to_string(),
        ));
    };

    // Get password
    let password = read_password("Enter decryption password: ")?;

    // Decrypt the data
    let crypto = Crypto::new();
    let decrypted_data = crypto.decrypt(&encrypted_data, &password)?;

    // Output the result
    if let Some(output_path) = output {
        write_file_content(&output_path, &decrypted_data.data)?;
        print_success(&format!(
            "Decrypted data saved to: {}",
            output_path.display()
        ));
    } else {
        println!("\nDecrypted secret:");
        println!("{}", decrypted_data.data);
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
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
    skip_word_check: bool,
    card: bool,
    card_width: f32,
    card_height: f32,
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
        return Err(crate::error::QRCryptError::InvalidInput(
            "Secret cannot be empty".to_string(),
        ));
    }

    // Validate seed phrase including BIP39 words (unless bypassed)
    if !skip_word_check {
        if let Err(e) = validate_seed_phrase(&secret_text) {
            print_error(&format!("Seed phrase validation failed: {}", e));
            return Err(e);
        }
    } else {
        print_warning("⚠️  Skipping BIP39 word validation (--skip-word-check flag used)");
        print_warning("⚠️  This may result in unrecoverable shares if words contain typos!");
    }

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

    let file_names = if card {
        // Generate card-optimized QR codes with repository URL
        let repo_url = "https://github.com/vblimits/QRCrypt.git";
        let card_files = qr_generator.save_shamir_card_qrs(
            &shares,
            &output_dir,
            &prefix,
            repo_url,
            card_width,
            card_height,
        )?;
        print_success(&format!(
            "Generated {} card-optimized share QR codes in: {}",
            shares.len(),
            output_dir.display()
        ));
        print_info(&format!(
            "Card dimensions: {:.1}cm x {:.1}cm",
            card_width, card_height
        ));
        print_info("Optimized for 300 DPI printing on stainless steel cards");
        card_files
    } else {
        let regular_files = qr_generator.save_shamir_qrs(&shares, &output_dir, &prefix)?;
        print_success(&format!(
            "Generated {} share QR codes in: {}",
            shares.len(),
            output_dir.display()
        ));
        regular_files
    };

    for file_name in &file_names {
        print_info(&format!("- {}", file_name));
    }

    // Save JSON shares as well
    for (i, share) in shares.iter().enumerate() {
        let json_filename = format!(
            "{}_{}_of_{}_share_{}.json",
            prefix,
            i + 1,
            shares.len(),
            share.share_id
        );
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
    scan_qr: bool,
    max_scans: Option<u8>,
    output: Option<std::path::PathBuf>,
) -> Result<()> {
    // Load shares
    let share_data = if scan_qr {
        QRScanner::scan_shares_interactive(max_scans)?
    } else if !shares.is_empty() {
        load_shares_from_files(&shares)?
    } else if !data.is_empty() {
        load_shares_from_data(&data)?
    } else {
        return Err(crate::error::QRCryptError::InvalidInput(
            "No share data provided".to_string(),
        ));
    };

    // Validate shares
    ShamirSecretSharing::validate_shares(&share_data)?;
    print_info(&format!("Loaded {} valid shares", share_data.len()));

    // Reconstruct the secret
    let reconstructed_secret = ShamirSecretSharing::reconstruct_secret(&share_data)?;

    // Output the result
    if let Some(output_path) = output {
        write_file_content(&output_path, &reconstructed_secret)?;
        print_success(&format!(
            "Reconstructed secret saved to: {}",
            output_path.display()
        ));
    } else {
        println!("\nReconstructed secret:");
        println!("{}", reconstructed_secret);
    }

    Ok(())
}

fn handle_validate(
    shares: Vec<std::path::PathBuf>,
    data: Vec<String>,
    scan_qr: bool,
    count: Option<u8>,
) -> Result<()> {
    // Load shares
    let share_data = if scan_qr {
        let scan_count = count.unwrap_or(5); // Default to 5 if not specified
        QRScanner::scan_for_validation(scan_count)?
    } else if !shares.is_empty() {
        load_shares_from_files(&shares)?
    } else if !data.is_empty() {
        load_shares_from_data(&data)?
    } else {
        return Err(crate::error::QRCryptError::InvalidInput(
            "No share data provided".to_string(),
        ));
    };

    // Validate shares
    ShamirSecretSharing::validate_shares(&share_data)?;

    if !share_data.is_empty() {
        let first_share = &share_data[0];
        print_success(&format!("✅ All {} shares are valid!", share_data.len()));
        print_info(&format!(
            "Threshold: {} of {} shares required for reconstruction",
            first_share.threshold, first_share.total_shares
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
            print_warning(
                "⚠️  This is for testing only! Never use example phrases for real wallets.",
            );
        }
        _ => {
            return Err(crate::error::QRCryptError::InvalidInput(format!(
                "Unknown example type: {}",
                example_type
            )));
        }
    }

    Ok(())
}

fn handle_validate_phrase(
    phrase: Option<String>,
    file: Option<std::path::PathBuf>,
    skip_checksum: bool,
) -> Result<()> {
    // Get the phrase to validate
    let phrase_text = if let Some(file_path) = file {
        read_file_content(&file_path)?
    } else if let Some(text) = phrase {
        text
    } else {
        read_secret_input("Enter seed phrase to validate: ")?
    };

    if phrase_text.trim().is_empty() {
        return Err(crate::error::QRCryptError::InvalidInput(
            "Phrase cannot be empty".to_string(),
        ));
    }

    println!("🔍 Validating seed phrase...\n");

    // Always validate words against BIP39 list
    if let Err(e) = validate_seed_phrase(&phrase_text) {
        print_error("❌ Seed phrase validation failed:");
        println!("{}", e);
        return Err(e);
    }

    print_success("✅ All words are valid BIP39 words!");

    // Optionally validate full BIP39 mnemonic with checksum
    if !skip_checksum {
        println!("\n🔐 Checking BIP39 mnemonic validity (including checksum)...");
        validate_full_bip39_mnemonic(&phrase_text)?;
    } else {
        println!("⚠️  Skipping BIP39 checksum validation (--skip-checksum flag used)");
    }

    let word_count = phrase_text.split_whitespace().count();
    println!("\n📊 Seed phrase summary:");
    println!("  • Word count: {}", word_count);
    println!("  • All words valid: ✅");
    if !skip_checksum {
        println!("  • BIP39 checksum: ✅ (if no warnings above)");
    }

    print_success("✅ Seed phrase validation complete!");

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn handle_decoy_encrypt(
    output: std::path::PathBuf,
    real_secret: Option<String>,
    real_file: Option<std::path::PathBuf>,
    decoy_secret: Option<String>,
    decoy_file: Option<std::path::PathBuf>,
    decoy_type: String,
    decoy_words: u8,
    scale: u32,
    border: u32,
    skip_word_check: bool,
) -> Result<()> {
    println!("🕵️  Creating plausible deniability QR code...\n");

    // Get real secret data
    let real_secret_text = if let Some(file_path) = real_file {
        read_file_content(&file_path)?
    } else if let Some(text) = real_secret {
        text
    } else {
        read_secret_input("Enter your REAL secret to hide: ")?
    };

    if real_secret_text.trim().is_empty() {
        return Err(crate::error::QRCryptError::InvalidInput(
            "Real secret cannot be empty".to_string(),
        ));
    }

    // Validate real secret (unless bypassed)
    if !skip_word_check {
        if let Err(e) = validate_seed_phrase(&real_secret_text) {
            print_warning(&format!("Real seed phrase validation warning: {}", e));
            if !confirm_action("Continue anyway")? {
                return Err(crate::error::QRCryptError::InvalidInput(
                    "Real seed phrase validation failed".to_string(),
                ));
            }
        }
    }

    // Get decoy secret data
    let decoy_secret_text = if let Some(file_path) = decoy_file {
        read_file_content(&file_path)?
    } else if let Some(text) = decoy_secret {
        text
    } else {
        print_info(&format!(
            "Generating {} decoy seed phrase with {} words...",
            decoy_type, decoy_words
        ));
        generate_decoy_seed(decoy_words, &decoy_type)?
    };

    if decoy_secret_text.trim().is_empty() {
        return Err(crate::error::QRCryptError::InvalidInput(
            "Decoy secret cannot be empty".to_string(),
        ));
    }

    print_info(&format!("Decoy seed phrase: {}", decoy_secret_text));

    // Get passwords
    let real_password = read_password("Enter STRONG password for your real secret: ")?;
    if real_password.trim().is_empty() {
        return Err(crate::error::QRCryptError::InvalidInput(
            "Real password cannot be empty".to_string(),
        ));
    }

    let decoy_password =
        read_password("Enter WEAK password for decoy (should be easy to guess/crack): ")?;
    if decoy_password.trim().is_empty() {
        return Err(crate::error::QRCryptError::InvalidInput(
            "Decoy password cannot be empty".to_string(),
        ));
    }

    // Create layered encryption
    let crypto = Crypto::new();
    let real_data = SecretData::new(real_secret_text);
    let decoy_data = SecretData::new(decoy_secret_text.clone());
    let decoy_hint = generate_decoy_hint(&decoy_type);

    let layered_data = crypto.encrypt_with_decoy(
        &real_data,
        &real_password,
        &decoy_data,
        &decoy_password,
        decoy_hint,
    )?;

    // Generate QR code with appropriate error correction
    let total_data_size = serde_json::to_string(&layered_data)?.len();
    let error_correction = if total_data_size > 1500 {
        EcLevel::L // Low error correction for large layered data
    } else {
        EcLevel::M // Medium error correction
    };

    let qr_generator = QRGenerator::with_settings(error_correction, scale, border);
    qr_generator.save_layered_qr(&layered_data, &output)?;

    print_success(&format!(
        "🕵️  Plausible deniability QR code saved to: {}",
        output.display()
    ));

    // Save JSON data as well
    let json_output = output.with_extension("json");
    write_file_content(&json_output, &serde_json::to_string_pretty(&layered_data)?)?;
    print_info(&format!(
        "JSON data also saved to: {}",
        json_output.display()
    ));

    // Show security info
    println!("\n🔒 Security Information:");
    print_warning("DECOY PASSWORD (weak, meant to be crackable):");
    println!("   Password: {}", decoy_password);
    if let Some(hint) = &layered_data.decoy_hint {
        println!("   Appears to be: {}", hint);
    }
    println!(
        "   Will reveal: \"{}...\"",
        decoy_secret_text
            .split_whitespace()
            .take(3)
            .collect::<Vec<_>>()
            .join(" ")
    );

    print_success("REAL PASSWORD (keep secret):");
    println!("   Password: [HIDDEN]");
    println!("   Contains your actual valuable secrets");

    print_warning("\n⚠️  OPSEC Tips:");
    println!("   • Give attackers the weak decoy password under coercion");
    println!("   • Make the decoy password easy to guess (dictionary word, etc.)");
    println!("   • The decoy should look like a real but low-value wallet");
    println!("   • Never reveal or write down your strong real password");
    println!("   • Consider using this with Tails OS for maximum security");

    Ok(())
}
