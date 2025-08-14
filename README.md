# mousequake

[![Test](https://github.com/KanchiShimono/mousequake/actions/workflows/test.yml/badge.svg)](https://github.com/KanchiShimono/mousequake/actions/workflows/test.yml)
[![Release](https://github.com/KanchiShimono/mousequake/actions/workflows/release.yml/badge.svg)](https://github.com/KanchiShimono/mousequake/actions/workflows/release.yml)

Simple tool for automatically shaking the mouse pointer.
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
| `--width` | `-w` | 1 | Distance to move the mouse (pixels) |
| `--interval` | `-i` | 10 | Time between movements (seconds) |
| `--help` | `-h` | | Show help information |
| `--version` | `-V` | | Show version |

### Examples

```sh
# Default: move 1 pixel every 10 seconds
mousequake

# Move 5 pixels every 30 seconds
mousequake -w 5 -i 30

# Move 2 pixels every 5 seconds
mousequake --width 2 --interval 5
```

## Shell Completion

Generate completion scripts for your shell:

```sh
# Bash
mousequake completion bash > ~/.local/share/bash-completion/completions/mousequake

# Zsh
mousequake completion zsh > ~/.zfunc/_mousequake

# Fish
mousequake completion fish > ~/.config/fish/completions/mousequake.fish

# PowerShell
mousequake completion powershell > _mousequake.ps1
```

## Installation

### Homebrew

```sh
brew install KanchiShimono/tap/mousequake
```

> **Note:** Shell completions are automatically installed when using Homebrew.

### Others

You can download the binary from [GitHub release page](https://github.com/KanchiShimono/mousequake/releases).
