# tokidex

[![Crates.io](https://img.shields.io/crates/v/tokidex.svg)](https://crates.io/crates/tokidex)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![macOS only](https://img.shields.io/badge/platform-macOS-lightgrey.svg)](https://github.com/michaelmjhhhh/tokidex)

`tokidex` is a macOS terminal UI for inspecting local Codex token usage.

It reads local Codex state only: no network calls, no Codex auth token access,
and no billing guesses.

## Highlights

- View Codex token usage by session.
- Switch between today, last 7 days, and all local history.
- Inspect input, cached input, output, reasoning output, and total tokens when
  local JSONL details are available.
- Search sessions by title, model, cwd, or id.
- Refresh in place from a small terminal UI.

## Data Sources

`tokidex` resolves the Codex home directory in this order:

1. `--codex-home <path>`
2. `$CODEX_HOME`
3. `~/.codex`

It reads `state_5.sqlite` for session summaries and totals, then parses each
session `rollout_path` JSONL file for the latest `token_count` event. When JSONL
details are missing or malformed, the UI falls back to the SQLite total.

## Usage

Install from crates.io:

```sh
cargo install tokidex
tokidex --range today
```

`cargo install` places the binary in `~/.cargo/bin`. If `tokidex` is not found
after installation, add Cargo's bin directory to your shell `PATH`:

```sh
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.zshrc
source ~/.zshrc
tokidex --range today
```

Update an existing install:

```sh
cargo install tokidex --force
```

Use privacy mode before sharing screenshots or streaming:

```sh
tokidex --privacy --range all
```

Privacy mode hides full session titles, local paths, rollout JSONL paths, full
session ids, and rate limit details. Token counts and model names remain visible.

Remove an old local/path install, then reinstall from crates.io:

```sh
cargo uninstall tokidex
cargo install tokidex
```

Run without installing:

```sh
cargo run -- --range today
cargo run -- --range week
cargo run -- --range all
cargo run -- --codex-home ~/.codex --range all
```

Install from a local checkout for development:

```sh
cargo install --path .
tokidex --range today
```

## Safety

`tokidex` is a local read-only viewer. It reads Codex's local SQLite and JSONL
state files, never reads `auth.json`, and does not send usage data anywhere.
For public screenshots, use `--privacy`.

## Keys

- `q`: quit
- `Up/Down` or `k/j`: move selection
- `/`: search
- `Esc`: clear search
- `d`: today
- `w`: week
- `a`: all
- `r`: refresh

## Verification

```sh
cargo test
cargo fmt --check
cargo clippy -- -D warnings
```
