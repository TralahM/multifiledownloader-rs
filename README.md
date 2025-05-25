# Multi File Downloader

[![Crates.io](https://img.shields.io/crates/v/multifiledownloader)](https://crates.io/crates/multifiledownloader)
[![CI](https://github.com/TralahM/multifiledownloader-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/TralahM/multifiledownloader-rs/actions/workflows/ci.yml)

A high-performance, concurrent multi-file downloader written in Rust with progress tracking and error handling.

<!--toc:start-->

- [Multi File Downloader](#multi-file-downloader)
  - [Features](#features)
  - [Usage](#usage)
    - [Basic Usage](#basic-usage)
    - [Advanced Usage](#advanced-usage)
    - [Reading URLs from a File](#reading-urls-from-a-file)
    - [Shell Completion](#shell-completion)
  - [Options](#options)
  - [Installation](#installation)
  - [Troubleshooting](#troubleshooting)
  - [License](#license)
  <!--toc:end-->

## Features

- 🚀 Concurrent downloads with configurable worker count
- 📊 Progress bars for individual files and overall progress
- 🔄 Resume support for partially downloaded files
- 🗑️ Clean destination directory before downloading
- 📂 Customizable destination directory (supports tilde expansion) 
   + The destination directory is created if it does not exist automatically
- 🔄 Automatic shell completion support
- 📊 Human-readable download statistics
- 🛠️ Robust error handling and logging
- 📝 dotenv support for configuration


## Usage

### Basic Usage

Download multiple files concurrently:
```bash
multifiledownloader -w 8 --dest ~/Downloads --urls "https://example.com/file1.txt,https://example.com/file2.txt"
```

### Advanced Usage

- Specify custom destination directory:
  ```bash
  multifiledownloader -d ~/Downloads/custom-dir -u "url1,url2,url3"
  ```

- Clean destination directory before downloading:
  ```bash
  multifiledownloader --clean -u "url1,url2"
  ```

- Set custom number of workers:
  ```bash
  multifiledownloader -w 4 -u "url1,url2"
  ```

### Reading URLs from a File

You can read URLs from a file where each URL is on a new line:
```bash
# Create a file with URLs
$ cat > urls.txt << EOF
https://example.com/file1.txt
https://example.com/file2.txt
EOF

# Download using the file
$ multifiledownloader -w 8 --dest ~/Downloads --urls "$(cat urls.txt | tr '\n' ',' | sed 's/,$//g')"
```

### Shell Completion

Generate shell completion scripts for your shell:

```bash
# Bash
multifiledownloader --completion bash | sudo tee /usr/local/etc/bash_completion.d/multifiledownloader

# Zsh or to a directory in your $fpath
multifiledownloader --completion zsh | sudo tee /usr/local/share/zsh/site-functions/_multifiledownloader

# Fish
multifiledownloader --completion fish | sudo tee /usr/local/share/fish/vendor_completions.d/multifiledownloader.fish

# PowerShell
multifiledownloader --completion powershell | Out-File -FilePath $PROFILE\multifiledownloader.ps1

# Elvish
multifiledownloader --completion elvish | tee $HOME/.elvish/completions/multifiledownloader.elv
```

## Options

| Option        | Description                                    | Default           |
| ------------- | ---------------------------------------------- | ----------------- |
| -w, --workers | Number of concurrent download workers          | CPU cores count   |
| -d, --dest    | Destination directory for downloaded files     | current directory |
| -u, --urls    | Comma-separated list of URLs to download       | required          |
| -c, --clean   | Clean destination directory before downloading | false             |
| --completion  | Generate shell completion script               | -                 |
| -h, --help    | Show help message                              | -                 |
| -V, --version | Show version information                       | -                 |

## Installation

### Using Cargo

```bash
cargo install multifiledownloader
```

### From Source

```bash
git clone https://github.com/tralahm/multifiledownloader-rs.git
cd multifiledownloader-rs
cargo build --release
cp target/release/multifiledownloader /usr/local/bin/
```

## Troubleshooting

### Common Issues

1. **Permission Errors**
   - Ensure you have write permissions to the destination directory
   - Use `--clean` flag to remove existing files before downloading

2. **Network Issues**
   - Check if URLs are accessible
   - Use fewer workers if experiencing connection timeouts

3. **Progress Bar Issues**
   - Progress bars may not display correctly in some terminals
   - Try using a different terminal emulator if experiencing issues

### Debugging

To enable debug logging:
```bash
RUST_LOG=debug multifiledownloader -u "url1,url2"
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
