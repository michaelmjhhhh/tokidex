#!/usr/bin/env bash
set -euo pipefail

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "tokidex is macOS-only." >&2
  exit 1
fi

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"

if [[ -n "${CARGO:-}" ]]; then
  cargo_bin="$CARGO"
elif command -v cargo >/dev/null 2>&1; then
  cargo_bin="$(command -v cargo)"
elif [[ -x "$HOME/.cargo/bin/cargo" ]]; then
  cargo_bin="$HOME/.cargo/bin/cargo"
else
  echo "Cargo was not found." >&2
  echo "Install Rust first: https://rustup.rs" >&2
  exit 1
fi

echo "Installing tokidex from $script_dir"
"$cargo_bin" install --path "$script_dir" --force

installed_bin="$HOME/.cargo/bin/tokidex"
if [[ -x "$installed_bin" ]]; then
  echo
  echo "tokidex installed:"
  echo "  $installed_bin --range today"
else
  echo
  echo "tokidex was installed by Cargo. If it is not on PATH, check Cargo's install output above."
fi

case ":$PATH:" in
  *":$HOME/.cargo/bin:"*)
    echo
    echo "Run it with:"
    echo "  tokidex --range today"
    ;;
  *)
    echo
    echo "Add Cargo binaries to PATH to run tokidex directly:"
    echo '  echo '\''export PATH="$HOME/.cargo/bin:$PATH"'\'' >> ~/.zshrc'
    echo "  source ~/.zshrc"
    echo
    echo "For now, run:"
    echo "  $installed_bin --range today"
    ;;
esac
