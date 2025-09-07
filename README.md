# QRCrypt

A secure Rust CLI tool for storing crypto wallet seed phrases and other sensitive data in encrypted QR codes with support for Shamir's Secret Sharing.

## Features

- 🔐 **AES-256-GCM Encryption**: Military-grade encryption for your sensitive data
- 🔑 **Argon2 Key Derivation**: Secure password-based key derivation
- 📱 **QR Code Generation**: Convert encrypted data into scannable QR codes
- 📷 **QR Code Scanning**: Read QR codes from images or webcam (simulated)
- 🧩 **Shamir's Secret Sharing**: Split secrets into multiple shares (e.g., 3-of-5)
- 🤖 **Smart Reconstruction**: Automatically stops when enough shares are scanned
- 🗑️ **Memory Safety**: Secure memory handling with automatic clearing
- ⚡ **Open Source**: Fully auditable and transparent

## Installation

### 🚀 Quick Install (Recommended)

**Linux (static binary - works everywhere):**
```bash
curl -L https://github.com/vblimits/QRCrypt/releases/latest/download/qrcrypt-linux-x86_64-static.tar.gz | tar -xz
sudo mv qrcrypt-linux-x86_64-static /usr/local/bin/qrcrypt
chmod +x /usr/local/bin/qrcrypt
```

**Linux (dynamic binary):**
```bash
curl -L https://github.com/vblimits/QRCrypt/releases/latest/download/qrcrypt-linux-x86_64.tar.gz | tar -xz
sudo mv qrcrypt-linux-x86_64 /usr/local/bin/qrcrypt
chmod +x /usr/local/bin/qrcrypt
```

> 💡 **Auto-releases**: New versions are automatically released when CI/CD tests pass on the main branch!

### 🛠️ Build from Source

Make sure you have Rust installed, then build from source:

```bash
git clone <repository-url>
cd QRCrypt
cargo build --release
```

The binary will be available at `target/release/qrcrypt`.

### 🛡️ Recommended: Tails Installation (Best Practice)

