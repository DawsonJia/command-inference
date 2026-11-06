# cmdi

AI-powered shell command fixer. Type an approximate command, get the corrected version.

## Install

```bash
cargo install cmd-infer
```

## Usage

```bash
# Set your API key (or pass it inline: CMDI_KEY=xxx cmdi ...)
export CMDI_KEY="your-key-here"

# Optional: point at an Anthropic-compatible gateway and pick a model
export CMDI_URL="http://localhost:8317"
export CMDI_MODEL="claude-haiku-5-5"

# Simple command (args joined automatically)
cmdi grep too high

# Command with shell metacharacters (wrap in quotes)
cmdi 'zcat file.zip | grep "too high"'
```

`cmdi` prints the corrected command to stdout. Copy and run it yourself.

## Configuration

| Environment Variable | Description | Default |
|---------------------|-------------|---------|
| `CMDI_KEY` | API key, sent as `x-api-key` (required) | - |
| `CMDI_URL` | API base URL; requests go to `$CMDI_URL/v1/messages` | `https://api.anthropic.com` |
| `CMDI_MODEL` | Model ID | `claude-opus-5-5` |

## How it works

Sends your approximate command to the Anthropic Messages API (`POST /v1/messages`) and prints the corrected version. Any gateway that speaks the Messages API works via `CMDI_URL`. No execution, no side effects.
