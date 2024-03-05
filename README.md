# FileWatcher

## What

CLI tool for directories or files synchronisation

## Why

Just a test project to try filesystem libraries like `notify`.

## Installation

Rust should be preinstalled.
Just clone the repo and use local cargo installation.

```bash
git clone https://github.com/aOri69/FileWatcher.git
cd FileWatcher
cargo install --path .
```

## Usage

Program accepts two arguments:

- Source path
- Destination path

```bash
fsync ./sync_test/source_dir ./sync_test/destination_dir
```

### Environment variables and logging

`RUST_LOG` variable is used for log level control.
If vairable is not set, default `info` level whould be used.
