# Logos GUI Module

The Logos GUI module provides cross-platform graphical user interface capabilities with support for both Wayland and X11 (Xorg) display servers.

## Features

- **Cross-Platform Compatibility**: Works seamlessly on systems using Wayland, X11 (Xorg), or other windowing systems
- **Modern Architecture**: Built with Rust for memory safety and performance
- **Flexible Backend Selection**: Automatically chooses the best available backend (winit, Wayland, or X11)
- **Easy-to-Use API**: Simple API for creating windows, handling events, and managing the application lifecycle

## Supported Backends

The GUI module supports multiple backends with the following priority:

1. **winit** - Cross-platform window creation and event handling (preferred)
2. **Wayland** - Native support for the Wayland display server protocol
3. **X11 (Xorg)** - Support for the X Window System

## Usage

To use the GUI module in your Logos application, enable the appropriate feature flags:

```bash
# Enable all GUI features
cargo run --features="gui"

# Enable specific backend
cargo run --features="winit-gui"
cargo run --features="wayland"
cargo run --features="x11"
```

## Basic Example

```rust
use logos_lang::{Window, Application};

fn main() {
    // Initialize the GUI system
    logos_lang::init_gui().unwrap();

    // Create a new application
    let mut app = Application::new();

    // Create a window
    let window = Window::new("My App", 800, 600).unwrap();

    // Add the window to the application
    app.add_window(window);

    // Run the application
    app.run().unwrap();
}
```

## Feature Flags

- `gui`: Enables all GUI functionality (winit, Wayland, and X11)
- `winit-gui`: Enables winit-based GUI functionality
- `wayland`: Enables Wayland-specific functionality
- `x11`: Enables X11-specific functionality

## Architecture

The GUI module is designed with a pluggable architecture that allows for multiple backends. At runtime, it detects which display server is available and uses the most appropriate backend:

1. If winit is available and working, it's used as the primary backend
2. If Wayland is available (WAYLAND_DISPLAY environment variable set), it's used as a fallback
3. If X11 is available (DISPLAY environment variable set), it's used as a secondary fallback
4. If no native display server is available, it falls back to cross-platform solutions

This ensures that Logos applications can run on a wide variety of systems regardless of the underlying display server technology.