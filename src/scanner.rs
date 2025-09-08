use crate::crypto::{EncryptedData, LayeredData};
use crate::error::{QRCryptError, Result};
use crate::qr::{QRData, QRReader};
use crate::shamir::ShamirShare;
use image::DynamicImage;
use rqrr::PreparedImage;
use std::io::{self, Write};

pub struct QRScanner;

impl QRScanner {
    /// Scan QR code from image file (for testing or file-based scanning)
    pub fn scan_from_image(img: &DynamicImage) -> Result<QRData> {
        let img = img.to_luma8();
        let prepared = PreparedImage::prepare(img);
        let grids = prepared.detect_grids();

        if grids.is_empty() {
            return Err(QRCryptError::QRParsing(
                "No QR code found in image".to_string(),
            ));
        }

        for grid in grids {
            if let Ok((_, content)) = grid.decode() {
                return QRReader::parse_qr_data(&content);
            }
        }

        Err(QRCryptError::QRParsing(
            "QR code detected but could not be decoded".to_string(),
        ))
    }

    /// Interactive QR code scanning - handles both camera and manual input
    pub fn interactive_scan() -> Result<QRData> {
        println!("🔍 QR Code Scanner");
        println!("Choose scanning method:");
        println!("1. Camera capture (requires camera support)");
        println!("2. Manual input (paste QR data)");
        println!("3. Image file");

        print!("Select option (1/2/3): ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        match input.trim() {
            "1" => {
                #[cfg(feature = "camera")]
                {
                    Self::capture_qr_from_camera()
                }
                #[cfg(not(feature = "camera"))]
                {
                    println!("❌ Camera support not enabled. Use manual input instead.");
                    Self::get_manual_input()
                }
            }
            "2" => Self::get_manual_input(),
            "3" => Self::scan_from_file(),
            _ => {
                println!("❌ Invalid option. Using manual input.");
                Self::get_manual_input()
            }
        }
    }

    /// Capture QR code using camera (simplified fallback)
    #[cfg(feature = "camera")]
    fn capture_qr_from_camera() -> Result<QRData> {
        println!("📷 Camera support is compiled in, but QR detection requires additional OpenCV modules.");
        println!("💡 For full camera QR detection, the build needs objdetect and videoio modules.");
        println!("🔄 Falling back to manual input for now...");
        Self::get_manual_input()
    }

    /// Fallback to manual input when camera fails
    fn get_manual_input() -> Result<QRData> {
        println!("📝 Manual QR Data Input");
        println!("Please paste the QR code JSON data:");

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input.is_empty() {
            return Err(QRCryptError::QRParsing("No input provided".to_string()));
        }

        QRReader::parse_qr_data(input)
    }

    /// Scan QR code from image file
    fn scan_from_file() -> Result<QRData> {
        println!("📁 Enter image file path:");
        let mut path = String::new();
        io::stdin().read_line(&mut path)?;
        let path = path.trim();

        let img = image::open(path)
            .map_err(|e| QRCryptError::QRParsing(format!("Failed to open image: {}", e)))?;

        Self::scan_from_image(&img)
    }

    /// Collect multiple shares for reconstruction
    pub fn collect_shares(max_shares: Option<usize>) -> Result<Vec<ShamirShare>> {
        let mut shares = Vec::new();
        let mut share_count = 0;
        let max = max_shares.unwrap_or(10);

        println!("📊 Collecting Shamir shares for reconstruction...");
        println!("💡 Shares will be automatically validated and deduplicated");

        loop {
            share_count += 1;
            println!("\n🔍 Scanning share {} of {}:", share_count, max);

            match Self::interactive_scan() {
                Ok(qr_data) => match qr_data {
                    QRData::ShamirShare(share) => {
                        // Check for duplicates
                        if shares.iter().any(|s: &ShamirShare| s.id == share.id) {
                            println!("⚠️  Duplicate share {} detected, skipping", share.id);
                            share_count -= 1;
                            continue;
                        }

                        println!(
                            "✅ Share {}/{} collected (ID: {})",
                            share.id, share.total_shares, share.id
                        );
                        shares.push(share.clone());

                        // Check if we have enough shares to reconstruct
                        if shares.len() >= share.threshold as usize {
                            println!(
                                "🎉 Collected {} of {} required shares - ready to reconstruct!",
                                shares.len(),
                                share.threshold
                            );
                            break;
                        }

                        println!(
                            "📊 Need {} more shares (minimum {} of {} required)",
                            share.threshold as usize - shares.len(),
                            share.threshold,
                            share.total_shares
                        );
                    }
                    QRData::Encrypted(_) => {
                        println!("❌ Expected Shamir share, got encrypted data");
                        share_count -= 1;
                    }
                    QRData::Layered(_) => {
                        println!("❌ Expected Shamir share, got layered data");
                        share_count -= 1;
                    }
                    QRData::Raw(_) => {
                        println!("❌ Expected Shamir share, got raw data");
                        share_count -= 1;
                    }
                },
                Err(e) => {
                    println!("❌ Failed to scan QR code: {}", e);
                    println!("🔄 Try again or press Ctrl+C to cancel");
                    share_count -= 1;
                }
            }

            if share_count >= max {
                println!("⚠️  Reached maximum share limit ({})", max);
                if shares.is_empty() {
                    return Err(QRCryptError::ShamirReconstruction(
                        "No valid shares collected".to_string(),
                    ));
                }
                break;
            }
        }

        if shares.is_empty() {
            return Err(QRCryptError::ShamirReconstruction(
                "No valid shares collected".to_string(),
            ));
        }

        // Validate that all shares are compatible
        let first_threshold = shares[0].threshold;
        let first_total = shares[0].total_shares;

        for share in &shares {
            if share.threshold != first_threshold || share.total_shares != first_total {
                return Err(QRCryptError::ShamirReconstruction(
                    "Incompatible shares detected - threshold/total mismatch".to_string(),
                ));
            }
        }

        Ok(shares)
    }
}
