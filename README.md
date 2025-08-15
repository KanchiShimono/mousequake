# mousequake

[![Rust: 2024](https://img.shields.io/badge/Rust-2024%20Edition-orange?logo=rust)](https://doc.rust-lang.org/edition-guide/)
[![Test](https://github.com/KanchiShimono/mousequake/actions/workflows/test.yml/badge.svg)](https://github.com/KanchiShimono/mousequake/actions/workflows/test.yml)
[![Release](https://github.com/KanchiShimono/mousequake/actions/workflows/release.yml/badge.svg)](https://github.com/KanchiShimono/mousequake/actions/workflows/release.yml)
[![Homebrew](https://img.shields.io/badge/Homebrew-kanchishimono/tap-yellow)](https://github.com/KanchiShimono/homebrew-tap)

Simple tool for automatically shaking the mouse pointer with various trajectory patterns.
This tool saves your time from forced machine sleeping during automated work.

## Quick Start

```sh
# Start with default settings (move 1 pixel every 10 seconds)
mousequake

# Stop with Ctrl+C
```

## Usage

```sh
mousequake [OPTIONS] [COMMAND]
```

### Options

| Option | Short | Default | Description |
|--------|-------|---------|-------------|
| `--size` | `-s` | 1 | Maximum width of the trajectory pattern (pixels) |
| `--interval` | `-i` | 10 | Time between movements (seconds) |
| `--trajectory` | `-t` | linear | Trajectory pattern (linear, circle, star, square, infinity) |
| `--help` | `-h` | | Show help information |
| `--version` | `-V` | | Show version |

### Examples

```sh
# Default: linear pattern with 1 pixel size every 10 seconds
mousequake

# Pattern size of 5 pixels every 30 seconds
mousequake -s 5 -i 30

# Circle pattern with 10px diameter
mousequake -t circle -s 10

# Star pattern with 20px size every 5 seconds
mousequake -t star -s 20 -i 5

# Infinity/figure-8 pattern
mousequake -t infinity -s 15
```

## Shell Completion

Generate completion scripts for your shell:

```sh
# Bash
mousequake completion bash > ~/.local/share/bash-completion/completions/mousequake

# Zsh (choose one of the following paths)
mousequake completion zsh > ~/.zsh/completions/_mousequake

# Fish
mousequake completion fish > ~/.config/fish/completions/mousequake.fish

# PowerShell
# Unix/Linux/macOS:
mousequake completion powershell > ~/.config/powershell/Completions/mousequake.ps1
# Windows:
mousequake completion powershell > "$HOME\Documents\PowerShell\Completions\mousequake.ps1"
```

## Installation

### Homebrew

```sh
brew install KanchiShimono/tap/mousequake
```

> **Note:** Shell completions are automatically installed when using Homebrew.

### Others

You can download the binary from [GitHub release page](https://github.com/KanchiShimono/mousequake/releases).
