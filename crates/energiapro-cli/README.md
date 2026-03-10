# EnergiaPro CLI

The `energiapro` CLI lets you query the EnergiaPro API without writing code.
It can list installations, fetch measurements, and export results as `text`,
`json`, `jsonl`, `csv`, or `parquet`.

## Installation

### macOS

Recommended: install with Homebrew.

```sh
brew tap nhedger/energiapro
brew install energiapro
```

If you prefer a standalone binary, download the asset that matches your Mac:

- Apple Silicon: `energiapro-aarch64-apple-darwin`
- Intel: `energiapro-x86_64-apple-darwin`

Example for Apple Silicon:

```sh
curl -L https://github.com/nhedger/energiapro/releases/latest/download/energiapro-aarch64-apple-darwin -o energiapro
chmod +x energiapro
sudo install -m 0755 energiapro /usr/local/bin/energiapro
```

### Linux

You can install with Homebrew/Linuxbrew:

```sh
brew tap nhedger/energiapro
brew install energiapro
```

Or install a prebuilt binary:

- x86_64: `energiapro-x86_64-unknown-linux-gnu`
- ARM64: `energiapro-aarch64-unknown-linux-gnu`

Example for x86_64:

```sh
curl -L https://github.com/nhedger/energiapro/releases/latest/download/energiapro-x86_64-unknown-linux-gnu -o energiapro
chmod +x energiapro
sudo install -m 0755 energiapro /usr/local/bin/energiapro
```

The published Linux binaries target `glibc` (`*-unknown-linux-gnu`). If you are
on a musl-based distribution such as Alpine, install from source instead.

### Windows

Download the Windows binary from the latest release:

- x86_64: `energiapro-x86_64-pc-windows-msvc.exe`

Example in PowerShell:

```powershell
$binDir = "$HOME\\bin"
New-Item -ItemType Directory -Force -Path $binDir | Out-Null
Invoke-WebRequest -Uri "https://github.com/nhedger/energiapro/releases/latest/download/energiapro-x86_64-pc-windows-msvc.exe" -OutFile "$binDir\\energiapro.exe"
```

Add `$HOME\bin` to your `PATH` if needed, then open a new terminal.

### From source

If you already have Rust installed, you can build and install the CLI on any
supported platform.

Install directly from GitHub:

```sh
cargo install --git https://github.com/nhedger/energiapro --locked energiapro-cli
```

If you already cloned this repository:

```sh
cargo install --path crates/energiapro-cli --locked
```

### Verify the installation

```sh
energiapro --version
```

Each GitHub release also includes a `SHA256SUMS.txt` file if you want to verify
downloaded binaries.

## Configuration

The CLI reads credentials from flags or environment variables:

- `ENERGIAPRO_USERNAME`
- `ENERGIAPRO_SECRET_KEY`
- `ENERGIAPRO_BASE_URL` (optional)

POSIX shells:

```sh
export ENERGIAPRO_USERNAME="your-username"
export ENERGIAPRO_SECRET_KEY="your-secret-key"
```

PowerShell:

```powershell
$env:ENERGIAPRO_USERNAME = "your-username"
$env:ENERGIAPRO_SECRET_KEY = "your-secret-key"
```

Command-line flags such as `--username` and `--secret-key` override the
environment.

## Usage

Show the available commands:

```sh
energiapro --help
```

List installations for a client:

```sh
energiapro installations CLIENT_ID
```

Fetch measurements for one installation:

```sh
energiapro measurements CLIENT_ID INSTALLATION_ID
```

Fetch measurements for a date range and write CSV output:

```sh
energiapro measurements CLIENT_ID INSTALLATION_ID --from 2024-01-01 --to 2024-01-31 --format csv > measurements.csv
```

Write installations as JSON:

```sh
energiapro installations CLIENT_ID --format json > installations.json
```

Available output formats:

- `text` (default)
- `json`
- `jsonl`
- `csv`
- `parquet`

Measurement scopes:

- `lpn-json` (default)
- `gc-plus-json`

For command-specific options, use:

```sh
energiapro installations --help
energiapro measurements --help
```
