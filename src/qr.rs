use qrcode::{QrCode, EcLevel};
use image::{DynamicImage, ImageFormat};
use serde::{Deserialize, Serialize};
use std::path::Path;
use crate::error::{QRCryptError, Result};
use crate::crypto::EncryptedData;
use crate::shamir::ShamirShare;

#[derive(Serialize, Deserialize)]
pub struct QRData {
    pub data_type: QRDataType,
    pub content: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum QRDataType {
    EncryptedSecret,
    ShamirShare,
}

pub struct QRGenerator {
    error_correction: EcLevel,
    scale: u32,
    border: u32,
}

impl Default for QRGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl QRGenerator {
    pub fn new() -> Self {
        Self {
            error_correction: EcLevel::M, // Medium error correction
            scale: 8, // 8x8 pixels per module
            border: 4, // 4 module border
        }
    }

    pub fn with_settings(error_correction: EcLevel, scale: u32, border: u32) -> Self {
        Self {
            error_correction,
            scale,
            border,
        }
    }

    pub fn generate_encrypted_qr(&self, encrypted_data: &EncryptedData) -> Result<DynamicImage> {
        let qr_data = QRData {
            data_type: QRDataType::EncryptedSecret,
            content: serde_json::to_string(encrypted_data)?,
        };

        let json_data = serde_json::to_string(&qr_data)?;
        self.create_qr_image(&json_data)
    }

    pub fn generate_shamir_qr(&self, share: &ShamirShare) -> Result<DynamicImage> {
        let qr_data = QRData {
            data_type: QRDataType::ShamirShare,
            content: serde_json::to_string(share)?,
        };

        let json_data = serde_json::to_string(&qr_data)?;
        self.create_qr_image(&json_data)
    }

    pub fn generate_multiple_shamir_qrs(&self, shares: &[ShamirShare]) -> Result<Vec<DynamicImage>> {
        shares
            .iter()
            .map(|share| self.generate_shamir_qr(share))
            .collect()
    }

    fn create_qr_image(&self, data: &str) -> Result<DynamicImage> {
        let code = QrCode::with_error_correction_level(data, self.error_correction)
            .map_err(|e| QRCryptError::QRGeneration(format!("Failed to create QR code: {:?}", e)))?;

        // Use char as the pixel type which is supported by qrcode
        let string_image = code.render::<char>()
            .light_color(' ')
            .dark_color('█')
            .build();

        // Convert the string representation to a pixel-based image
        let lines: Vec<&str> = string_image.lines().collect();
        let height = lines.len() as u32;
        let width = if height > 0 { lines[0].len() as u32 } else { 0 };

        if width == 0 || height == 0 {
            return Err(QRCryptError::QRGeneration("Generated QR code has zero dimensions".to_string()));
        }

        let mut pixels = Vec::new();
        for line in &lines {
            for ch in line.chars() {
                pixels.push(if ch == ' ' { 255u8 } else { 0u8 }); // White for space, black for █
            }
        }

        // Scale the image
        let scaled_width = width * self.scale;
        let scaled_height = height * self.scale;
        let mut scaled_pixels = Vec::with_capacity((scaled_width * scaled_height) as usize);

        for y in 0..scaled_height {
            for x in 0..scaled_width {
                let original_x = x / self.scale;
                let original_y = y / self.scale;
                let original_index = (original_y * width + original_x) as usize;
                if original_index < pixels.len() {
                    scaled_pixels.push(pixels[original_index]);
                } else {
                    scaled_pixels.push(255u8);
                }
            }
        }

        let img_buffer = image::ImageBuffer::from_raw(scaled_width, scaled_height, scaled_pixels)
            .ok_or_else(|| QRCryptError::QRGeneration("Failed to create image buffer".to_string()))?;

        Ok(DynamicImage::ImageLuma8(img_buffer))
    }

    pub fn save_qr_image(&self, image: &DynamicImage, path: &Path) -> Result<()> {
        image.save_with_format(path, ImageFormat::Png)
            .map_err(|e| QRCryptError::QRGeneration(format!("Failed to save QR image: {}", e)))?;
        Ok(())
    }

    pub fn save_encrypted_qr(&self, encrypted_data: &EncryptedData, path: &Path) -> Result<()> {
        let image = self.generate_encrypted_qr(encrypted_data)?;
        self.save_qr_image(&image, path)
    }

