use std::fs;
use std::io::{self, Write};
use std::path::Path;
use rpassword;
use crate::error::{QRCryptError, Result};
use crate::shamir::ShamirShare;
use crate::crypto::EncryptedData;
use crate::qr::QRReader;

pub fn read_password(prompt: &str) -> Result<String> {
    print!("{}", prompt);
    io::stdout().flush().map_err(QRCryptError::Io)?;
    
    rpassword::read_password()
        .map_err(|e| QRCryptError::InvalidInput(format!("Failed to read password: {}", e)))
}

pub fn read_secret_input(prompt: &str) -> Result<String> {
    print!("{}", prompt);
    io::stdout().flush().map_err(QRCryptError::Io)?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input).map_err(QRCryptError::Io)?;
    
    Ok(input.trim().to_string())
}

pub fn read_file_content(path: &Path) -> Result<String> {
    fs::read_to_string(path)
        .map_err(|e| QRCryptError::Io(e))
        .map(|content| content.trim().to_string())
}

pub fn write_file_content(path: &Path, content: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(QRCryptError::Io)?;
    }
    
    fs::write(path, content).map_err(QRCryptError::Io)
}

pub fn load_shares_from_files(file_paths: &[impl AsRef<Path>]) -> Result<Vec<ShamirShare>> {
    let mut shares = Vec::new();
    
    for path in file_paths {
        let content = read_file_content(path.as_ref())?;
        
        // Try to parse as QRData first (from QR code scan), then as direct ShamirShare
        let share = if let Ok(qr_data) = QRReader::parse_qr_data(&content) {
            QRReader::parse_shamir_share(&qr_data)?
        } else {
            serde_json::from_str::<ShamirShare>(&content)
                .map_err(|e| QRCryptError::InvalidInput(format!("Invalid share file format: {}", e)))?
        };
        
        shares.push(share);
    }
    
    Ok(shares)
}

pub fn load_shares_from_data(data_strings: &[String]) -> Result<Vec<ShamirShare>> {
    let mut shares = Vec::new();
    
    for data in data_strings {
        // Try to parse as QRData first, then as direct ShamirShare
        let share = if let Ok(qr_data) = QRReader::parse_qr_data(data) {
            QRReader::parse_shamir_share(&qr_data)?
        } else {
            serde_json::from_str::<ShamirShare>(data)
                .map_err(|e| QRCryptError::InvalidInput(format!("Invalid share data format: {}", e)))?
        };
        
        shares.push(share);
    }
    
    Ok(shares)
}

pub fn load_encrypted_from_file(path: &Path) -> Result<EncryptedData> {
    let content = read_file_content(path)?;
    
    // Try to parse as QRData first, then as direct EncryptedData
    if let Ok(qr_data) = QRReader::parse_qr_data(&content) {
        QRReader::parse_encrypted_data(&qr_data)
    } else {
        serde_json::from_str::<EncryptedData>(&content)
            .map_err(|e| QRCryptError::InvalidInput(format!("Invalid encrypted data format: {}", e)))
    }
}

pub fn load_encrypted_from_data(data: &str) -> Result<EncryptedData> {
    // Try to parse as QRData first, then as direct EncryptedData
    if let Ok(qr_data) = QRReader::parse_qr_data(data) {
        QRReader::parse_encrypted_data(&qr_data)
    } else {
        serde_json::from_str::<EncryptedData>(data)
            .map_err(|e| QRCryptError::InvalidInput(format!("Invalid encrypted data format: {}", e)))
    }
}

pub fn validate_seed_phrase(phrase: &str) -> Result<()> {
    let words: Vec<&str> = phrase.split_whitespace().collect();
    
    if words.is_empty() {
        return Err(QRCryptError::InvalidInput("Seed phrase cannot be empty".to_string()));
    }
    
    // Basic validation for common seed phrase lengths
    let word_count = words.len();
    if ![12, 15, 18, 21, 24].contains(&word_count) {
        println!("Warning: Seed phrase has {} words. Common lengths are 12, 15, 18, 21, or 24 words.", word_count);
    }
    
    // Check for very short words that might indicate input errors
    for word in &words {
        if word.len() < 2 {
            return Err(QRCryptError::InvalidInput(format!("Word '{}' seems too short. Please check your input.", word)));
        }
    }
    
    Ok(())
}

pub fn generate_example_seed(word_count: u8) -> Result<String> {
    // Generate a deterministic but realistic-looking seed phrase for testing
    let words = [
        "abandon", "ability", "able", "about", "above", "absent", "absorb", "abstract",
        "absurd", "abuse", "access", "accident", "account", "accuse", "achieve", "acid",
        "acoustic", "acquire", "across", "act", "action", "actor", "actress", "actual",
        "adapt", "add", "addict", "address", "adjust", "admit", "adult", "advance",
        "advice", "aerobic", "affair", "afford", "afraid", "again", "agent", "agree",
        "ahead", "aim", "air", "airport", "aisle", "alarm", "album", "alcohol"
    ];
    
    if ![12, 15, 18, 21, 24].contains(&word_count) {
        return Err(QRCryptError::InvalidInput("Word count must be 12, 15, 18, 21, or 24".to_string()));
    }
    
    let selected_words: Vec<&str> = words.iter()
        .take(word_count as usize - 1)
        .copied()
        .collect();
    
    let mut result = selected_words.join(" ");
    result.push_str(" about"); // Always end with "about" for consistency
    
    Ok(result)
}

pub fn print_success(message: &str) {
    println!("✅ {}", message);
}

pub fn print_warning(message: &str) {
    println!("⚠️  {}", message);
}

pub fn print_error(message: &str) {
    eprintln!("❌ {}", message);
}

pub fn print_info(message: &str) {
    println!("ℹ️  {}", message);
}

pub fn confirm_action(prompt: &str) -> Result<bool> {
    print!("{} (y/N): ", prompt);
    io::stdout().flush().map_err(QRCryptError::Io)?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input).map_err(QRCryptError::Io)?;
    
    let input = input.trim().to_lowercase();
    Ok(input == "y" || input == "yes")
}