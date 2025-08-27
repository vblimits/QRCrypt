# Plausible Deniability in QRCrypt

QRCrypt supports **plausible deniability** through layered encryption - a single QR code that contains both decoy data and your real secrets, protected by different passwords.

## How It Works

When someone forces you to reveal your crypto wallet seed phrase, you can give them the **weak decoy password** instead of your real one. They'll get believable but worthless decoy data, while your real valuable secrets remain hidden.

### Security Model

1. **Decoy Layer**: Protected by a weak, easily guessable password
   - Contains fake/low-value seed phrases  
   - Should look realistic to an attacker
   - Password should be simple (dictionary word, etc.)

2. **Hidden Layer**: Protected by a strong, complex password
   - Contains your actual valuable seed phrases
   - Never reveal this password under any circumstance
   - Use a truly random, long password

## Usage

### Creating a Layered QR Code

```bash
# Interactive mode (will prompt for all inputs)
qrcrypt decoy-encrypt -o my_wallet.png

# With predefined real secret
qrcrypt decoy-encrypt -o my_wallet.png \
  --real-secret "your actual 12+ word seed phrase here"

# Generate different types of decoys
qrcrypt decoy-encrypt -o wallet.png --decoy-type lowvalue    # Appears to be low-value wallet
qrcrypt decoy-encrypt -o wallet.png --decoy-type empty      # Appears to be test/demo wallet  
qrcrypt decoy-encrypt -o wallet.png --decoy-type random     # Random valid BIP39 words
```

### Decrypting

Decryption automatically detects layered QR codes:

```bash
# Try with decoy password first (what you'd give an attacker)
qrcrypt decrypt -i my_wallet.png
# Enter: "password123" → reveals decoy data

# Try with real password (your actual secret)  
qrcrypt decrypt -i my_wallet.png  
# Enter: "MyReallyStrongPassword2024!" → reveals real data
```

## Decoy Types

### `lowvalue`
- Generates words that look like a real but low-value wallet
- Includes crypto-related terms mixed with random BIP39 words
- Decoy hint: "Old test wallet - ~$50 in BTC"

### `empty`  
- Generates words that look like an empty/practice wallet
- Includes terms like "test", "demo", "example", "empty"
- Decoy hint: "Demo wallet for learning - no real funds"

### `random`
- Completely random but valid BIP39 words
- Decoy hint: "Practice seed phrase"

## OPSEC Best Practices

### ✅ DO
- Use a weak, guessable decoy password (`password`, `123456`, `wallet`)
- Make the decoy data look realistic but low-value
- Practice your "cover story" about the decoy wallet
- Store decoy and real passwords separately
- Use this feature with Tails OS for maximum security
- Test decryption of both layers before relying on it

### ❌ DON'T  
- Make the decoy password too complex (defeats the purpose)
- Use obviously fake decoy data (like "test test test...")
- Store both passwords in the same location
- Reveal the existence of a hidden layer
- Use the same password for both layers

## Example Scenario

**Attacker**: "Give me your crypto wallet password or else!"

**You**: "Okay, okay! It's just 'password123' - it's my old practice wallet with like $50 in Bitcoin"

*Attacker decrypts and sees*: `"trial demo practice empty test void sample basic exercise simple market coin"`

**Attacker**: "This better be real..."

**You**: "That's all I have! It's just my learning wallet from when I started with crypto"

*Meanwhile your real $50,000 seed phrase remains safely encrypted under the strong password*

## Technical Details

- Uses AES-256-GCM for both encryption layers
- Argon2 password hashing prevents brute force attacks  
- Each layer has independent salts and nonces
- QR codes include metadata hints for the decoy layer
- No way to detect existence of hidden layer without the real password
- Compatible with all existing QRCrypt scanning/sharing features

## Limitations

- QR codes are larger due to dual encryption (may need lower error correction)
- Requires memorizing two passwords instead of one
- Decoy must be believable to be effective
- Not suitable if attacker has deep technical knowledge of QRCrypt

## Legal Disclaimer

This feature is designed for personal security in extreme situations. The authors are not responsible for how this feature is used. Consider the legal implications in your jurisdiction, as some places may have laws against hidden data or cryptography.