# Rmenu: A Simple dmenu-like Launcher for Windows

[![Build Status](https://img.shields.io/github/actions/workflow/status/SynrgStudio/rmenu/rust.yml?branch=main)](https://github.com/SynrgStudio/rmenu/actions)
[![Latest Release](https://img.shields.io/github/v/release/SynrgStudio/rmenu)](https://github.com/SynrgStudio/rmenu/releases/latest)
[![License](https://img.shields.io/github/license/SynrgStudio/rmenu)](https://github.com/SynrgStudio/rmenu/blob/main/LICENSE)

**Rmenu** is a lightweight and fast application launcher and menu generator for Windows, heavily inspired by `dmenu` from the suckless.org tools. It reads a list of items from standard input or command-line arguments, allows the user to efficiently search and select an item, and prints the selected item to standard output. This makes it a powerful component for scripting and creating custom workflows.

<p align="center">
  <img src="https://raw.githubusercontent.com/SynrgStudio/rmenu/main/screenshots/rmenu_example.png" alt="NEED TO ADD SCREENSHOT :D" width="600"/>
</p>

## Features

*   **`dmenu` Philosophy:** Reads items from `stdin` (one per line) or from the `-e` argument (comma-separated by default), prints selected item to `stdout`. Exits with code 0 on selection, 1 on Escape.
*   **Highly Customizable:**
    *   Colors (background, foreground, selected, border)
    *   Fonts (name, size, weight)
    *   Dimensions (width, height, padding, border width, item height)
    *   Positioning (predefined layouts or detailed x/y coordinates)
*   **Flexible Layouts:**
    *   Predefined layouts: `top-fullwidth`, `bottom-fullwidth`, `center-dialog`, `top-left`, `top-right`, `bottom-left`, `bottom-right`.
    *   Custom positioning using percentages or absolute pixel values.
*   **Command-Line Overrides:** Most configuration options can be overridden via CLI arguments.
*   **Configuration File:** Uses a simple `config.ini` file (located at `%APPDATA%\\rmenu\\config.ini` by default).
*   **Lightweight and Fast:** Built in Rust for performance.
*   **Case Insensitive/Sensitive Search:** Configurable behavior.
*   **Customizable Prompt:** Display a custom prompt text.
*   **Silent Mode:** Suppress all diagnostic output to `stderr`.

## Table of Contents

*   [Installation](#installation)
    *   [From Releases](#from-releases)
    *   [Building from Source](#building-from-source)
*   [Usage](#usage)
    *   [Basic Examples](#basic-examples)
    *   [Command-Line Options](#command-line-options)
*   [Configuration](#configuration)
    *   [`config.ini` File](#configini-file)
    *   [Color Formatting](#color-formatting)
    *   [Position Formatting](#position-formatting)
*   [Scripting Examples (PowerShell)](#scripting-examples-powershell)
*   [Contributing](#contributing)
*   [License](#license)

## Installation

### From Releases

The easiest way to get `rmenu` is to download the latest pre-compiled binary from the [Releases Page](https://github.com/SynrgStudio/rmenu/releases).

1.  Go to the [Releases Page](https://github.com/SynrgStudio/rmenu/releases).
2.  Download the `rmenu.exe` file from the latest release.
3.  Place `rmenu.exe` in a directory that is part of your system's PATH environment variable (e.g., `C:\\Windows`, or a custom tools directory). This will allow you to run `rmenu` from any command prompt.

### Building from Source

If you prefer to build `rmenu` from source, you'll need Rust installed.

1.  **Install Rust:** If you don't have Rust, get it from [rust-lang.org](https://www.rust-lang.org/).
2.  **Clone the repository:**
    ```bash
    git clone https://github.com/SynrgStudio/rmenu.git
    cd rmenu
    ```
3.  **Build:**
    *   For a debug build:
        ```bash
        cargo build
        ```
        The executable will be in `target/debug/rmenu.exe`.
    *   For a release (optimized) build:
        ```bash
        cargo build --release
        ```
        The executable will be in `target/release/rmenu.exe`.
4.  (Optional) Copy the resulting `rmenu.exe` to a directory in your PATH.

## Usage

Rmenu can be invoked from the command line (e.g., PowerShell, CMD).

### Basic Examples

*   **Provide items via `stdin`:**
    ```powershell
    echo "Option 1`nOption 2`nAnother Option" | rmenu.exe
    ```
*   **Provide items via `-e` argument (comma-separated by default):**
    ```powershell
    rmenu.exe -e "Item A,Item B,Item C"
    ```
*   **Use a prompt:**
    ```powershell
    rmenu.exe -p "Choose your destiny:" -e "Path 1,Path 2"
    ```
*   **Use a predefined layout:**
    ```powershell
    Get-ChildItem -File | Select-Object -ExpandProperty Name | rmenu.exe --layout top-fullwidth --prompt "Select a file:"
    ```

### Command-Line Options

You can get a full list of options by running `rmenu.exe --help`:

```
rmenu - A simple dmenu-like launcher for Windows
Usage: rmenu [OPTIONS]

Input Options:
  -e, --elements <LIST>   List of items (delimiter in config.ini, default: ',').
                            If not provided, rmenu reads from stdin (one per line).
  -p, --prompt <TEXT>     Text to display as prompt.

Configuration and Behavior Options:
  -c, --config <PATH>     Path to the configuration file (config.ini).
  -s, --silent            Suppress all error/diagnostic messages (stderr).
  -h, --help              Show this help.

Geometry and Layout Options (override config.ini):
  --layout <NAME>         Apply a predefined layout. Options:
                            custom, top-fullwidth, bottom-fullwidth, center-dialog,
                            top-left, top-right, bottom-left, bottom-right.
                            If 'custom' or omitted, detailed values are used.
  --x-pos <POS>           X position. E.g., '100' (pixels) or 'r0.5' (relative).
  --y-pos <POS>           Y position. E.g., '0' or 'r0.3'.
  --width-percent <FLOAT> Width as a percentage of screen (0.0-1.0).
  --max-width <PX>        Maximum width in pixels.
  --height <PX>           Height of the input bar in pixels.
  --item-height <PX>      Height of each list item in pixels.
  --padding <PX>          Internal padding in pixels.
  --border-width <PX>     Border width in pixels.
```

## Configuration

Rmenu can be extensively customized using a configuration file.

### `config.ini` File

By default, `rmenu` looks for a configuration file at `%APPDATA%\\rmenu\\config.ini`. If it's not found, `rmenu` will use default values and attempt to create a default `config.ini` at that location on its first run (if it has write permissions).

You can specify a custom path to a configuration file using the `-c` or `--config` command-line option.

The `config.ini` file is structured into sections:

```ini
[Colors]
background = #282C34
foreground = #ABB2BF
selected_background = #3A3F4B
selected_foreground = #E6E6E6
border = #21252B

[Dimensions]
# Available layouts: custom, top-fullwidth, bottom-fullwidth, center-dialog, top-left, top-right, bottom-left, bottom-right
default_layout = custom

# The following values are used if default_layout is 'custom' or not defined,
# and if not overridden by command-line arguments.
width_percent = 0.6
max_width = 1000
height = 32
item_height = 28
x_position = r0.5
y_position = r0.3
padding = 8
border_width = 1

[Font]
name = Consolas
size = 15
weight = 400 ; (e.g., 400 for Normal, 700 for Bold)

[Behavior]
case_sensitive = false
instant_selection = false ; (If true, selecting an item with arrow keys and then typing further will immediately confirm the current selection - Not yet implemented)
max_items = 10       ; Maximum number of items to display in the list
element_delimiter = , ; Delimiter for items passed via -e argument
```

**Note on `instant_selection`:** This feature is planned but not yet fully implemented.

### Color Formatting

Colors are specified in hexadecimal RGB format (e.g., `#RRGGBB`).

*   `#282C34` (Dark bluish-gray)
*   `#FF0000` (Red)

### Position Formatting

For `x_position` and `y_position`:

*   **Absolute pixels:** `100` (100 pixels from the left/top edge)
*   **Relative to screen center:** `r0.5` (centers the window along that axis based on screen dimension). `r0.3` would position it at 30% of the screen dimension, centered.

## Scripting Examples (PowerShell)

Rmenu's power comes alive when used in scripts. Here are a few basic examples for PowerShell:

**1. Simple Choice:**

```powershell
$choice = "Yes`nNo`nMaybe" | rmenu.exe --prompt "Make a choice:"
if ($choice) {
    Write-Host "You chose: $choice"
}
```

**2. Launch an Application:**

```powershell
$apps = @{
    "Notepad" = "notepad.exe";
    "Calculator" = "calc.exe"
}
$selection = $apps.Keys | rmenu.exe --prompt "Launch:"
if ($selection -and $apps.ContainsKey($selection)) {
    Start-Process $apps[$selection]
}
```

**3. Select a File from Current Directory:**

```powershell
$selectedFile = Get-ChildItem -File | Select-Object -ExpandProperty Name | rmenu.exe --prompt "Open file:" --layout top-fullwidth
if ($selectedFile) {
    Invoke-Item ".\$selectedFile"
}
```

*(For more detailed PowerShell examples, see `POWERSHELL_EXAMPLES.md`)*

## Contributing

Contributions are welcome! Whether it's bug reports, feature requests, or code contributions:

1.  **Fork the repository.**
2.  **Create a new branch** for your feature or bug fix: `git checkout -b feature/your-feature-name` or `bugfix/issue-fix`.
3.  **Make your changes.**
4.  **Test your changes thoroughly.**
5.  **Commit your changes:** `git commit -am 'Add some feature'`
6.  **Push to the branch:** `git push origin feature/your-feature-name`
7.  **Create a new Pull Request.**

Please try to follow the existing code style and add comments where necessary. If you're adding a new feature, consider adding tests if applicable.