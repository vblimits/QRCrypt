use crate::crypto::{EncryptedData, LayeredData};
use crate::error::{QRCryptError, Result};
use crate::qr::{QRData, QRReader};
use crate::shamir::ShamirShare;
use image::DynamicImage;
use rqrr::PreparedImage;
use std::io::{self, Write};

#[cfg(feature = "camera")]
use opencv::{
    core::{Mat, Vector},
    highgui, imgproc, objdetect,
    prelude::*,
    videoio,
};

pub struct QRScanner;

impl QRScanner {
    /// Scan QR code from image file (for testing or file-based scanning)
    pub fn scan_from_image(image_path: &str) -> Result<QRData> {
        let img = image::open(image_path)
            .map_err(|e| QRCryptError::QRParsing(format!("Failed to open image: {}", e)))?;

        Self::decode_qr_from_image(&img)
    }

    /// Scan QR code from webcam (interactive)
    pub fn scan_from_webcam(prompt: &str) -> Result<QRData> {
        println!("{}", prompt);

        #[cfg(feature = "camera")]
        {
            println!(
                "📷 Opening camera... Press 'q' to quit, 's' to capture when you see a QR code"
            );

            // Try to capture from webcam first
            match Self::capture_qr_from_camera() {
                Ok(qr_data) => return Ok(qr_data),
                Err(e) => {
                    println!("⚠️ Camera capture failed: {}", e);
                    println!("📁 Falling back to manual input mode...");
                }
            }
        }

        #[cfg(not(feature = "camera"))]
        {
            println!("📷 Camera support not enabled. Use --features camera when building to enable webcam support.");
            println!("📁 Using manual input mode...");
        }

        Self::fallback_manual_input()
    }

    /// Capture QR code using OpenCV camera
    #[cfg(feature = "camera")]
    fn capture_qr_from_camera() -> Result<QRData> {
        // Initialize camera
        let mut cam = videoio::VideoCapture::new(0, videoio::CAP_ANY)
            .map_err(|e| QRCryptError::QRParsing(format!("Failed to open camera: {}", e)))?;

        if !cam
            .is_opened()
            .map_err(|e| QRCryptError::QRParsing(format!("Camera error: {}", e)))?
        {
            return Err(QRCryptError::QRParsing("Camera not available".to_string()));
        }

        // Set camera properties for better QR detection
        let _ = cam.set(videoio::CAP_PROP_FRAME_WIDTH, 640.0);
        let _ = cam.set(videoio::CAP_PROP_FRAME_HEIGHT, 480.0);

        // Initialize QR detector
        let qr_detector = objdetect::QRCodeDetector::default()
            .map_err(|e| QRCryptError::QRParsing(format!("Failed to create QR detector: {}", e)))?;

        let mut frame = Mat::default();
        let mut qr_found = false;
        let mut qr_data = String::new();

        println!("🎥 Camera started. Hold up a QR code and press 's' to scan, 'q' to quit");

        loop {
            // Capture frame
            cam.read(&mut frame)
                .map_err(|e| QRCryptError::QRParsing(format!("Failed to read frame: {}", e)))?;

            if frame.empty() {
                continue;
            }

            // Detect QR codes in the frame
            let mut bbox = Vector::new();
            let mut rectified_image = Mat::default();

            match qr_detector.detect_and_decode(&frame, &mut bbox, &mut rectified_image) {
                Ok(decoded_text) => {
                    let decoded_string = String::from_utf8_lossy(&decoded_text).to_string();
                    if !decoded_string.is_empty() {
                        // Draw bounding box around detected QR code
                        if !bbox.is_empty() {
                            let color = opencv::core::Scalar::new(0.0, 255.0, 0.0, 0.0); // Green
                            for i in 0..bbox.len() {
                                let p1: opencv::core::Point2f = bbox.get(i).unwrap();
                                let p2: opencv::core::Point2f =
                                    bbox.get((i + 1) % bbox.len()).unwrap();
                                imgproc::line(
                                    &mut frame,
                                    opencv::core::Point::new(p1.x as i32, p1.y as i32),
                                    opencv::core::Point::new(p2.x as i32, p2.y as i32),
                                    color,
                                    2,
                                    imgproc::LINE_8,
                                    0,
                                )
                                .ok();
                            }
                        }

                        // Show detection status
                        let text = "QR Code detected! Press 's' to capture";
                        let font = imgproc::FONT_HERSHEY_SIMPLEX;
                        let color = opencv::core::Scalar::new(0.0, 255.0, 0.0, 0.0);
                        imgproc::put_text(
                            &mut frame,
                            text,
                            opencv::core::Point::new(10, 30),
                            font,
                            0.7,
                            color,
                            2,
                            imgproc::LINE_8,
                            false,
                        )
                        .ok();

                        qr_data = decoded_string;
                        qr_found = true;
                    }
                }
                Err(_) => {
                    // No QR code detected, continue
                }
            }

            // Show the frame
            highgui::imshow(
                "QRCrypt Scanner - Press 's' to capture, 'q' to quit",
                &frame,
            )
            .map_err(|e| QRCryptError::QRParsing(format!("Failed to display frame: {}", e)))?;

            // Check for key press
            let key = highgui::wait_key(1)
                .map_err(|e| QRCryptError::QRParsing(format!("Key input error: {}", e)))?;

            match key & 0xFF {
                113 => break, // 'q' key
                115 => {
                    // 's' key
                    if qr_found {
                        println!("✅ QR code captured successfully!");
                        break;
                    } else {
                        println!("❌ No QR code detected. Hold up a QR code and try again.");
                    }
                }
                27 => break, // ESC key
                _ => {}
            }
        }

        // Cleanup
        highgui::destroy_all_windows()
            .map_err(|e| QRCryptError::QRParsing(format!("Failed to close windows: {}", e)))?;

        if qr_found {
            // Parse the QR data
            QRReader::parse_qr_data(&qr_data)
        } else {
            Err(QRCryptError::QRParsing(
                "No QR code was captured".to_string(),
            ))
        }
    }

