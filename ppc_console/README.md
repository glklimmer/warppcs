# Cheat console for warppcs

## Setup

Build the cheat console with:

```bash
cargo build -p ppc_console --release
```

Add following alias to your shell config (`.bashrc` or `.zshrc`):

```bash
alias ppc="$HOME/path/to/repo/target/release/ppc_console"
```

## Usage

Write `ppc` while the game is running.
