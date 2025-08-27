use std::fs;
use std::io::{self, Write};
use std::path::Path;
use rpassword;
use bip39::{Language, Mnemonic};
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
    
    // Validate against BIP39 word list
    validate_bip39_words(&words)?;
    
    Ok(())
}

pub fn validate_bip39_words(words: &[&str]) -> Result<()> {
    let wordlist = Language::English.word_list();
    let mut invalid_words = Vec::new();
    
    for &word in words {
        let word_lower = word.to_lowercase();
        if !wordlist.contains(&word_lower.as_str()) {
            // Try to find close matches for typo suggestions
            let suggestions = find_word_suggestions(&word_lower, wordlist);
            invalid_words.push((word.to_string(), suggestions));
        }
    }
    
    if !invalid_words.is_empty() {
        let mut error_msg = String::from("Invalid BIP39 words found:\n");
        
        for (invalid_word, suggestions) in &invalid_words {
            error_msg.push_str(&format!("  '{}' is not in the BIP39 word list", invalid_word));
            
            if !suggestions.is_empty() {
                error_msg.push_str(&format!("\n    Did you mean: {}?", suggestions.join(", ")));
            }
            error_msg.push('\n');
        }
        
        error_msg.push_str("\nPlease check your seed phrase for typos. Only words from the official BIP39 word list are valid.");
        return Err(QRCryptError::InvalidInput(error_msg));
    }
    
    Ok(())
}

pub fn validate_full_bip39_mnemonic(phrase: &str) -> Result<()> {
    // First do our basic validation
    validate_seed_phrase(phrase)?;
    
    // Then use the BIP39 library to validate the full mnemonic including checksum
    let phrase_normalized = phrase.split_whitespace().collect::<Vec<&str>>().join(" ");
    
    match Mnemonic::parse_in_normalized(Language::English, &phrase_normalized) {
        Ok(_) => {
            println!("✅ Valid BIP39 mnemonic with correct checksum");
            Ok(())
        }
        Err(e) => {
            // The words are valid but checksum might be wrong - give a warning but don't fail
            println!("⚠️  Warning: BIP39 checksum validation failed: {}", e);
            println!("    The words are valid, but this may not be a properly generated BIP39 mnemonic.");
            println!("    For security, ensure this mnemonic was generated by a trusted wallet.");
            Ok(()) // Don't fail - allow user to proceed if they want
        }
    }
}

fn find_word_suggestions(invalid_word: &str, wordlist: &[&str]) -> Vec<String> {
    let mut suggestions = Vec::new();
    
    // Look for words with similar length and characters
    for &word in wordlist {
        if should_suggest_word(invalid_word, word) {
            suggestions.push(word.to_string());
            if suggestions.len() >= 3 {
                break; // Limit to 3 suggestions
            }
        }
    }
    
    suggestions
}

fn should_suggest_word(invalid: &str, candidate: &str) -> bool {
    // Suggest if:
    // 1. Same length and 1-2 character difference
    // 2. Length differs by 1 and very similar
    // 3. Starts with same 2+ characters
    
    let invalid_len = invalid.len();
    let candidate_len = candidate.len();
    
    // Same length check
    if invalid_len == candidate_len {
        let diff_count = invalid.chars()
            .zip(candidate.chars())
            .filter(|(a, b)| a != b)
            .count();
        return diff_count <= 2;
    }
    
    // Length difference by 1
    if (invalid_len as i32 - candidate_len as i32).abs() == 1 {
        let edit_distance = calculate_edit_distance(invalid, candidate);
        return edit_distance <= 1;
    }
    
    // Starts with same prefix
    if invalid_len >= 3 && candidate_len >= 3 {
        let prefix_len = std::cmp::min(3, std::cmp::min(invalid_len, candidate_len));
        return invalid[..prefix_len] == candidate[..prefix_len];
    }
    
    false
}

fn calculate_edit_distance(s1: &str, s2: &str) -> usize {
    let s1_chars: Vec<char> = s1.chars().collect();
    let s2_chars: Vec<char> = s2.chars().collect();
    let len1 = s1_chars.len();
    let len2 = s2_chars.len();
    
    // Simple Levenshtein distance calculation
    let mut dp = vec![vec![0; len2 + 1]; len1 + 1];
    
    for i in 0..=len1 {
        dp[i][0] = i;
    }
    for j in 0..=len2 {
        dp[0][j] = j;
    }
    
    for i in 1..=len1 {
        for j in 1..=len2 {
            if s1_chars[i - 1] == s2_chars[j - 1] {
                dp[i][j] = dp[i - 1][j - 1];
            } else {
                dp[i][j] = 1 + std::cmp::min(
                    dp[i - 1][j],     // deletion
                    std::cmp::min(
                        dp[i][j - 1], // insertion
                        dp[i - 1][j - 1] // substitution
                    )
                );
            }
        }
    }
    
    dp[len1][len2]
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