    /// Fallback to manual input when camera fails
    fn fallback_manual_input() -> Result<QRData> {
        println!("📝 Manual input mode:");
        println!("You can either:");
        println!("1. Paste the QR code JSON data directly");
        println!("2. Provide a file path to a QR code image");

        print!("Enter QR data as JSON or file path: ");
        io::stdout().flush().map_err(QRCryptError::Io)?;

        let mut qr_input = String::new();
        io::stdin()
            .read_line(&mut qr_input)
            .map_err(QRCryptError::Io)?;
        let qr_input = qr_input.trim();

        // Try to parse as JSON first
        if let Ok(qr_data) = QRReader::parse_qr_data(qr_input) {
            return Ok(qr_data);
        }

        // If not JSON, try as file path
        if std::path::Path::new(qr_input).exists() {
            return Self::scan_from_image(qr_input);
        }

        Err(QRCryptError::QRParsing(
            "Invalid QR data or file path".to_string(),
        ))
    }

    /// Scan multiple QR codes interactively for shares
    pub fn scan_shares_interactive(max_scans: Option<u8>) -> Result<Vec<ShamirShare>> {
        let mut shares = Vec::new();
        let mut scan_count = 0;
        let max_count = max_scans.unwrap_or(10); // Default max 10 scans

        println!("🔍 Starting QR code scanning for Shamir shares...");
        println!("📝 Note: Scanning will automatically stop when enough shares are collected");

        loop {
            scan_count += 1;

            println!("\n--- Scan #{} ---", scan_count);

            match Self::scan_from_webcam(&format!("Scanning QR code {}/{}", scan_count, max_count))
            {
                Ok(qr_data) => {
                    match QRReader::parse_shamir_share(&qr_data) {
                        Ok(share) => {
                            println!(
                                "✅ Successfully scanned share {} (ID: {})",
                                shares.len() + 1,
                                share.share_id
                            );

                            // Check for duplicates
                            if shares
                                .iter()
                                .any(|s: &ShamirShare| s.share_id == share.share_id)
                            {
                                println!("⚠️  Duplicate share detected - skipping");
                                continue;
                            }

                            shares.push(share);

                            // Check if we have enough shares (threshold is in the first share)
                            if let Some(first_share) = shares.first() {
                                if shares.len() >= first_share.threshold as usize {
                                    println!(
                                        "✅ Collected sufficient shares ({}/{}) - stopping scan",
                                        shares.len(),
                                        first_share.threshold
                                    );
                                    break;
                                } else {
                                    println!(
                                        "📊 Progress: {}/{} shares needed",
                                        shares.len(),
                                        first_share.threshold
                                    );
                                }
                            }
                        }
                        Err(e) => {
                            println!("❌ Failed to parse Shamir share: {}", e);
                            println!("💡 Make sure you're scanning a QR code generated by QRCrypt split command");
                        }
                    }
                }
                Err(e) => {
                    println!("❌ Failed to scan QR code: {}", e);
                }
            }

            // Check if we've reached max scans
            if scan_count >= max_count {
                println!("⚠️  Reached maximum scan limit ({})", max_count);
                break;
            }

            // Ask if user wants to continue
            if shares.is_empty()
                || (shares
                    .first()
                    .map(|s| shares.len() < s.threshold as usize)
                    .unwrap_or(true))
            {
                print!("Continue scanning? (Y/n): ");
                io::stdout().flush().map_err(QRCryptError::Io)?;

                let mut continue_input = String::new();
                io::stdin()
                    .read_line(&mut continue_input)
                    .map_err(QRCryptError::Io)?;

                if continue_input.trim().to_lowercase() == "n" {
                    break;
                }
            }
        }

        if shares.is_empty() {
            return Err(QRCryptError::QRParsing(
                "No valid shares were scanned".to_string(),
            ));
        }

        Ok(shares)
    }

