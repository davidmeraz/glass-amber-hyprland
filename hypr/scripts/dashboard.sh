#!/bin/zsh
set -euo pipefail
IFS=$'\n\t'

# === WIDGETS ESTÁTICOS ===

MAP_SIZE="670 275"
MAP_POS="25 75"

DONUT_SIZE="475 275"
DONUT_POS="720 75"

TERMINAL_SIZE="675 275"
TERMINAL_POS="1220 75"
    
SYSTEM_INFO_SIZE="500 400"
SYSTEM_INFO_POS="25 370"

AUDIO_CIRCLE_SIZE="738 423"
AUDIO_CIRCLE_POS="591 371"

LAUNCHPAD_SIZE="500 400"
LAUNCHPAD_POS="1395 370"

ADN_SIZE="275 235"
ADN_POS="25 830"

VISUALIZER_SIZE="1275 235"
VISUALIZER_POS="325 830"

GAME_LIFE_SIZE="275 235"
GAME_LIFE_POS="1622 830"

for title in radar calendar candles audiocircle audio_circle donut net sysmon ramdump system_info specs game_life lyrics nowplaying musica map terminal visualizer adn; do
    hyprctl dispatch closewindow title:^$title$ > /dev/null 2>&1 || true
done

# Matar procesos en background por si acaso
pkill -f "tw-visualizer" >/dev/null 2>&1 || true
pkill -f "tw-system-info" >/dev/null 2>&1 || true
pkill -f "tw-audiocircle" >/dev/null 2>&1 || true
pkill -f "tw-life" >/dev/null 2>&1 || true
pkill -f "tw-map" >/dev/null 2>&1 || true
pkill -f "tw-launchpad" >/dev/null 2>&1 || true
pkill -f "tw-donut" >/dev/null 2>&1 || true
pkill -f "tw-adn" >/dev/null 2>&1 || true
sleep 0.5

# Variables de color comunes (Fuerza una paleta monocromática en Kitty)
BW_OPTS="-o foreground=#ffffff -o background=#050505 -o color0=#000000 -o color1=#aaaaaa -o color2=#bbbbbb -o color3=#cccccc -o color4=#dddddd -o color5=#eeeeee -o color6=#ffffff -o color7=#ffffff -o color8=#555555 -o color9=#aaaaaa -o color10=#bbbbbb -o color11=#cccccc -o color12=#dddddd -o color13=#eeeeee -o color14=#ffffff -o color15=#ffffff"

# Generamos un archivo config dinámico para cava en blanco y negro (si se usa)
mkdir -p ~/.config/cava
cat > ~/.config/cava/bw_config <<'EOF'
[color]
background = default
foreground = '#ffffff'
[general]
framerate = 60
EOF

# Comprobar dependencias básicas
command -v hyprctl >/dev/null 2>&1 || { echo "hyprctl no encontrado" >&2; exit 1; }
command -v kitty >/dev/null 2>&1 || { echo "kitty no encontrado" >&2; exit 1; }

# --- EMISION DE VENTANAS CON COORDENADAS PERFECTAS AL NACER (usando zsh) ---

# Fila 1 (Top)
hyprctl dispatch exec "[workspace 1 silent; size $MAP_SIZE; move $MAP_POS] kitty ${BW_OPTS} --title 'map' -o background_opacity=0.0 -o window_padding_width=20 zsh -c '~/.local/bin/tw-map'"
sleep 0.15

hyprctl dispatch exec "[workspace 1 silent; size $DONUT_SIZE; move $DONUT_POS] kitty ${BW_OPTS} --title 'donut' -o font_size=8 -o background_opacity=0.0 -o window_padding_width=0 zsh -c '~/.local/bin/tw-donut'"
sleep 0.15

# Terminal interactiva movida a la parte superior derecha
hyprctl dispatch exec "[workspace 1 silent; size $TERMINAL_SIZE; move $TERMINAL_POS] kitty ${BW_OPTS} --title 'terminal' -o background_opacity=0.0 -o window_padding_width=10 zsh"
sleep 0.15

# Fila 2 (Mitad Abajo)
hyprctl dispatch exec "[workspace 1 silent; size $SYSTEM_INFO_SIZE; move $SYSTEM_INFO_POS] kitty ${BW_OPTS} --title 'system_info' -o background_opacity=0.0 -o window_padding_width=20 zsh -c '~/.local/bin/tw-system-info'"
sleep 0.15

hyprctl dispatch exec "[workspace 1 silent; size $AUDIO_CIRCLE_SIZE; move $AUDIO_CIRCLE_POS] kitty ${BW_OPTS} --title 'audio_circle' -o background_opacity=0.0 -o window_padding_width=20 zsh -c '~/.local/bin/tw-audiocircle'"
sleep 0.15

hyprctl dispatch exec "[workspace 1 silent; size $GAME_LIFE_SIZE; move $GAME_LIFE_POS] kitty ${BW_OPTS} --title 'game_life' -o background_opacity=0.0 -o window_padding_width=20 zsh -c '~/.local/bin/tw-life'"
sleep 0.15

# Launchpad ocupando el antiguo lugar de la terminal
hyprctl dispatch exec "[workspace 1 silent; size $LAUNCHPAD_SIZE; move $LAUNCHPAD_POS] kitty ${BW_OPTS} --title 'launchpad' -o background_opacity=0.0 -o window_padding_width=10 zsh -c '~/.local/bin/tw-launchpad'"
sleep 0.15

# Fila 3 (Base Completa dividida en 3)
hyprctl dispatch exec "[workspace 1 silent; size $ADN_SIZE; move $ADN_POS] kitty ${BW_OPTS} --title 'adn' -o background_opacity=0.0 -o window_padding_width=20 zsh -c '~/.local/bin/tw-adn'"
sleep 0.15

hyprctl dispatch exec "[workspace 1 silent; size $VISUALIZER_SIZE; move $VISUALIZER_POS] kitty ${BW_OPTS} --title 'visualizer' -o background_opacity=0.0 -o window_padding_width=5 zsh -c '~/.local/bin/tw-visualizer'"
sleep 0.15
