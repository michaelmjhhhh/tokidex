# tokidex

`tokidex` is a macOS-only terminal UI for inspecting local Codex token usage.

It reads local Codex state only. It does not call the network, read Codex auth
tokens, or estimate billing cost.

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

Update an existing install:

```sh
cargo install tokidex --force
```

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
