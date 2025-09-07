use qrcode::{QrCode, EcLevel};
use image::{DynamicImage, ImageFormat, Rgba, RgbaImage};
use imageproc::drawing::draw_text_mut;
use rusttype::{Font, Scale};
use serde::{Deserialize, Serialize};
use std::path::Path;
use crate::error::{QRCryptError, Result};
use crate::crypto::{EncryptedData, LayeredData};
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
    LayeredSecret,
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

    #[allow(dead_code)]
    pub fn get_border(&self) -> u32 {
        self.border
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


    pub fn generate_layered_qr(&self, layered_data: &LayeredData) -> Result<DynamicImage> {
        let qr_data = QRData {
            data_type: QRDataType::LayeredSecret,
            content: serde_json::to_string(layered_data)?,
        };

        let json_data = serde_json::to_string(&qr_data)?;
        self.create_qr_image(&json_data)
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

    pub fn save_layered_qr(&self, layered_data: &LayeredData, path: &Path) -> Result<()> {
        let image = self.generate_layered_qr(layered_data)?;
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

    pub fn save_shamir_card_qrs(
        &self,
        shares: &[ShamirShare],
        base_path: &Path,
        prefix: &str,
        repo_url: &str,
        card_width_cm: f32,
        card_height_cm: f32,
    ) -> Result<Vec<String>> {
        let mut file_paths = Vec::new();
        
        for (i, share) in shares.iter().enumerate() {
            let filename = format!("{}_{}_of_{}_share_{}_card.png", 
                prefix, 
                i + 1, 
                shares.len(),
                share.share_id
            );
            let file_path = base_path.join(&filename);
            
            // Convert share to JSON for QR code content
            let share_json = serde_json::to_string(share)?;
            let qr_data = crate::qr::QRData {
                data_type: crate::qr::QRDataType::ShamirShare,
                content: share_json,
            };
            let json_data = serde_json::to_string(&qr_data)?;
            
            // Generate card QR with repository URL
            let image = self.generate_card_qr(&json_data, repo_url, card_width_cm, card_height_cm)?;
            self.save_qr_image(&image, &file_path)?;
            
            file_paths.push(filename);
        }
        
        Ok(file_paths)
    }

    pub fn generate_card_qr(&self, content: &str, repo_url: &str, card_width_cm: f32, card_height_cm: f32) -> Result<DynamicImage> {
        // Convert cm to pixels at 300 DPI
        let dpi = 300.0;
        let cm_to_inch = 2.54;
        let card_width_px = (card_width_cm * dpi / cm_to_inch) as u32;
        let card_height_px = (card_height_cm * dpi / cm_to_inch) as u32;
        
        // Generate QR code for the content
        let code = QrCode::with_error_correction_level(content, self.error_correction)
            .map_err(|e| QRCryptError::QRGeneration(format!("Failed to create QR code: {:?}", e)))?;

        // Create QR image
        let qr_img = code.render::<char>()
            .light_color(' ')
            .dark_color('█')
            .build();

        // Convert to pixel data
        let lines: Vec<&str> = qr_img.lines().collect();
        let qr_height = lines.len() as u32;
        let qr_width = if qr_height > 0 { lines[0].len() as u32 } else { 0 };

        if qr_width == 0 || qr_height == 0 {
            return Err(QRCryptError::QRGeneration("Generated QR code has zero dimensions".to_string()));
        }

        // Calculate QR size - make it the full height of the card minus small margins
        let margin = 10u32; // Small margin
        let available_height = card_height_px.saturating_sub(2 * margin);
        
        // QR code will be square and use full height
        let scale = available_height / qr_width;
        let final_qr_size = qr_width * scale;

        // Create QR pixel data
        let mut qr_pixels = Vec::new();
        for line in &lines {
            for ch in line.chars() {
                qr_pixels.push(if ch == ' ' { 255u8 } else { 0u8 });
            }
        }

        // Scale QR code
        let mut scaled_qr_pixels = Vec::with_capacity((final_qr_size * final_qr_size) as usize);
        for y in 0..final_qr_size {
            for x in 0..final_qr_size {
                let orig_x = x / scale;
                let orig_y = y / scale;
                let orig_index = (orig_y * qr_width + orig_x) as usize;
                if orig_index < qr_pixels.len() {
                    scaled_qr_pixels.push(qr_pixels[orig_index]);
                } else {
                    scaled_qr_pixels.push(255u8);
                }
            }
        }

        // Create card canvas
        let mut card_buffer = RgbaImage::new(card_width_px, card_height_px);
        
        // Fill with white background
        for pixel in card_buffer.pixels_mut() {
            *pixel = Rgba([255u8, 255u8, 255u8, 255u8]);
        }

        // Position QR code on card (left-justified, centered vertically)
        let qr_x = margin;
        let qr_y = (card_height_px - final_qr_size) / 2;

        // Copy QR code to card
        for y in 0..final_qr_size {
            for x in 0..final_qr_size {
                let qr_idx = (y * final_qr_size + x) as usize;
                if qr_idx < scaled_qr_pixels.len() {
                    let intensity = scaled_qr_pixels[qr_idx];
                    let card_x = qr_x + x;
                    let card_y = qr_y + y;
                    
                    if card_x < card_width_px && card_y < card_height_px {
                        card_buffer.put_pixel(card_x, card_y, Rgba([intensity, intensity, intensity, 255u8]));
                    }
                }
            }
        }

        // Add logo and URL text
        self.add_text_to_card(&mut card_buffer, repo_url, card_width_px, card_height_px, margin);
        
        Ok(DynamicImage::ImageRgba8(card_buffer))
    }

    fn add_text_to_card(&self, card_buffer: &mut RgbaImage, repo_url: &str, card_width: u32, card_height: u32, margin: u32) {
        use imageproc::drawing::draw_text_mut;
        use rusttype::{Font, Scale};
        
        // Use the new system font loading
        if let Some(font) = self.load_system_font() {
            eprintln!("Successfully loaded system font for main text rendering");
            let text_color = Rgba([0u8, 0u8, 0u8, 255u8]);
            
            // Calculate QR area to position text to the right of it
            let available_height = card_height.saturating_sub(2 * margin);
            let qr_size = available_height; // QR is square using full height
            let text_area_x = margin + qr_size + 20; // Start text 20px to the right of QR
            
            // "QRCrypt" at top of text area
            let logo_scale = Scale::uniform(72.0); // 72pt
            let logo_y = margin + 40;
            draw_text_mut(card_buffer, text_color, text_area_x as i32, logo_y as i32, logo_scale, &font, "QRCrypt");
            
            // URL at bottom, positioned closer to QR code right edge
            let url_scale = Scale::uniform(24.0);
            let url_y = card_height - margin - 40;
            // Move URL much closer to QR code - almost at its right edge
            let url_x = margin + qr_size + 5; // Just 5px from QR right edge instead of 20px
            
            draw_text_mut(card_buffer, text_color, url_x as i32, url_y as i32, url_scale, &font, repo_url);
        } else {
            eprintln!("No system font available");
        }
    }
    
    

    fn load_system_font(&self) -> Option<Font<'static>> {
        // Try common system font paths
        let font_paths = vec![
            // Linux fonts
            "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
            "/usr/share/fonts/TTF/DejaVuSans.ttf",
            "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
            "/usr/share/fonts/TTF/LiberationSans-Regular.ttf",
            "/usr/share/fonts/truetype/ubuntu/Ubuntu-R.ttf",
            "/usr/share/fonts/truetype/noto/NotoSans-Regular.ttf",
            
            // macOS fonts
            "/System/Library/Fonts/Helvetica.ttc",
            "/System/Library/Fonts/Arial.ttf",
            "/Library/Fonts/Arial.ttf",
            
            // Windows fonts
            "C:\\Windows\\Fonts\\arial.ttf",
            "C:\\Windows\\Fonts\\Arial.ttf",
            "C:\\Windows\\Fonts\\calibri.ttf",
            "C:\\Windows\\Fonts\\Calibri.ttf",
        ];
        
        for path in font_paths {
            if let Ok(font_data) = std::fs::read(path) {
                if let Some(font) = Font::try_from_vec(font_data) {
                    eprintln!("Loaded system font from: {}", path);
                    return Some(font);
                }
            }
        }
        
        eprintln!("No system fonts found, trying bundled font as fallback");
        // Fallback to bundled font if it exists
        if let Ok(font_data) = std::fs::read("assets/DejaVuSans.ttf") {
            if let Some(font) = Font::try_from_vec(font_data) {
                eprintln!("Loaded bundled font as fallback");
                return Some(font);
            }
        }
        
        None
    }


    pub fn save_card_qr(&self, content: &str, repo_url: &str, path: &Path, card_width_cm: f32, card_height_cm: f32) -> Result<()> {
        let image = self.generate_card_qr(content, repo_url, card_width_cm, card_height_cm)?;
        self.save_qr_image(&image, path)
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

    pub fn parse_layered_data(qr_data: &QRData) -> Result<LayeredData> {
        match qr_data.data_type {
            QRDataType::LayeredSecret => {
                serde_json::from_str(&qr_data.content)
                    .map_err(|e| QRCryptError::QRParsing(format!("Failed to parse layered data: {}", e)))
            }
            _ => Err(QRCryptError::QRParsing("QR code does not contain layered secret data".to_string())),
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
        let images: Vec<_> = shares.iter()
            .map(|share| qr_gen.generate_shamir_qr(share).unwrap())
            .collect();
        
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