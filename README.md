# Multi File Downloader

<!--toc:start-->

- [Multi File Downloader](#multi-file-downloader)
  - [Features](#features)
  - [Usage](#usage)
  - [Options](#options)
  - [Installation](#installation)
  - [License](#license)
  <!--toc:end-->

A simple tool to download multiple files from the command line concurrently.

## Features

- Download multiple files from the command line concurrently
- Clean destination directory before downloading
- Show progress bars for each file and an overall progress bar
- Customizable number of workers
- Customizable destination directory
- Show total downloaded size, per file, total time, eta, and average speed
- Shell completion for bash, zsh, fish, powershell, and elvish

## Usage

```sh

multifiledownloader -w=8 --dest=~/Downloads --urls=https://example.com/file1.txt,https://example.com/file2.txt,https://example.com/file3.txt
multifiledownloader -w=8 --dest=~/Downloads/test --clean --urls=https://example.com/file1.txt,https://example.com/file2.txt,https://example.com/file3.txt

# generate shell completion script
multifiledownloader --completion bash | tee /usr/local/etc/bash_completion.d/multifiledownloader
multifiledownloader --completion zsh | tee /usr/local/share/zsh/site-functions/_multifiledownloader
multifiledownloader --completion fish | tee /usr/local/share/fish/vendor_completions.d/multifiledownloader.fish
multifiledownloader --completion powershell | tee $PROFILE\multifiledownloader.ps1
multifiledownloader --completion elvish | tee $HOME/.elvish/completions/multifiledownloader.elv

```

### Read Url from a file one per line

```sh
multifiledownloader -w=8 --dest=~/Downloads --urls=$(cat urls.txt|tr '\n' ','|sed 's/,$//g')
```

## Options

| Option        | Description                                    | Default           |
| ------------- | ---------------------------------------------- | ----------------- |
| -w, --workers | Number of workers to use                       | 8                 |
| -d, --dest    | Destination directory                          | current directory |
| -u, --urls    | Comma-separated list of URLs to download       | -                 |
| -c, --clean   | Clean destination directory before downloading | false             |
| -h, --help    | Show this help message                         | false             |
| -V, --version | Show version information                       | false             |
| --completion  | Generate shell completion script               | false             |

## Installation

```sh

cargo install --git https://github.com/tralahm/multifiledownloader-rs
```

## License

This project is licensed under the [MIT License](LICENSE).
