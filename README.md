# Glass Amber Hyprland Theme

This repository contains the configuration files for a premium, custom-built Hyprland desktop environment featuring a monochrome terminal dashboard, an Amber Glass HUD aesthetic, and a fully interactive Dynamic Island.

## Components

*   **Hyprland Config (`hypr/`)**: Core window manager rules, animations, and golden border definitions.
    *   `hyprland.conf`: The main configuration file.
    *   `scripts/dashboard.sh`: Manages the layout and monochrome styling (`#050505` background, `#ffffff` text) of the terminal widgets.
*   **Dynamic Island (`dynamic-island/`)**: A custom Eww-based top bar and morphing widget.
    *   `island.zsh`: Control script to start/stop the widget.
    *   `eww/`: Contains the `.scss` styles (Amber base, transparency rules) and `.yuck` UI layout.
*   **ZSH Environment (`zsh/`)**: Terminal aesthetics.
    *   `.zshrc`: Custom prompt and `LS_COLORS`/`EZA_COLORS` using the exact MC20 golden hex (`201;168;76`).

## Installation

1.  Copy `hyprland.conf` and `dashboard.sh` to your `~/.config/hypr/` directory.
2.  Copy the `dynamic-island` folder to `~/Proyectos/zsh/dynamic-island/` (or update the paths in `hyprland.conf` and `.zshrc` to match your preferred location).
3.  Copy the contents of `.zshrc` to your own `~/.zshrc`, or use it directly.

## Requirements

*   Hyprland
*   Eww (ElKowars wacky widgets)
*   Kitty terminal
*   ZSH with Oh My Zsh
*   eza (for styled `ls` commands)
*   Playerctl, nmcli, wpctl (for Dynamic Island modules)