    pub fn save_shamir_qrs(
        &self,
        shares: &[ShamirShare],
        base_path: &Path,
        prefix: &str,
    ) -> Result<Vec<String>> {
        let mut file_paths = Vec::new();
        
        for (i, share) in shares.iter().enumerate() {
            let filename = format!("{}_{}_of_{}_share_{}.png", 
                prefix, 
                i + 1, 
                shares.len(),
                share.share_id
            );
            let file_path = base_path.join(&filename);
            
            let image = self.generate_shamir_qr(share)?;
            self.save_qr_image(&image, &file_path)?;
            
            file_paths.push(filename);
        }
        
        Ok(file_paths)
    }

    pub fn generate_info_text(&self, shares: &[ShamirShare]) -> String {
        if shares.is_empty() {
            return "No shares provided".to_string();
        }

        let first_share = &shares[0];
        format!(
            "Shamir's Secret Sharing Configuration:\n\
            - Total shares: {}\n\
            - Threshold (minimum required): {}\n\
            - Shares generated: {}\n\
            - Version: {}\n\n\
            To reconstruct the original secret, you need at least {} out of {} shares.\n\
            Each QR code contains one share. Scan the required number of QR codes\n\
            and use the 'reconstruct' command to recover your original data.",
            first_share.total_shares,
            first_share.threshold,
            shares.len(),
            first_share.version,
            first_share.threshold,
            first_share.total_shares
        )
    }
}

pub struct QRReader;

impl QRReader {
    pub fn parse_qr_data(json_data: &str) -> Result<QRData> {
        serde_json::from_str(json_data)
            .map_err(|e| QRCryptError::QRParsing(format!("Failed to parse QR data: {}", e)))
    }

    pub fn parse_encrypted_data(qr_data: &QRData) -> Result<EncryptedData> {
        match qr_data.data_type {
            QRDataType::EncryptedSecret => {
                serde_json::from_str(&qr_data.content)
                    .map_err(|e| QRCryptError::QRParsing(format!("Failed to parse encrypted data: {}", e)))
            }
            _ => Err(QRCryptError::QRParsing("QR code does not contain encrypted data".to_string())),
        }
    }

    pub fn parse_shamir_share(qr_data: &QRData) -> Result<ShamirShare> {
        match qr_data.data_type {
            QRDataType::ShamirShare => {
                serde_json::from_str(&qr_data.content)
                    .map_err(|e| QRCryptError::QRParsing(format!("Failed to parse Shamir share: {}", e)))
            }
            _ => Err(QRCryptError::QRParsing("QR code does not contain Shamir share data".to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::{Crypto, SecretData};
    use crate::shamir::ShamirSecretSharing;

    #[test]
    fn test_qr_generation_encrypted() {
        let crypto = Crypto::new();
        let secret = SecretData::new("test secret phrase".to_string());
        let encrypted = crypto.encrypt(&secret, "password123").unwrap();
        
        let qr_gen = QRGenerator::new();
        let image = qr_gen.generate_encrypted_qr(&encrypted).unwrap();
        
        // Should be able to generate without error
        assert!(image.width() > 0);
        assert!(image.height() > 0);
    }

    #[test]
    fn test_qr_generation_shamir() {
        let secret = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let shares = ShamirSecretSharing::split_secret(secret, 3, 5).unwrap();
        
        let qr_gen = QRGenerator::new();
        let images = qr_gen.generate_multiple_shamir_qrs(&shares).unwrap();
        
        assert_eq!(images.len(), 5);
        for image in images {
            assert!(image.width() > 0);
            assert!(image.height() > 0);
        }
    }

    #[test]
    fn test_qr_data_parsing() {
        let secret = SecretData::new("test secret".to_string());
        let crypto = Crypto::new();
        let encrypted = crypto.encrypt(&secret, "password").unwrap();
        
        let qr_data = QRData {
            data_type: QRDataType::EncryptedSecret,
            content: serde_json::to_string(&encrypted).unwrap(),
        };
        
        let json = serde_json::to_string(&qr_data).unwrap();
        let parsed = QRReader::parse_qr_data(&json).unwrap();
        
        match parsed.data_type {
            QRDataType::EncryptedSecret => {
                let parsed_encrypted = QRReader::parse_encrypted_data(&parsed).unwrap();
                assert_eq!(parsed_encrypted.version, encrypted.version);
            }
            _ => panic!("Wrong data type"),
        }
    }
}