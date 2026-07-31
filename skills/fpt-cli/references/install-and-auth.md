## Install and update `fpt-cli`

### Released binary installation
Prefer downloading a release archive, optionally verifying `fpt-checksums.txt`, and then placing the binary into the same default install path used by the official installers.
Avoid remote-script piping and direct remote-expression execution patterns in automated environments.

Default install locations:
- macOS / Linux installer default: `~/.local/bin/fpt`
- Windows installer default: `%USERPROFILE%\.fpt\bin\fpt.exe`
- Override with `FPT_INSTALL_DIR` when a different target directory is required.

Release asset names:

- Linux: `fpt-v0.1.0-x86_64-unknown-linux-gnu.tar.gz`
- Windows: `fpt-v0.1.0-x86_64-pc-windows-msvc.zip`
- macOS (Intel): `fpt-v0.1.0-x86_64-apple-darwin.tar.gz`
- macOS (Apple Silicon): `fpt-v0.1.0-aarch64-apple-darwin.tar.gz`

#### macOS / Linux example
```bash
export FPT_VERSION="v0.1.0"
export FPT_INSTALL_DIR="${FPT_INSTALL_DIR:-$HOME/.local/bin}"
export FPT_ARCHIVE="fpt-${FPT_VERSION}-x86_64-unknown-linux-gnu.tar.gz"
curl -fLO "https://github.com/dcc-mcp/fpt-cli/releases/download/${FPT_VERSION}/${FPT_ARCHIVE}"
curl -fLO "https://github.com/dcc-mcp/fpt-cli/releases/download/${FPT_VERSION}/fpt-checksums.txt"
sha256sum -c --ignore-missing fpt-checksums.txt
tar -xzf "${FPT_ARCHIVE}"
mkdir -p "$FPT_INSTALL_DIR"
install -m 755 ./fpt "$FPT_INSTALL_DIR/fpt"
"$FPT_INSTALL_DIR/fpt" capabilities --output json
```

#### Windows PowerShell example
```powershell
$FptVersion = "v0.1.0"
$InstallDir = if ($env:FPT_INSTALL_DIR) { $env:FPT_INSTALL_DIR } else { Join-Path $env:USERPROFILE ".fpt\bin" }
$Archive = "fpt-$FptVersion-x86_64-pc-windows-msvc.zip"
$ExtractDir = Join-Path $env:TEMP "fpt-extract"
Invoke-WebRequest -Uri "https://github.com/dcc-mcp/fpt-cli/releases/download/$FptVersion/$Archive" -OutFile $Archive
Expand-Archive -Path $Archive -DestinationPath $ExtractDir -Force
New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
Copy-Item -Path (Join-Path $ExtractDir "fpt.exe") -Destination (Join-Path $InstallDir "fpt.exe") -Force
& (Join-Path $InstallDir "fpt.exe") capabilities --output json
```

### In-place update
Use the released binary's self-update command when `fpt` is already installed.

```bash
fpt self update --check --output pretty-json
fpt self update
```


### Repository-local execution
When operating from the source checkout, prefer the repository-managed environment.

```bash
vx cargo run -p fpt-cli -- capabilities --output json
vx just test
```

## Authentication quick reference

### Preferred environment variables

| Variable | Required | Auth modes | Description |
|---|---|---|---|
| `FPT_SITE` | always | all | Full URL of the ShotGrid / FPT site |
| `FPT_AUTH_MODE` | always | all | One of `script`, `user_password`, `session_token` |
| `FPT_API_VERSION` | optional | all | API version override (e.g. `v1.1`); defaults to the CLI built-in |
| `FPT_SCRIPT_NAME` | required | `script` | Script entity name registered in ShotGrid |
| `FPT_SCRIPT_KEY` | required | `script` | Application key for the script entity |
| `FPT_USERNAME` | required | `user_password` | ShotGrid user login (email) |
| `FPT_PASSWORD` | required | `user_password` | ShotGrid user password |
| `FPT_AUTH_TOKEN` | optional | `user_password` | One-time 2FA token when the site enforces MFA |
| `FPT_SESSION_TOKEN` | required | `session_token` | Pre-obtained session token |

### Auth modes
- `script` — requires `FPT_SCRIPT_NAME` + `FPT_SCRIPT_KEY`
- `user_password` — requires `FPT_USERNAME` + `FPT_PASSWORD`; optionally `FPT_AUTH_TOKEN` for 2FA
- `session_token` — requires `FPT_SESSION_TOKEN`

### Auth validation
Validate credentials before running entity or schema commands.

```bash
fpt auth test --output json
```

### Example: script auth (bash)
```bash
export FPT_SITE="https://example.shotgrid.autodesk.com"
export FPT_AUTH_MODE="script"
export FPT_SCRIPT_NAME="openclaw"
export FPT_SCRIPT_KEY="your-script-key"
fpt auth test --output json
```

### Example: user-password auth (bash)
```bash
export FPT_SITE="https://example.shotgrid.autodesk.com"
export FPT_AUTH_MODE="user_password"
export FPT_USERNAME="user@example.com"
export FPT_PASSWORD="your-password"
# Optional: add FPT_AUTH_TOKEN when the site requires 2FA
# export FPT_AUTH_TOKEN="123456"
fpt auth test --output json
```

### Example: session-token auth (bash)
```bash
export FPT_SITE="https://example.shotgrid.autodesk.com"
export FPT_AUTH_MODE="session_token"
export FPT_SESSION_TOKEN="your-session-token"
fpt auth test --output json
```

### Windows PowerShell example: script auth
```powershell
$env:FPT_SITE = "https://example.shotgrid.autodesk.com"
$env:FPT_AUTH_MODE = "script"
$env:FPT_SCRIPT_NAME = "openclaw"
$env:FPT_SCRIPT_KEY = "your-script-key"
fpt auth test --output pretty-json
```

### Windows PowerShell example: user-password auth
```powershell
$env:FPT_SITE = "https://example.shotgrid.autodesk.com"
$env:FPT_AUTH_MODE = "user_password"
$env:FPT_USERNAME = "user@example.com"
$env:FPT_PASSWORD = "your-password"
# Optional: add FPT_AUTH_TOKEN when the site requires 2FA
# $env:FPT_AUTH_TOKEN = "123456"
fpt auth test --output pretty-json
```

### Windows PowerShell example: session-token auth
```powershell
$env:FPT_SITE = "https://example.shotgrid.autodesk.com"
$env:FPT_AUTH_MODE = "session_token"
$env:FPT_SESSION_TOKEN = "your-session-token"
fpt auth test --output pretty-json
```

## Safety notes
- Prefer release archives over remote-script execution in agent workflows.
- Prefer environment variables over raw credential arguments.
- Treat `FPT_*` as the primary namespace.
- Use `SG_*` only as fallback compatibility inputs.
- Add `FPT_AUTH_TOKEN` when the ShotGrid site requires 2FA.
