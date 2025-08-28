use sharks::{Share, Sharks};
use serde::{Deserialize, Serialize};
use base64::{Engine as _, engine::general_purpose};
use crate::error::{QRCryptError, Result};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ShamirShare {
    pub share_id: u8,
    pub threshold: u8,
    pub total_shares: u8,
    pub share_data: String,
    pub version: u8,
}

pub struct ShamirSecretSharing;

impl ShamirSecretSharing {
    pub fn split_secret(
        secret: &str,
        threshold: u8,
        total_shares: u8,
    ) -> Result<Vec<ShamirShare>> {
        if threshold == 0 || total_shares == 0 {
            return Err(QRCryptError::ShamirError("Threshold and total shares must be greater than 0".to_string()));
        }
        
        if threshold > total_shares {
            return Err(QRCryptError::ShamirError("Threshold cannot be greater than total shares".to_string()));
        }
        
        // Note: total_shares is u8, so max value is already 255
        let sharks = Sharks(threshold);
        let mut dealer = sharks.dealer(secret.as_bytes());
        
        let shares: Vec<Share> = (0..total_shares)
            .map(|_| dealer.next().expect("Failed to generate share"))
            .collect();
        
        let shamir_shares: Vec<ShamirShare> = shares
            .iter()
            .enumerate()
            .map(|(i, share)| {
                // Convert Share to Vec<u8> - Vec<u8> implements From<&Share>
                let share_vec: Vec<u8> = Vec::from(share);
                let encoded_share = general_purpose::STANDARD.encode(&share_vec);
                ShamirShare {
                    share_id: (i + 1) as u8,
                    threshold,
                    total_shares,
                    share_data: encoded_share,
                    version: 1,
                }
            })
            .collect();
        
        Ok(shamir_shares)
    }

    pub fn reconstruct_secret(shares: &[ShamirShare]) -> Result<String> {
        if shares.is_empty() {
            return Err(QRCryptError::ShamirError("No shares provided".to_string()));
        }

        // Validate all shares have the same parameters
        let first_share = &shares[0];
        for share in shares.iter().skip(1) {
            if share.threshold != first_share.threshold
                || share.total_shares != first_share.total_shares
                || share.version != first_share.version
            {
                return Err(QRCryptError::ShamirError("Inconsistent share parameters".to_string()));
            }
        }

        if first_share.version != 1 {
            return Err(QRCryptError::ShamirError(format!("Unsupported version: {}", first_share.version)));
        }

        if shares.len() < first_share.threshold as usize {
            return Err(QRCryptError::ShamirError(format!(
                "Insufficient shares: need {}, got {}",
                first_share.threshold,
                shares.len()
            )));
        }

        // Decode shares
        let decoded_shares: Result<Vec<Share>> = shares
            .iter()
            .map(|share| {
                let share_bytes = general_purpose::STANDARD.decode(&share.share_data)?;
                Share::try_from(share_bytes.as_slice())
                    .map_err(|e| QRCryptError::ShamirError(format!("Invalid share format: {:?}", e)))
            })
            .collect();

        let decoded_shares = decoded_shares?;

        // Reconstruct secret
        let sharks = Sharks(first_share.threshold);
        let secret_bytes = sharks
            .recover(&decoded_shares)
            .map_err(|e| QRCryptError::ShamirError(format!("Failed to reconstruct secret: {:?}", e)))?;

        let secret = String::from_utf8(secret_bytes)
            .map_err(|e| QRCryptError::ShamirError(format!("Invalid UTF-8 in reconstructed secret: {}", e)))?;

        Ok(secret)
    }

    pub fn validate_shares(shares: &[ShamirShare]) -> Result<()> {
        if shares.is_empty() {
            return Err(QRCryptError::ShamirError("No shares provided for validation".to_string()));
        }

        let first_share = &shares[0];
        
        for share in shares {
            if share.threshold != first_share.threshold
                || share.total_shares != first_share.total_shares
                || share.version != first_share.version
            {
                return Err(QRCryptError::ShamirError("Inconsistent share parameters".to_string()));
            }

            if share.share_id == 0 || share.share_id > share.total_shares {
                return Err(QRCryptError::ShamirError(format!(
                    "Invalid share ID: {} (should be 1-{})",
                    share.share_id,
                    share.total_shares
                )));
            }

            // Try to decode the share data
            general_purpose::STANDARD.decode(&share.share_data)
                .map_err(|_| QRCryptError::ShamirError("Invalid base64 encoding in share data".to_string()))?;
        }

        // Check for duplicate share IDs
        let mut share_ids: Vec<u8> = shares.iter().map(|s| s.share_id).collect();
        share_ids.sort_unstable();
        share_ids.dedup();
        
        if share_ids.len() != shares.len() {
            return Err(QRCryptError::ShamirError("Duplicate share IDs detected".to_string()));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_and_reconstruct() {
        let secret = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let threshold = 3;
        let total_shares = 5;

        let shares = ShamirSecretSharing::split_secret(secret, threshold, total_shares).unwrap();
        assert_eq!(shares.len(), total_shares as usize);

        // Test with minimum threshold
        let reconstructed = ShamirSecretSharing::reconstruct_secret(&shares[0..threshold as usize]).unwrap();
        assert_eq!(reconstructed, secret);

        // Test with more than threshold
        let reconstructed = ShamirSecretSharing::reconstruct_secret(&shares).unwrap();
        assert_eq!(reconstructed, secret);
    }

    #[test]
    fn test_insufficient_shares() {
        let secret = "test secret";
        let shares = ShamirSecretSharing::split_secret(secret, 3, 5).unwrap();

        // Try with insufficient shares
        let result = ShamirSecretSharing::reconstruct_secret(&shares[0..2]);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_shares() {
        let secret = "test secret";
        let shares = ShamirSecretSharing::split_secret(secret, 3, 5).unwrap();

        assert!(ShamirSecretSharing::validate_shares(&shares).is_ok());
        assert!(ShamirSecretSharing::validate_shares(&[]).is_err());
    }
}