# Charify: Image to ASCII Art (Rust)

A command-line tool written in Rust to convert images and GIFs into ASCII art, with options for color, contrast, inversion, and animation looping.

## Features

- Supports common image formats (PNG, JPEG, GIF, etc.).
- Converts static images and animated GIFs.
- Adjustable output width.
- Optional ANSI color output (requires a terminal supporting true color).
- Contrast adjustment.
- Invert mapping (useful for dark backgrounds).
- Option to loop GIF animations.
- Choice between simple high-contrast characters (default) or a more detailed set (used with `--color`).
- Subtle blur applied in color mode to smooth details.

## Prerequisites

- Rust programming language and Cargo: [https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install)
- A terminal that supports ANSI true color (for `--color` mode). Most modern terminals (Windows Terminal, iTerm2, GNOME Terminal, Kitty, etc.) do.

## Building

Clone the repository and build the release executable:

```bash
git clone <your-repo-url> # Replace with actual URL later
cd charify # Updated directory name
cargo build --release
```

The executable will be located at `./target/release/charify`.

## Usage

```bash
./target/release/charify -i <input_path> [OPTIONS]
```

**Arguments:**

- `-i`, `--input <PATH>`: Path to the input image or GIF file.

**Options:**

- `-w`, `--width <WIDTH>`: Width of the output ASCII art in characters [default: 100].
- `--invert`: Invert the character map (useful for images with dark backgrounds).
- `--contrast <FACTOR>`: Adjust contrast (1.0 = normal, >1.0 = higher) [default: 1.0].
- `--color`: Output ASCII art with ANSI colors (uses a more detailed character set and applies a subtle blur).
- `--loop-gif`: Loop GIF animation indefinitely (only applies if input is a GIF).
- `-h`, `--help`: Print help information.
- `-V`, `--version`: Print version information.

**Examples:**

```bash
# Simple grayscale conversion
./target/release/charify -i image.png -w 120

# Color conversion with higher contrast
./target/release/charify -i photo.jpg -w 150 --color --contrast 1.5

# Process a GIF, looping, with inverted mapping for dark background
./target/release/charify -i animation.gif -w 80 --invert --loop-gif

# Process a GIF with color
./target/release/charify -i animation.gif -w 90 --color --loop-gif
```
