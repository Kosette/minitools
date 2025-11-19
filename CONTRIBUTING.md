# Contributing to Minitools

Thank you for considering contributing to Minitools! This document provides guidelines and instructions for contributing.

## 🚀 Getting Started

### Prerequisites

- Rust 1.70 or later
- Git
- For testing specific tools:
  - FFmpeg (for ffmerge)
  - yt-dlp (for bdown)

### Setting Up Development Environment

1. Clone the repository:
```bash
git clone https://github.com/Kosette/minitools.git
cd minitools
```

2. Build the project:
```bash
cargo build --workspace
```

3. Run tests:
```bash
cargo test --workspace
```

## 📝 Code Style

### Formatting

This project uses `rustfmt` for code formatting. Before submitting a PR, please run:

```bash
cargo fmt --all
```

### Linting

Run Clippy to catch common mistakes:

```bash
cargo clippy --workspace -- -D warnings
```

### Code Guidelines

- **Error Handling**: Use `Result` types and provide meaningful error messages
- **Comments**: Add comments for complex logic, but prefer self-documenting code
- **Performance**: Consider performance for file operations and large data processing
- **Memory**: Use streaming for large files rather than loading everything into memory
- **Cross-platform**: Be mindful of Windows/Linux/macOS differences

## 🐛 Reporting Bugs

When reporting bugs, please include:

1. **Description**: Clear description of the issue
2. **Steps to Reproduce**: Detailed steps to reproduce the behavior
3. **Expected Behavior**: What you expected to happen
4. **Actual Behavior**: What actually happened
5. **Environment**: OS, Rust version, tool version
6. **Logs/Screenshots**: Any relevant logs or screenshots

## ✨ Suggesting Features

Feature suggestions are welcome! Please:

1. Check if the feature has already been suggested
2. Provide a clear description of the feature
3. Explain the use case and benefits
4. Consider implementation complexity

## 🔧 Pull Request Process

### Before Submitting

1. **Create an Issue**: Discuss major changes in an issue first
2. **Branch**: Create a feature branch from `main`
3. **Test**: Ensure all tests pass
4. **Format**: Run `cargo fmt --all`
5. **Lint**: Run `cargo clippy --workspace`
6. **Documentation**: Update README.md if needed

### PR Guidelines

1. **Title**: Use clear, descriptive titles
2. **Description**: Explain what changes you made and why
3. **Tests**: Add tests for new features
4. **Documentation**: Update docs for user-facing changes
5. **Commits**: Use meaningful commit messages

### Commit Message Format

```
<type>: <description>

[optional body]
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting)
- `refactor`: Code refactoring
- `perf`: Performance improvements
- `test`: Adding tests
- `chore`: Maintenance tasks

Example:
```
feat: add copy-to-clipboard for hash results

Added clipboard copy buttons next to each hash value in the hasher
GUI for easier copying of hash results.
```

## 🏗️ Project Structure

```
minitools/
├── src/              # Main package and CLI tools
│   ├── bin/         # Command-line binaries
│   │   ├── b64.rs       # Base64 encoder/decoder
│   │   ├── bcrypt.rs    # Password hasher
│   │   ├── dupfinder.rs # Duplicate file finder
│   │   └── pngc.rs      # PNG compressor GUI
│   └── main.rs      # Main entry point
├── oxipng-gui/      # PNG optimizer GUI
├── ffmerge/         # Video/audio merger
├── hasher/          # Hash calculator
├── rnmd/            # File renamer
├── bdown/           # Downloader GUI
└── resources/       # Icons, fonts, assets
```

## 🧪 Testing

### Running Tests

```bash
# All tests
cargo test --workspace

# Specific package
cargo test -p hasher

# With output
cargo test --workspace -- --nocapture
```

### Writing Tests

- Add unit tests for pure functions
- Add integration tests for tools
- Test error cases and edge cases
- Mock external dependencies when possible

## 📚 Adding a New Tool

To add a new tool:

1. **Decide Type**: CLI binary or GUI application
2. **For CLI**: Add to `src/bin/`
3. **For GUI**: Create new workspace member
4. **Update Cargo.toml**: Add dependencies and features
5. **Add Icon**: Place icon in `resources/`
6. **Documentation**: Update README.md
7. **Tests**: Add appropriate tests

### Example: Adding a New CLI Tool

1. Create `src/bin/mytool.rs`:
```rust
fn main() {
    println!("My new tool!");
}
```

2. Update `Cargo.toml`:
```toml
[[bin]]
name = "mytool"
required-features = ["mytool"]
path = "src/bin/mytool.rs"

[features]
mytool = ["dep:some-dependency"]

[dependencies]
some-dependency = { version = "1.0", optional = true }
```

3. Update README.md with usage instructions

## 🎨 GUI Guidelines

For GUI applications using egui:

- **Responsiveness**: Don't block the UI thread
- **Progress**: Show progress for long operations
- **Cancel**: Allow users to cancel long operations
- **Drag & Drop**: Support drag and drop where appropriate
- **Keyboard**: Support keyboard shortcuts
- **Accessibility**: Use clear labels and tooltips

## 🔒 Security

- Never commit secrets or credentials
- Validate all user input
- Handle file operations safely
- Be cautious with external commands
- Report security issues privately

## 📄 License

By contributing, you agree that your contributions will be licensed under the same license as the project.

## 🤝 Code of Conduct

- Be respectful and inclusive
- Welcome newcomers
- Focus on constructive feedback
- Collaborate openly

## 💡 Tips for Contributors

- **Start Small**: Begin with small issues or documentation
- **Ask Questions**: Don't hesitate to ask for clarification
- **Be Patient**: Reviews take time
- **Learn**: Use this as an opportunity to learn Rust and GUI programming

## 🌟 Recognition

Contributors will be recognized in the project. Thank you for making Minitools better!

---

If you have questions, feel free to open an issue for discussion.
