# tokidex Promotion Kit

## One-Liner

Tokidex is a macOS terminal UI for inspecting local Codex token usage.

## Short Description

Tokidex reads your local Codex state and shows token usage by session in a small
terminal UI. It is local-only: no network calls, no Codex auth token access, and
no billing guesses.

## Links

- GitHub: https://github.com/michaelmjhhhh/tokidex
- crates.io: https://crates.io/crates/tokidex

## Install

```sh
cargo install tokidex
tokidex --range today
```

## X / Twitter

I made `tokidex`, a tiny macOS TUI for inspecting local Codex token usage.

It reads Codex's local SQLite/JSONL state and shows token usage by session:
today, last 7 days, or all history.

Local-only: no network calls, no auth token access.

```sh
cargo install tokidex
tokidex --range today
```

GitHub: https://github.com/michaelmjhhhh/tokidex
crates.io: https://crates.io/crates/tokidex

## Hacker News

Title:

```text
Show HN: Tokidex – a macOS TUI for local Codex token usage
```

Post:

```text
I built Tokidex, a small Rust TUI for inspecting local Codex token usage on macOS.

It reads Codex's local state files and shows usage by session, with today/week/all filters and search. When Codex JSONL token_count events are available, it breaks usage down into input, cached input, output, reasoning output, and total tokens.

It is deliberately local-only: no network calls, no auth token access, and no billing guesses.

Install:
cargo install tokidex
tokidex --range today

GitHub: https://github.com/michaelmjhhhh/tokidex
crates.io: https://crates.io/crates/tokidex
```

## Reddit

```text
I made Tokidex, a small Rust TUI for inspecting local Codex token usage on macOS.

It reads local Codex SQLite/JSONL state and shows token usage by session. You can switch between today, last 7 days, and all history; search sessions; and inspect token breakdowns when local token_count details are available.

The tool is local-only: no network calls, no Codex auth token access, and no billing estimates.

Install:
cargo install tokidex
tokidex --range today

GitHub: https://github.com/michaelmjhhhh/tokidex
crates.io: https://crates.io/crates/tokidex
```

## Discord / Slack

```text
I published Tokidex, a small macOS TUI for local Codex token usage:

cargo install tokidex
tokidex --range today

It is local-only: reads Codex SQLite/JSONL state, no network calls, no auth token access.
https://github.com/michaelmjhhhh/tokidex
```
