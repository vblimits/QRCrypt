use crate::error::{QRCryptError, Result};
use base64::{engine::general_purpose, Engine as _};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use vsss_rs::{combine_shares, shamir::split_secret, Gf256};

type ShareData = (u8, Vec<u8>);
type DecodedSharesResult = Result<Vec<Vec<ShareData>>>;

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
    pub fn split_secret(secret: &str, threshold: u8, total_shares: u8) -> Result<Vec<ShamirShare>> {
        if threshold == 0 || total_shares == 0 {
            return Err(QRCryptError::ShamirError(
                "Threshold and total shares must be greater than 0".to_string(),
            ));
        }

        if threshold > total_shares {
            return Err(QRCryptError::ShamirError(
                "Threshold cannot be greater than total shares".to_string(),
            ));
        }

        // For vsss-rs, we need to split each byte separately
        let secret_bytes = secret.as_bytes();
        let mut all_shares = Vec::new();

        // Split each byte of the secret using Shamir's scheme
        let rng = OsRng;
        for (byte_index, &byte_value) in secret_bytes.iter().enumerate() {
            let gf_byte = Gf256::from(byte_value);
            let byte_shares: Vec<ShareData> =
                split_secret(threshold as usize, total_shares as usize, gf_byte, rng).map_err(
                    |e| {
                        QRCryptError::ShamirError(format!(
                            "Failed to split secret byte {}: {:?}",
                            byte_index, e
                        ))
                    },
                )?;
            all_shares.push(byte_shares);
        }

        // Reorganize shares by share number instead of byte position
        let mut shamir_shares = Vec::new();
        for share_idx in 0..total_shares {
            let mut share_data_for_share = Vec::new();
            for byte_shares in &all_shares {
                share_data_for_share.push(&byte_shares[share_idx as usize]);
            }

            let encoded_share = general_purpose::STANDARD
                .encode(bincode::serialize(&share_data_for_share).unwrap());
            shamir_shares.push(ShamirShare {
                share_id: share_idx + 1,
                threshold,
                total_shares,
                share_data: encoded_share,
                version: 1,
            });
        }

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
                return Err(QRCryptError::ShamirError(
                    "Inconsistent share parameters".to_string(),
                ));
            }
        }

        if first_share.version != 1 {
            return Err(QRCryptError::ShamirError(format!(
                "Unsupported version: {}",
                first_share.version
            )));
        }

        if shares.len() < first_share.threshold as usize {
            return Err(QRCryptError::ShamirError(format!(
                "Insufficient shares: need {}, got {}",
                first_share.threshold,
                shares.len()
            )));
        }

        // Decode shares - each share contains all byte shares for that participant
        let decoded_shares: DecodedSharesResult = shares
            .iter()
            .map(|share| {
                let share_bytes = general_purpose::STANDARD
                    .decode(&share.share_data)
                    .map_err(|e| {
                        QRCryptError::ShamirError(format!("Invalid base64 in share: {}", e))
                    })?;
                bincode::deserialize(&share_bytes).map_err(|e| {
                    QRCryptError::ShamirError(format!("Invalid share format: {:?}", e))
                })
            })
            .collect();

        let decoded_shares = decoded_shares?;

        // Ensure all shares have the same length (same number of bytes)
        if decoded_shares.is_empty() {
            return Err(QRCryptError::ShamirError("No decoded shares".to_string()));
        }

        let byte_count = decoded_shares[0].len();
        for share in &decoded_shares {
            if share.len() != byte_count {
                return Err(QRCryptError::ShamirError(
                    "Shares have inconsistent lengths".to_string(),
                ));
            }
        }

        // Reconstruct each byte separately
        let mut secret_bytes = Vec::new();
        for byte_idx in 0..byte_count {
            // Collect the shares for this byte position
            let byte_shares: Vec<&ShareData> = decoded_shares
                .iter()
                .map(|share| &share[byte_idx])
                .collect();

            // Reconstruct this byte
            let reconstructed_byte: Gf256 = combine_shares(
                byte_shares
                    .iter()
                    .map(|s| (*s).clone())
                    .collect::<Vec<_>>()
                    .as_slice(),
            )
            .map_err(|e| {
                QRCryptError::ShamirError(format!(
                    "Failed to reconstruct byte {}: {:?}",
                    byte_idx, e
                ))
            })?;

            secret_bytes.push(u8::from(reconstructed_byte));
        }

        let secret = String::from_utf8(secret_bytes).map_err(|e| {
            QRCryptError::ShamirError(format!("Invalid UTF-8 in reconstructed secret: {}", e))
        })?;

        Ok(secret)
    }

    pub fn validate_shares(shares: &[ShamirShare]) -> Result<()> {
        if shares.is_empty() {
            return Err(QRCryptError::ShamirError(
                "No shares provided for validation".to_string(),
            ));
        }

        let first_share = &shares[0];

        for share in shares {
            if share.threshold != first_share.threshold
                || share.total_shares != first_share.total_shares
                || share.version != first_share.version
            {
                return Err(QRCryptError::ShamirError(
                    "Inconsistent share parameters".to_string(),
                ));
            }

            if share.share_id == 0 || share.share_id > share.total_shares {
                return Err(QRCryptError::ShamirError(format!(
                    "Invalid share ID: {} (should be 1-{})",
                    share.share_id, share.total_shares
                )));
            }

            // Try to decode the share data
            general_purpose::STANDARD
                .decode(&share.share_data)
                .map_err(|_| {
                    QRCryptError::ShamirError("Invalid base64 encoding in share data".to_string())
                })?;
        }

        // Check for duplicate share IDs
        let mut share_ids: Vec<u8> = shares.iter().map(|s| s.share_id).collect();
        share_ids.sort_unstable();
        share_ids.dedup();

        if share_ids.len() != shares.len() {
            return Err(QRCryptError::ShamirError(
                "Duplicate share IDs detected".to_string(),
            ));
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
        let reconstructed =
            ShamirSecretSharing::reconstruct_secret(&shares[0..threshold as usize]).unwrap();
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
