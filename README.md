# QRCrypt

A secure Rust CLI tool for storing crypto wallet seed phrases and other sensitive data in encrypted QR codes with support for Shamir's Secret Sharing.

## Features

- 🔐 **AES-256-GCM Encryption**: Military-grade encryption for your sensitive data
- 🔑 **Argon2 Key Derivation**: Secure password-based key derivation
- 📱 **QR Code Generation**: Convert encrypted data into scannable QR codes
- 🧩 **Shamir's Secret Sharing**: Split secrets into multiple shares (e.g., 3-of-5)
- 🗑️ **Memory Safety**: Secure memory handling with automatic clearing
- ⚡ **Open Source**: Fully auditable and transparent

## Installation

Make sure you have Rust installed, then build from source:

```bash
git clone <repository-url>
cd QRCrypt
cargo build --release
```

The binary will be available at `target/release/qrcrypt`.

## Quick Start

### 1. Generate Example Seed Phrase
```bash
qrcrypt example --words 12
```

### 2. Encrypt and Generate QR Code
```bash
qrcrypt encrypt --output my_wallet.png --secret "abandon ability able about above absent absorb abstract absurd abuse access account"
```

### 3. Split Secret using Shamir's Secret Sharing
```bash
qrcrypt split --threshold 3 --total 5 --output-dir ./shares --secret "abandon ability able about above absent absorb abstract absurd abuse access account"
```

### 4. Decrypt QR Code
```bash
qrcrypt decrypt --input my_wallet.json
```

### 5. Reconstruct Secret from Shares
```bash
qrcrypt reconstruct --shares ./shares/share_1_of_5_share_1.json ./shares/share_2_of_5_share_2.json ./shares/share_3_of_5_share_3.json
```

## Commands

### `encrypt`
Encrypt a secret and generate a QR code.

```bash
qrcrypt encrypt [OPTIONS] --output <OUTPUT>

Options:
  -o, --output <OUTPUT>    Output file path for the QR code
  -s, --secret <SECRET>    Secret text to encrypt (if not provided, will prompt)
  -f, --file <FILE>        Read secret from file instead of input
      --scale <SCALE>      QR code scale (pixels per module) [default: 8]
      --border <BORDER>    QR code border width (modules) [default: 4]
```

### `decrypt`
Decrypt a QR code and display the secret.

```bash
qrcrypt decrypt [OPTIONS]

Options:
  -i, --input <INPUT>      JSON file containing encrypted data
  -d, --data <DATA>        JSON string containing encrypted data
  -o, --output <OUTPUT>    Save decrypted output to file
```

### `split`
Split secret using Shamir's Secret Sharing and generate QR codes.

```bash
qrcrypt split [OPTIONS] --threshold <THRESHOLD> --total <TOTAL> --output-dir <OUTPUT_DIR>

Options:
  -t, --threshold <THRESHOLD>    Number of shares required to reconstruct (threshold)
  -n, --total <TOTAL>           Total number of shares to generate
  -o, --output-dir <OUTPUT_DIR> Output directory for QR code files
      --prefix <PREFIX>         Prefix for output filenames [default: share]
  -s, --secret <SECRET>         Secret text to split
  -f, --file <FILE>            Read secret from file instead of input
      --scale <SCALE>          QR code scale [default: 8]
      --border <BORDER>        QR code border width [default: 4]
      --info                   Generate info file with reconstruction instructions
```

### `reconstruct`
Reconstruct secret from Shamir shares.

```bash
qrcrypt reconstruct [OPTIONS]

Options:
  -s, --shares <SHARES>...  Paths to share files (JSON format)
  -d, --data <DATA>...      JSON strings containing share data
  -o, --output <OUTPUT>     Save reconstructed output to file
```

### `validate`
Validate Shamir shares without reconstructing.

```bash
qrcrypt validate [OPTIONS]

Options:
  -s, --shares <SHARES>...  Paths to share files (JSON format)
  -d, --data <DATA>...      JSON strings containing share data
```

### `example`
Generate example seed phrase for testing.

```bash
qrcrypt example [OPTIONS]

Options:
  -e, --example-type <TYPE>  Type of example to generate [default: bip39]
  -w, --words <WORDS>        Number of words (12, 15, 18, 21, 24) [default: 12]
```

## Security Features

- **AES-256-GCM**: Authenticated encryption ensuring both confidentiality and integrity
- **Argon2**: Memory-hard password hashing resistant to brute-force attacks
- **Secure Memory**: Automatic clearing of sensitive data from memory
- **No Network**: Fully offline operation for maximum security
- **Open Source**: Complete transparency for security auditing

## Use Cases

1. **Backup Crypto Wallets**: Store seed phrases securely as QR codes
2. **Distributed Storage**: Use Shamir sharing to distribute across multiple locations
3. **Air-Gapped Security**: Generate and store secrets on offline systems
4. **Recovery Planning**: Create redundant backups with flexible threshold requirements

## Output Files

### Single Encryption
- `output.png`: QR code image
- `output.json`: JSON data file (for programmatic access)

### Shamir Sharing
- `share_1_of_5_share_1.png`: QR code for share 1
- `share_1_of_5_share_1.json`: JSON data for share 1
- `info.txt`: Instructions for reconstruction (if `--info` flag used)

## Security Considerations

⚠️ **Important Security Notes:**

1. **Password Strength**: Use strong, unique passwords for encryption
2. **Secure Storage**: Store QR codes and shares in different secure locations
3. **Physical Security**: QR codes can be scanned - protect printed copies
4. **Test Recovery**: Always test decryption/reconstruction before relying on backups
5. **Multiple Backups**: Don't rely on a single backup method

## Example Workflows

### Simple Backup
```bash
# Generate and encrypt a seed phrase
qrcrypt example --words 24 > my_seed.txt
qrcrypt encrypt --file my_seed.txt --output backup.png
rm my_seed.txt  # Securely delete original

# Later, recover the seed
qrcrypt decrypt --input backup.json
```

### Distributed Backup (3-of-5)
```bash
# Split seed into 5 shares, requiring 3 to recover
qrcrypt split --threshold 3 --total 5 --output-dir ./backup_shares --file my_seed.txt --info

# Distribute shares to different locations
# Later, recover with any 3 shares
qrcrypt reconstruct --shares ./backup_shares/share_1_of_5_share_1.json ./backup_shares/share_3_of_5_share_3.json ./backup_shares/share_5_of_5_share_5.json
```

## Contributing

This is an open-source project focused on defensive security. Contributions for security improvements, bug fixes, and feature enhancements are welcome.

## License

MIT License - See LICENSE file for details.

## Disclaimer

⚠️ **This tool is provided as-is for educational and legitimate security purposes only. Always test thoroughly before using with real cryptocurrency seeds. The authors are not responsible for any loss of funds or data.**
