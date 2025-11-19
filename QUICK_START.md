# Quick Start Guide

快速开始使用 Minitools 的指南。

## 🚀 快速安装

### 前置要求
- Rust 1.70+ ([安装 Rust](https://rustup.rs/))
- Git

### 克隆并构建
```bash
# 克隆仓库
git clone https://github.com/Kosette/minitools.git
cd minitools

# 构建所有工具（开发版本）
cargo build --workspace

# 或构建优化的发布版本
cargo build --workspace --release
```

## 🎯 快速使用示例

### GUI 工具

#### 运行 PNG 优化器
```bash
# 开发版本
cargo run -p oxipng-gui

# 发布版本
./target/release/oxipng-gui
```

#### 运行文件哈希计算器
```bash
cargo run -p hasher
# 或
./target/release/hasher
```

#### 运行视频音频合并器
```bash
# 需要先安装 FFmpeg
cargo run -p ffmerge
```

#### 运行文件重命名工具
```bash
cargo run -p rnmd
```

#### 运行下载器
```bash
# 需要先安装 yt-dlp
cargo run -p bdown
```

### 命令行工具

#### Base64 编码/解码
```bash
# 构建
cargo build --features b64

# 编码
./target/debug/b64 encode "Hello, World!"
# 输出: SGVsbG8sIFdvcmxkIQ==

# 解码
./target/debug/b64 decode SGVsbG8sIFdvcmxkIQ==
# 输出: Hello, World!

# 使用管道
echo "secret text" | ./target/debug/b64 encode
```

#### 密码哈希
```bash
# 构建
cargo build --features bcrypt

# 生成哈希
./target/debug/bcrypt hash mypassword

# 验证密码
./target/debug/bcrypt verify mypassword '$2b$12$...'
```

#### 查找重复文件
```bash
# 构建
cargo build --features dupfinder

# 在当前目录查找
./target/debug/dupfinder *

# 查找特定类型的文件
./target/debug/dupfinder *.jpg *.png *.gif

# 在子目录中查找
./target/debug/dupfinder **/*
```

#### PNG 压缩（简单 GUI）
```bash
# 构建
cargo build --features pngc

# 运行
./target/debug/pngc
```

## 📦 构建特定工具

### 仅构建 GUI 工具
```bash
cargo build -p oxipng-gui --release
cargo build -p hasher --release
cargo build -p ffmerge --release
cargo build -p rnmd --release
cargo build -p bdown --release
```

### 仅构建 CLI 工具
```bash
cargo build --features b64 --release
cargo build --features bcrypt --release
cargo build --features dupfinder --release
cargo build --features pngc --release
```

## 🔧 常见问题

### FFmpeg 未找到
```bash
# Ubuntu/Debian
sudo apt install ffmpeg

# macOS
brew install ffmpeg

# Windows
# 从 https://ffmpeg.org/download.html 下载并添加到 PATH
```

### yt-dlp 未找到
```bash
# 使用 pip
pip install yt-dlp

# 或使用 pipx
pipx install yt-dlp

# macOS
brew install yt-dlp

# Windows
# 从 https://github.com/yt-dlp/yt-dlp/releases 下载
```

### 构建失败
```bash
# 清理并重新构建
cargo clean
cargo build --workspace --release

# 更新 Rust
rustup update stable
```

### GUI 窗口不显示
```bash
# 确保安装了必要的系统库
# Ubuntu/Debian
sudo apt install libgtk-3-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev

# Fedora
sudo dnf install gtk3-devel
```

## 💡 使用技巧

### 1. 批量处理文件
GUI 工具都支持拖放功能，可以一次性拖入多个文件或文件夹。

### 2. 使用通配符
```bash
# 查找所有 PNG 文件的重复项
dupfinder **/*.png

# 处理多种格式
dupfinder *.{jpg,png,gif}
```

### 3. 管道操作
```bash
# 编码文件内容
cat file.txt | b64 encode > encoded.txt

# 解码并保存
cat encoded.txt | b64 decode > decoded.txt
```

### 4. 脚本集成
```bash
#!/bin/bash
# 自动查找并报告重复文件
dupfinder /path/to/directory/* > duplicates.txt
if [ -s duplicates.txt ]; then
    echo "Found duplicates!"
    cat duplicates.txt
fi
```

## 📊 性能优化建议

### oxipng-gui
- **大量小文件**: 使用更多线程
- **少量大文件**: 使用较少线程避免内存压力
- **最佳设置**: 线程数 = CPU 核心数 / 2

### dupfinder
- **大型目录**: 使用特定的通配符模式而不是 `*`
- **网络驱动器**: 考虑先复制到本地

### hasher
- **大文件**: 工具已优化，使用流式处理
- **批量处理**: 逐个处理文件以查看实时进度

## 🎨 自定义

### 修改窗口大小
编辑各工具的 `main.rs` 文件:
```rust
let native_options = eframe::NativeOptions {
    viewport: egui::ViewportBuilder::default()
        .with_inner_size([600.0, 500.0])  // 修改这里
        // ...
};
```

### 修改优化等级
编辑 `Cargo.toml` 的 `[profile.release]` 部分:
```toml
[profile.release]
opt-level = 3  # 最大优化（默认是 "s" 用于大小优化）
lto = "fat"    # 完整 LTO
```

## 🆘 获取帮助

- **查看详细文档**: [README.md](README.md)
- **贡献指南**: [CONTRIBUTING.md](CONTRIBUTING.md)
- **改进建议**: [IMPROVEMENTS_SUMMARY.md](IMPROVEMENTS_SUMMARY.md)
- **问题报告**: GitHub Issues

## 📝 下一步

1. ✅ 尝试运行各个工具
2. 📖 阅读 [README.md](README.md) 了解详细功能
3. 🔧 根据需要自定义配置
4. 🤝 考虑贡献代码（参见 [CONTRIBUTING.md](CONTRIBUTING.md)）

---

祝使用愉快！如有问题，请随时提出 issue。
