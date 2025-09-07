# QRCrypt - Windows Release

## 📦 Release Information

- **Version**: 0.1.0
- **Platform**: Windows 64-bit (x86_64)
- **Architecture**: PE32+ executable
- **Compiler**: MinGW-w64 GCC cross-compiler
- **Target**: x86_64-pc-windows-gnu

## 📁 Files

- **`qrcrypt.exe`** - Main executable (7.5MB)
- **`qrcrypt-windows-x64.zip`** - Compressed release package (2.8MB)

## 🚀 Quick Start

1. Download `qrcrypt.exe` 
2. Open Command Prompt or PowerShell
3. Run commands:

```cmd
# Test installation
qrcrypt.exe --help

# Validate a seed phrase
qrcrypt.exe validate-phrase --phrase "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"

# Create encrypted QR code
qrcrypt.exe encrypt -o my_wallet.png --secret "your seed phrase here"

# Create plausible deniability QR
qrcrypt.exe decoy-encrypt -o secure_wallet.png --real-secret "real seed" --decoy-type lowvalue
```

## 🛡 Security Features

### ✅ Full Feature Set Available:
- **BIP39 Word Validation** - Catches typos with smart suggestions
- **AES-256-GCM Encryption** - Military-grade authenticated encryption  
- **Shamir's Secret Sharing** - Split secrets into recoverable shares
- **Plausible Deniability** - Hide real secrets behind fake decoy data
- **QR Code Generation** - Secure, scannable backup storage
- **Tails OS Compatible** - Perfect for air-gapped security

### 🎭 Plausible Deniability Example:
```cmd
# Create layered QR with hidden real data and believable decoy
qrcrypt.exe decoy-encrypt -o wallet.png ^
  --real-secret "your valuable 24-word seed phrase here" ^
  --decoy-type lowvalue

# Attacker uses weak password → gets fake "$50 test wallet" 
# You use strong password → gets your real valuable secrets
```

## 📋 System Requirements

- **OS**: Windows 10/11 (64-bit)
- **RAM**: Minimal (CLI tool)
- **Dependencies**: None (statically linked)
- **Antivirus**: May flag as false positive (new executable)

## 🔒 Security Best Practices

### For Maximum Security:
1. **Use with Tails OS** - Download Tails, run from USB
2. **Air-gapped Computer** - Disconnect from internet
3. **Verify Checksums** - Always verify file integrity
4. **Strong Passwords** - Use 20+ character random passwords
5. **Physical Security** - Store QR codes in secure locations

### Recommended Workflow:
1. Boot Tails OS on air-gapped computer
2. Copy `qrcrypt.exe` via USB
3. Generate your encrypted QR codes
4. Print QR codes on paper
5. Store paper copies in multiple secure locations
6. Securely wipe the computer

## 🧪 Testing the Release

```cmd
# Test word validation (should pass)
qrcrypt.exe validate-phrase --phrase "abandon ability able about above absent absorb abstract absurd abuse access account"

# Test with typo (should suggest corrections)
qrcrypt.exe validate-phrase --phrase "abandon ability able about above absent absorb abstract absurd abuse access acount"

# Test decoy generation
qrcrypt.exe decoy-encrypt -o test.png --real-secret "test real secret" --decoy-secret "test decoy secret"

# Then test both passwords work:
qrcrypt.exe decrypt -i test.png  # Try both weak and strong passwords
```

## ⚠️ Important Notes

- **Password Input**: Windows version supports secure password input
- **File Paths**: Use quotes for paths with spaces: `"C:\My Folder\wallet.png"`
- **Antivirus**: May need to whitelist the executable
- **Updates**: Check GitHub releases for newer versions

## 🔧 Build Information

- **Built with**: Rust 1.83+ cross-compilation
- **Cross-compiler**: x86_64-w64-mingw32-gcc
- **Optimization**: Release mode with full optimizations
- **Static Linking**: No external DLL dependencies required

## 🆘 Troubleshooting

### "Windows protected your PC" message:
1. Click "More info"
2. Click "Run anyway"  
3. (This is normal for new executables without code signing certificates)

### Antivirus false positives:
- Add `qrcrypt.exe` to antivirus whitelist
- The tool uses cryptographic functions which may trigger heuristics

### Command not found:
- Ensure you're in the same directory as `qrcrypt.exe`
- Or add the directory to your Windows PATH

## 📜 License & Legal

- **License**: MIT License
- **Open Source**: Full source code available on GitHub
- **Audit**: Code is available for security review
- **No Warranty**: Use at your own risk for valuable crypto assets

---

**⚡ QRCrypt - Secure your crypto seeds with military-grade encryption and plausible deniability!**