# Ferrite

> **⚠️ Work in Progress (WIP)** - This project is currently under active development and may have incomplete features or bugs.

A terminal-based authentication interface for [greetd](https://git.sr.ht/~kennylevinsen/greetd), built with Rust and [ratatui](https://github.com/ratatui-org/ratatui).

<img width="1898" height="1005" alt="image" src="https://github.com/user-attachments/assets/d2e1f599-6c41-494d-bab8-2d541f248cc8" />


## Overview

Ferrite provides a clean, keyboard-driven TUI for authenticating users and starting desktop sessions through greetd. It automatically discovers available desktop sessions and system users, allowing for a streamlined login experience.

## Features

- 🖥️ Terminal-based user interface
- 🔐 Secure password input with masking
- 📋 Automatic session discovery (Wayland and X11)
- 👤 Automatic user discovery from `/etc/passwd`
- ⌨️ Keyboard navigation (Up/Down arrows, Enter, Esc)
- 🎨 Clean, minimal UI design

## Requirements

- Rust (latest stable version)
- greetd display manager
- A Unix-like system (Linux/BSD)

## Building

```bash
# Clone the repository
git clone <repository-url>
cd ferrite

# Build the project
cargo build --release

# The binary will be at target/release/ferrite
```

## Installation

After building, you can install the binary:

**Using Make (recommended):**
```bash
# Build the project (as regular user, no sudo needed)
make build

# Install binary + systemd tmpfiles configuration (requires sudo)
sudo make install

# After installation, create the directories defined in the tmpfiles config
sudo systemd-tmpfiles --create /etc/tmpfiles.d/ferrite.conf
```

**Manual installation:**
```bash
# Install to /usr/local/bin (requires sudo)
sudo cp target/release/ferrite /usr/local/bin/

# Install systemd tmpfiles configuration (requires sudo)
sudo install -Dm644 systemd-tmpfiles.conf /etc/tmpfiles.d/ferrite.conf
sudo systemd-tmpfiles --create /etc/tmpfiles.d/ferrite.conf

# Or install to your local bin directory (no sudo needed)
cp target/release/ferrite ~/.local/bin/
```

**Uninstall:**
```bash
sudo make uninstall
```

## Configuration

Ferrite automatically discovers:
- **Sessions**: From standard XDG directories:
  - `/usr/share/wayland-sessions`
  - `/usr/share/xsessions`
  - `/usr/local/share/wayland-sessions`
  - `/usr/local/share/xsessions`
  - `/etc/X11/Sessions`
  - `~/.local/share/wayland-sessions`
  - `~/.local/share/xsessions`

- **Users**: From `/etc/passwd` (users with UID 0 or >= 1000, excluding nologin shells)

## Usage

Configure greetd to use ferrite as the greeter. Example greetd configuration:

```toml
[terminal]
vt = 1

[default_session]
command = "ferrite"
user = "greeter"
```

## Controls

- **↑/↓**: Navigate between fields
- **←/→**: Navigate within select fields (session, username)
- **Enter**: Submit authentication


## Project Structure

```
src/
├── main.rs      # Entry point & event loop
├── app.rs       # Application state & logic
├── ui.rs        # UI rendering
├── event.rs     # Event handling
├── auth.rs      # Authentication logic (greetd IPC)
├── util.rs      # Utility functions (session/user discovery)
└── widgets/     # Custom TUI widgets
    ├── mod.rs
    ├── widget.rs
    ├── text.rs
    └── select.rs
```

## Development

```bash
# Run in development mode
cargo run

# Run tests (if any)
cargo test

# Check for issues
cargo clippy

# Format code
cargo fmt
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Contributing

This project is currently in active development. Contributions are welcome, but please note that the codebase may be subject to significant changes.

## Acknowledgments

- Built with [ratatui](https://github.com/ratatui-org/ratatui)
- Uses [greetd-ipc](https://crates.io/crates/greetd_ipc) for authentication
- Inspired by [ly](https://github.com/fairyglade/ly), a TUI display manager for Linux