    /// Scan a single encrypted QR code
    pub fn scan_encrypted_interactive() -> Result<EncryptedData> {
        println!("🔍 Scanning encrypted QR code...");

        let qr_data = Self::scan_from_webcam("Hold up your encrypted QR code")?;
        QRReader::parse_encrypted_data(&qr_data)
    }

    /// Scan a single layered QR code with plausible deniability
    pub fn scan_layered_interactive() -> Result<LayeredData> {
        println!("🕵️  Scanning layered QR code (plausible deniability)...");

        let qr_data = Self::scan_from_webcam("Hold up your layered QR code")?;
        QRReader::parse_layered_data(&qr_data)
    }

    /// Scan QR codes for validation (specified count)
    pub fn scan_for_validation(count: u8) -> Result<Vec<ShamirShare>> {
        let mut shares = Vec::new();

        println!("🔍 Scanning {} QR codes for validation...", count);

        for i in 1..=count {
            println!("\n--- Scan {}/{} ---", i, count);

            match Self::scan_from_webcam(&format!("Scanning QR code {}", i)) {
                Ok(qr_data) => match QRReader::parse_shamir_share(&qr_data) {
                    Ok(share) => {
                        println!(
                            "✅ Successfully scanned share {} (ID: {})",
                            i, share.share_id
                        );
                        shares.push(share);
                    }
                    Err(e) => {
                        println!("❌ Failed to parse Shamir share: {}", e);
                        return Err(e);
                    }
                },
                Err(e) => {
                    println!("❌ Failed to scan QR code: {}", e);
                    return Err(e);
                }
            }
        }

        Ok(shares)
    }

