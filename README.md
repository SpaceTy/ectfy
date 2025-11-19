# ectfy

A simple command-line tool for encrypting and decrypting files and folders using AES-256-GCM.

## Installation

```bash
git clone https://github.com/SpaceTy/ectfy
cd ectfy
cargo build --release
```

The binary will be at `target/release/ectfy`. Move it to your PATH if desired.

## Requirements

- Rust (latest stable)
- `fzf` (for interactive file selection)

Install fzf:
```bash
# Arch Linux
sudo pacman -S fzf

# Ubuntu/Debian
sudo apt install fzf

# macOS
brew install fzf
```

## Usage

### Interactive Mode

Run without arguments to select files interactively:

```bash
ectfy
```

Use arrow keys to navigate, SPACE to select multiple files, ENTER to confirm. You'll be prompted for passwords and helper questions.

### Direct Mode

Provide a file or folder path:

```bash
# Encrypt a file
ectfy document.pdf

# Encrypt a folder
ectfy ~/Documents/

# Decrypt (automatically detects .ect files)
ectfy document.pdf.ect
```

### Options

- `-s, --show-password`: Show password while typing

## How It Works

- Encrypted files get a `.ect` extension
- Uses AES-256-GCM with PBKDF2 key derivation (100,000 iterations)
- Stores a helper question with each encrypted file for password recovery
- Folders are processed recursively

## Examples

Encrypting:
```
$ ectfy photo.jpg
Enter password: 
Confirm password: 
Enter helper question: What is your favorite movie?
✓ Encrypted photo.jpg → photo.jpg.ect
```

Decrypting:
```
$ ectfy photo.jpg.ect
Helper question: What is your favorite movie?
Enter password: 
✓ Decrypted photo.jpg.ect → photo.jpg
```

## Testing

```bash
cargo test
```

## License

MIT
