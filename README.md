# Minitools

A collection of useful utility tools written in Rust, featuring both GUI and command-line applications for common tasks.

## 🚀 Features

### GUI Applications

#### 1. **Oxipng GUI** - PNG Image Optimizer
Optimize PNG images with drag-and-drop support and multi-threading.

**Features:**
- Drag and drop files or folders
- Adjustable optimization level (0-6)
- Configurable thread count
- Recursive folder scanning
- Real-time progress tracking

#### 2. **FFmerge** - Video/Audio Merger
Merge video and audio files using FFmpeg.

**Features:**
- Drag and drop support for video and audio files
- Automatic file type detection
- Option to delete source files after merging
- Supports MP4, MKV, AVI (video) and M4A, MP3, AAC (audio)

#### 3. **Hasher** - File Hash Calculator
Calculate multiple hash values for files.

**Features:**
- Calculate SHA1, SHA256, SHA512, and MD5 hashes
- Drag and drop file support
- Export results to TXT or CSV format
- Efficient streaming for large files

#### 4. **RNMD (Rename MD5)** - Hash-based File Renamer
Rename files based on their hash values.

**Features:**
- Support for MD5 and BLAKE3 hash algorithms
- Recursive folder processing
- Real-time progress tracking
- Cancel operation support

#### 5. **BDown** - yt-dlp Downloader GUI
Simple GUI for yt-dlp video downloader.

**Features:**
- Batch download support (multiple URLs)
- Real-time log output
- Cancel download support

### Command-Line Tools

#### 1. **b64** - Base64 Encoder/Decoder
```bash
# Encode
b64 encode "Hello, World!"
b64 e "Hello, World!"

# Decode
b64 decode SGVsbG8sIFdvcmxkIQ==
b64 d SGVsbG8sIFdvcmxkIQ==

# Using stdin
echo "Hello, World!" | b64 encode
```

#### 2. **bcrypt** - Password Hasher
```bash
bcrypt mypassword
```
Generates a bcrypt hash and verifies it.

#### 3. **dupfinder** - Duplicate File Finder
```bash
# Find duplicates in current directory
dupfinder *

# Find duplicates with glob pattern
dupfinder *.jpg *.png

# Find duplicates in specific directory
dupfinder /path/to/directory/*
```

#### 4. **pngc** - PNG Compressor (Simple GUI)
A simpler version of the Oxipng GUI with basic compression features.

## 📦 Installation

### Prerequisites
- Rust toolchain (1.70 or later)
- For FFmerge: FFmpeg must be installed and available in PATH
- For BDown: yt-dlp must be installed and available in PATH

### Building from Source

```bash
# Clone the repository
git clone https://github.com/Kosette/minitools.git
cd minitools

# Build all tools
cargo build --release --workspace

# Build specific tools with features
cargo build --release --features b64
cargo build --release --features bcrypt
cargo build --release --features dupfinder
cargo build --release --features pngc

# Build specific GUI application
cargo build --release -p oxipng-gui
cargo build --release -p ffmerge
cargo build --release -p hasher
cargo build --release -p rnmd
cargo build --release -p bdown
```

### Binary Locations
After building, binaries will be located in:
- `target/release/` - for all executables

## 🎯 Usage

### GUI Applications
Simply run the executable. All GUI applications support:
- Drag and drop functionality
- Intuitive button-based interfaces
- Real-time status updates

### Command-Line Tools

#### Base64 Encoding/Decoding
```bash
# Encode text
b64 encode "sensitive data"

# Decode Base64
b64 decode "c2Vuc2l0aXZlIGRhdGE="

# Pipe support
cat file.txt | b64 encode > encoded.txt
cat encoded.txt | b64 decode
```

#### Password Hashing
```bash
bcrypt "my_secure_password"
```

#### Finding Duplicates
```bash
# In current directory
dupfinder *

# Recursive search
dupfinder **/*

# Specific file types
dupfinder *.jpg *.png *.gif
```

## 🛠️ Development

### Project Structure
```
minitools/
├── src/              # Main package and CLI tools
│   ├── bin/         # Command-line tool binaries
│   └── main.rs      # Main entry point
├── oxipng-gui/      # PNG optimizer GUI
├── ffmerge/         # Video/audio merger GUI
├── hasher/          # Hash calculator GUI
├── rnmd/            # File renamer GUI
├── bdown/           # Downloader GUI
└── resources/       # Icons, fonts, and assets
```

### Testing
```bash
# Run tests
cargo test --workspace

# Run specific package tests
cargo test -p oxipng-gui
```

### Code Style
```bash
# Format code
cargo fmt --all

# Run clippy
cargo clippy --workspace
```

## 📝 Technical Details

### Dependencies
- **eframe**: GUI framework (egui)
- **tokio**: Async runtime (for bdown)
- **rayon**: Data parallelism (for oxipng-gui)
- **walkdir**: Directory traversal
- **Various hash libraries**: md5, sha1, sha2, blake3

### Build Optimizations
The release profile is configured for size optimization:
- LTO: thin
- Code generation units: 1
- Optimization level: size (s)
- Strip symbols: enabled
- Panic: abort

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## 📄 License

See the repository for license information.

## ⚠️ Notes

- **FFmerge** requires FFmpeg to be installed separately
- **BDown** requires yt-dlp to be installed separately
- Some features are Windows-specific (like window creation flags)
- GUI applications use custom fonts for better international character support

## 🔧 Troubleshooting

### Common Issues

1. **"command not found" for FFmpeg or yt-dlp**
   - Install the required external tool and ensure it's in your PATH

2. **GUI application doesn't start**
   - Make sure you have the necessary graphics drivers
   - Try running from a terminal to see error messages

3. **Build fails**
   - Ensure you have the latest Rust toolchain: `rustup update`
   - Clean and rebuild: `cargo clean && cargo build --release --workspace`

## 🎨 Screenshots

(Screenshots would be added here for each GUI application)

## 📈 Performance Tips

- **Oxipng GUI**: Use fewer threads on systems with limited CPU cores
- **Hasher**: Process files individually for very large files to see progress
- **RNMD**: Use non-recursive mode for faster processing of flat directories
- **Dupfinder**: Works best with glob patterns to limit scope