For maximum security when handling cryptocurrency seed phrases, we **strongly recommend** using [Tails OS](https://tails.net/) - a security-focused, amnesic Linux distribution that leaves no traces after shutdown.

#### Why Use Tails?
- **Air-gapped security**: Disconnect from internet during use
- **Amnesic**: No traces left on the computer after shutdown
- **Pre-hardened**: Built-in security features and privacy protections
- **Offline operation**: Perfect for QRCrypt's offline-first design

#### Tails Installation Steps

1. **Download and verify Tails** from [tails.net](https://tails.net/)
2. **Boot Tails** on a dedicated computer (preferably offline)
3. **Install Rust toolchain**:
   ```bash
   sudo apt update
   sudo apt install rustc cargo build-essential git
   ```
4. **Get QRCrypt** (via USB drive or cloning):
   ```bash
   # If you have the source on USB
   cp -r /media/amnesia/USB/QRCrypt ~/
   
   # Or clone if you have internet (then disconnect)
   git clone <repository-url>
   ```
5. **Build QRCrypt**:
   ```bash
   cd QRCrypt
   cargo build --release
   ```
6. **Disconnect from internet** before handling real seed phrases

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
# From file
qrcrypt decrypt --input my_wallet.json

# Or scan QR code interactively
qrcrypt decrypt --scan-qr
```

### 5. Reconstruct Secret from Shares
```bash
# From files
qrcrypt reconstruct --shares ./shares/share_1_of_5_share_1.json ./shares/share_2_of_5_share_2.json ./shares/share_3_of_5_share_3.json

# Or scan QR codes interactively (stops automatically when enough collected)
qrcrypt reconstruct --scan-qr
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
      --scan-qr            Scan QR code from webcam/camera
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
  -s, --shares <SHARES>...     Paths to share files (JSON format)
  -d, --data <DATA>...         JSON strings containing share data
      --scan-qr                Scan QR codes from webcam (auto-stops when sufficient)
      --max-scans <MAX_SCANS>  Maximum QR codes to scan (optional)
  -o, --output <OUTPUT>        Save reconstructed output to file
```

### `validate`
Validate Shamir shares without reconstructing.

```bash
qrcrypt validate [OPTIONS]

Options:
  -s, --shares <SHARES>...  Paths to share files (JSON format)
  -d, --data <DATA>...      JSON strings containing share data
      --scan-qr             Scan QR codes from webcam for validation
      --count <COUNT>       Number of QR codes to scan (required with --scan-qr)
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

### Critical Security Practices
1. **Use Tails OS**: For maximum security, always use [Tails](https://tails.net/) when handling real cryptocurrency seeds
2. **Air-gapped Operation**: Disconnect from internet before processing real seed phrases
3. **Password Strength**: Use strong, unique passwords for encryption
4. **Secure Storage**: Store QR codes and shares in different secure locations
5. **Physical Security**: QR codes can be scanned - protect printed copies
6. **Test Recovery**: Always test decryption/reconstruction before relying on backups
7. **Multiple Backups**: Don't rely on a single backup method

### Tails-Specific Security Benefits
- **Amnesic**: All traces automatically deleted on shutdown
- **Pre-hardened**: Built-in security and privacy protections
- **Offline capable**: No network required for QRCrypt operations
- **Minimal attack surface**: Reduced risk of malware or keyloggers
- **Memory protection**: Enhanced protection against memory-based attacks

### Operational Security (OpSec)
- **Dedicated hardware**: Use a computer exclusively for seed management
- **Clean environment**: Ensure no cameras, recording devices, or observers
- **Multiple locations**: Store backup shares in geographically distributed locations
- **Physical destruction**: Securely destroy any temporary storage media
- **Verification**: Always verify reconstructed seeds in a test wallet first

## Example Workflows

### 🛡️ Secure Tails Workflow (Recommended)

#### Initial Setup in Tails
```bash
# Boot Tails, install dependencies, build QRCrypt (see installation above)
# IMPORTANT: Disconnect from internet before proceeding

# 1. Create secure workspace
mkdir ~/secure_backup
cd ~/secure_backup

# 2. Generate or input your real seed phrase
# (Type carefully - this is your actual wallet seed!)
echo "your actual 24 word seed phrase goes here..." > seed.txt

# 3. Create encrypted backup
~/QRCrypt/target/release/qrcrypt encrypt --file seed.txt --output wallet_backup.png

# 4. Create Shamir shares for distributed storage
~/QRCrypt/target/release/qrcrypt split --threshold 3 --total 5 --output-dir ./shares --file seed.txt --info

# 5. Securely delete original
shred -vfz -n 3 seed.txt

# 6. Copy results to multiple USB drives for different locations
# 7. Print QR codes on paper for offline storage
# 8. Shutdown Tails (all traces automatically deleted)
```

#### Recovery in Tails
```bash
# Boot Tails, build QRCrypt, disconnect internet
# Insert USB with backup files

# Option 1: Decrypt single encrypted backup
~/QRCrypt/target/release/qrcrypt decrypt --input /media/amnesia/USB/wallet_backup.json

# Option 2: Reconstruct from Shamir shares (need 3 of 5)
# From files
~/QRCrypt/target/release/qrcrypt reconstruct \
  --shares /media/amnesia/USB1/share_1_of_5_share_1.json \
  --shares /media/amnesia/USB2/share_2_of_5_share_2.json \
  --shares /media/amnesia/USB3/share_3_of_5_share_3.json

# Or scan printed QR codes (no threshold exposure!)
~/QRCrypt/target/release/qrcrypt reconstruct --scan-qr
```

### Standard Workflows

#### Simple Backup
```bash
# Generate and encrypt a seed phrase
qrcrypt example --words 24 > my_seed.txt
qrcrypt encrypt --file my_seed.txt --output backup.png
rm my_seed.txt  # Securely delete original

# Later, recover the seed
qrcrypt decrypt --input backup.json
```

#### Distributed Backup (3-of-5)
```bash
# Split seed into 5 shares, requiring 3 to recover
qrcrypt split --threshold 3 --total 5 --output-dir ./backup_shares --file my_seed.txt --info

# Distribute shares to different locations
# Later, recover with any 3 shares
qrcrypt reconstruct --shares ./backup_shares/share_1_of_5_share_1.json ./backup_shares/share_3_of_5_share_3.json ./backup_shares/share_5_of_5_share_5.json
```

## Contributing

This is an open-source project focused on defensive security. Contributions for security improvements, bug fixes, and feature enhancements are welcome.

### 🔄 Development Workflow

1. **Fork and clone** the repository
2. **Make changes** and test locally: `cargo test && cargo clippy`
3. **Submit pull request** - CI will automatically test your changes
4. **Auto-versioning** - When merged to main, patch version auto-increments (0.96.0 → 0.96.1)
5. **Auto-release** - New version is automatically released! 🚀

### 🤖 Automatic Version Management

**Every push to main:**
- ✅ Tests run automatically  
- 🔢 Patch version increments (0.96.0 → 0.96.1 → 0.96.2...)
- 🏷️ Git tag created automatically (v0.96.1, v0.96.2...)
- 📦 Release created with binaries
- 💾 Version and tag committed/pushed back to repo

**For major/minor versions:**
```bash
# When you need to bump major or minor versions
./scripts/bump-version.sh minor  # 0.96.x → 0.97.0
./scripts/bump-version.sh major  # 0.96.x → 1.0.0

git add Cargo.toml
git commit -m "Bump to new major/minor version"
git push origin main  # Subsequent pushes will auto-increment patch
```

**Version types:**
- `patch`: Auto-incremented on every main branch push (0.96.0 → 0.96.1)
- `minor`: Manual bump for new features (0.96.x → 0.97.0)  
- `major`: Manual bump for breaking changes (0.96.x → 1.0.0)

See [.github/RELEASES.md](.github/RELEASES.md) for detailed release documentation.

## License

MIT License - See LICENSE file for details.

## Disclaimer

⚠️ **This tool is provided as-is for educational and legitimate security purposes only. Always test thoroughly before using with real cryptocurrency seeds. The authors are not responsible for any loss of funds or data.**