    fn decode_qr_from_image(img: &DynamicImage) -> Result<QRData> {
        // Convert to grayscale - this already gives us the right format for rqrr
        let gray_img = img.to_luma8();

        // Prepare image for rqrr directly
        let mut prepared = PreparedImage::prepare(gray_img);

        // Find QR codes
        let grids = prepared.detect_grids();
        if grids.is_empty() {
            return Err(QRCryptError::QRParsing(
                "No QR codes found in image".to_string(),
            ));
        }

        // Decode the first QR code found
        let grid = &grids[0];
        let (_meta, content) = grid
            .decode()
            .map_err(|e| QRCryptError::QRParsing(format!("Failed to decode QR code: {:?}", e)))?;

        // Parse the content as QR data
        QRReader::parse_qr_data(&content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::{Crypto, SecretData};
    use crate::qr::QRGenerator;
    use crate::shamir::ShamirSecretSharing;
    use std::fs;
    use std::path::Path;

    #[test]
    fn test_qr_round_trip_encrypted() {
        // Create test directory
        let test_dir = "test_qr_output";
        fs::create_dir_all(test_dir).unwrap();

        // Test data
        let secret_data = SecretData::new("test secret for QR round trip".to_string());
        let password = "test_password_123";

        // Encrypt the data
        let crypto = Crypto::new();
        let encrypted_data = crypto.encrypt(&secret_data, password).unwrap();

        // Generate QR code and save it
        let qr_generator = QRGenerator::new();
        let qr_image_path = format!("{}/test_encrypted.png", test_dir);
        qr_generator
            .save_encrypted_qr(&encrypted_data, Path::new(&qr_image_path))
            .unwrap();

        // Verify the image file was created
        assert!(Path::new(&qr_image_path).exists());

        // Load the QR code back from the image
        let loaded_qr_data = QRScanner::scan_from_image(&qr_image_path).unwrap();
        let loaded_encrypted = QRReader::parse_encrypted_data(&loaded_qr_data).unwrap();

        // Verify the data matches
        assert_eq!(encrypted_data.version, loaded_encrypted.version);
        assert_eq!(encrypted_data.nonce, loaded_encrypted.nonce);
        assert_eq!(encrypted_data.salt, loaded_encrypted.salt);
        assert_eq!(encrypted_data.ciphertext, loaded_encrypted.ciphertext);

        // Decrypt and verify the original secret
        let decrypted = crypto.decrypt(&loaded_encrypted, password).unwrap();
        assert_eq!(decrypted.data, "test secret for QR round trip");

        // Cleanup
        fs::remove_dir_all(test_dir).unwrap();
    }

    #[test]
    fn test_qr_round_trip_shamir_shares() {
        // Create test directory
        let test_dir = "test_qr_shamir";
        fs::create_dir_all(test_dir).unwrap();

        // Test data
        let secret =
            "abandon ability able about above absent absorb abstract absurd abuse access account";
        let threshold = 2;
        let total_shares = 3;

        // Create Shamir shares
        let shares = ShamirSecretSharing::split_secret(secret, threshold, total_shares).unwrap();
        assert_eq!(shares.len(), 3);

        let qr_generator = QRGenerator::new();
        let mut saved_paths = Vec::new();

        // Save each share as a QR code
        for (i, share) in shares.iter().enumerate() {
            let qr_image_path = format!("{}/share_{}.png", test_dir, i + 1);
            let qr_image = qr_generator.generate_shamir_qr(share).unwrap();
            qr_generator
                .save_qr_image(&qr_image, Path::new(&qr_image_path))
                .unwrap();

            // Verify the image file was created
            assert!(Path::new(&qr_image_path).exists());
            saved_paths.push(qr_image_path);
        }

        // Load the QR codes back from images
        let mut loaded_shares = Vec::new();
        for path in &saved_paths {
            let loaded_qr_data = QRScanner::scan_from_image(path).unwrap();
            let loaded_share = QRReader::parse_shamir_share(&loaded_qr_data).unwrap();
            loaded_shares.push(loaded_share);
        }

        // Verify we loaded the correct number of shares
        assert_eq!(loaded_shares.len(), 3);

        // Verify share properties match
        for (original, loaded) in shares.iter().zip(loaded_shares.iter()) {
            assert_eq!(original.share_id, loaded.share_id);
            assert_eq!(original.threshold, loaded.threshold);
            assert_eq!(original.total_shares, loaded.total_shares);
            assert_eq!(original.version, loaded.version);
            assert_eq!(original.share_data, loaded.share_data);
        }

        // Test reconstruction with minimum threshold (2 out of 3)
        let reconstructed = ShamirSecretSharing::reconstruct_secret(&loaded_shares[0..2]).unwrap();
        assert_eq!(reconstructed, secret);

        // Test reconstruction with all shares
        let reconstructed_all = ShamirSecretSharing::reconstruct_secret(&loaded_shares).unwrap();
        assert_eq!(reconstructed_all, secret);

        // Cleanup
        fs::remove_dir_all(test_dir).unwrap();
    }

    #[test]
    fn test_qr_validation_workflow() {
        // Create test directory
        let test_dir = "test_qr_validation";
        fs::create_dir_all(test_dir).unwrap();

        // Test data - use a proper seed phrase
        let secret = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let threshold = 3;
        let total_shares = 5;

        // Create Shamir shares
        let shares = ShamirSecretSharing::split_secret(secret, threshold, total_shares).unwrap();

        let qr_generator = QRGenerator::new();
        let mut qr_paths = Vec::new();

        // Save first 3 shares as QR codes (minimum needed for reconstruction)
        for (i, share) in shares.iter().take(3).enumerate() {
            let qr_image_path = format!("{}/validation_share_{}.png", test_dir, i + 1);
            let qr_image = qr_generator.generate_shamir_qr(share).unwrap();
            qr_generator
                .save_qr_image(&qr_image, Path::new(&qr_image_path))
                .unwrap();
            qr_paths.push(qr_image_path);
        }

        // Load and validate the shares
        let mut loaded_shares = Vec::new();
        for path in &qr_paths {
            let qr_data = QRScanner::scan_from_image(path).unwrap();
            let share = QRReader::parse_shamir_share(&qr_data).unwrap();
            loaded_shares.push(share);
        }

        // Validate the loaded shares
        ShamirSecretSharing::validate_shares(&loaded_shares).unwrap();

        // Verify we can reconstruct with these shares
        let reconstructed = ShamirSecretSharing::reconstruct_secret(&loaded_shares).unwrap();
        assert_eq!(reconstructed, secret);

        // Cleanup
        fs::remove_dir_all(test_dir).unwrap();
    }

    #[test]
    fn test_qr_error_handling() {
        // Test with non-existent file
        let result = QRScanner::scan_from_image("non_existent_file.png");
        assert!(result.is_err());

        // Test with invalid QR data (this would require a real invalid QR image)
        // For now, we'll test the error path when no QR codes are found
        // This is more of a placeholder for future testing with actual invalid QR images
    }
}
