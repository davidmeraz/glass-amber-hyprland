#!/bin/bash
set -euo pipefail
IFS=$'\n\t'
# Script para iniciar widgets con efecto cascada y manejo de Layouts

LAYOUT="${1:-default}"

echo "Cargando configuración de diseño: $LAYOUT"

# === WIDGETS ESTÁTICOS ===

# default (Opción A): Y=65 (+5px inward)
MAP_SIZE="670 275";      MAP_POS="25 65"
DONUT_SIZE="470 275";    DONUT_POS="720 65"
LYRICS_SIZE="670 275";   LYRICS_POS="1220 65"
    
RAMDUMP_SIZE="470 435";  RAMDUMP_POS="25 360"
AUDIOCIRCLE_SIZE="500 250"; AUDIOCIRCLE_POS="700 360"
SPECS_SIZE="470 435";    SPECS_POS="1415 360"

TERMINAL_SIZE="870 170";   TERMINAL_POS="520 630"

VISUALIZER_SIZE="1870 235"; VISUALIZER_POS="25 820"

# (Quitado el código basura temporal)

# Limpiar el dashboard viejo (Ahora sí cerramos la terminal por tu solicitud)
for title in radar calendar candles audiocircle donut net sysmon ramdump specs lyrics nowplaying musica map terminal visualizer; do
    hyprctl dispatch closewindow title:$title > /dev/null 2>&1 || true
done
pkill -f "swapper.sh" >/dev/null 2>&1 || true
sleep 0.5

# Variables de color comunes (Fuerza una paleta monocromática en Kitty)
BW_OPTS="-o foreground=#ffffff -o background=#050505 -o color0=#000000 -o color1=#aaaaaa -o color2=#bbbbbb -o color3=#cccccc -o color4=#dddddd -o color5=#eeeeee -o color6=#ffffff -o color7=#ffffff -o color8=#555555 -o color9=#aaaaaa -o color10=#bbbbbb -o color11=#cccccc -o color12=#dddddd -o color13=#eeeeee -o color14=#ffffff -o color15=#ffffff"

# Generamos un archivo config dinámico para cava en blanco y negro
mkdir -p ~/.config/cava
cat > ~/.config/cava/bw_config <<'EOF'
[color]
background = default
foreground = '#ffffff'
[general]
framerate = 60
EOF

# Comprobar dependencias básicas antes de lanzar ventanas
command -v hyprctl >/dev/null 2>&1 || { echo "hyprctl no encontrado" >&2; exit 1; }
command -v kitty >/dev/null 2>&1 || { echo "kitty no encontrado" >&2; exit 1; }

# --- EMISION DE VENTANAS CON COORDENADAS PERFECTAS AL NACER ---

# Fila 1 (Top)
hyprctl dispatch exec "[workspace 1 silent; size $MAP_SIZE; move $MAP_POS] kitty ${BW_OPTS} --title 'map' -o background_opacity=0.0 -o window_padding_width=20 ~/.local/share/terminal-widgets/target/release/tw-map"
sleep 0.15

hyprctl dispatch exec "[workspace 1 silent; size $DONUT_SIZE; move $DONUT_POS] kitty ${BW_OPTS} --title 'donut' -o font_size=8 -o background_opacity=0.0 -o window_padding_width=0 sh -c '/home/dav3e/.local/bin/donut'"
sleep 0.15

hyprctl dispatch exec "[workspace 1 silent; size $LYRICS_SIZE; move $LYRICS_POS] kitty ${BW_OPTS} --title 'nowplaying' -o background_opacity=0.0 -o window_padding_width=10 /home/dav3e/.local/bin/tw-nowplaying"
sleep 0.15

# Fila 2 (Mitad Abajo - 2 columnas laterales y el círculo de audio en el centro)
hyprctl dispatch exec "[workspace 1 silent; size $RAMDUMP_SIZE; move $RAMDUMP_POS] kitty ${BW_OPTS} --title 'ramdump' -o background_opacity=0.0 -o window_padding_width=20 ~/.local/share/terminal-widgets/target/release/tw-ramdump"
sleep 0.15

hyprctl dispatch exec "[workspace 1 silent; size $AUDIOCIRCLE_SIZE; move $AUDIOCIRCLE_POS] kitty ${BW_OPTS} --title 'audiocircle' -o background_opacity=0.0 -o window_padding_width=20 ~/.local/share/terminal-widgets/target/release/tw-audiocircle"
sleep 0.15

hyprctl dispatch exec "[workspace 1 silent; size $SPECS_SIZE; move $SPECS_POS] kitty ${BW_OPTS} --title 'specs' -o background_opacity=0.0 -o window_padding_width=20 ~/.local/share/terminal-widgets/target/release/tw-life"
sleep 0.15

# Terminal interactiva (debajo del audiocircle)
hyprctl dispatch exec "[workspace 1 silent; size $TERMINAL_SIZE; move $TERMINAL_POS] kitty ${BW_OPTS} --title 'terminal' -o background_opacity=0.0 -o window_padding_width=10"
sleep 0.15

# Fila 3 (Base Completa)
hyprctl dispatch exec "[workspace 1 silent; size $VISUALIZER_SIZE; move $VISUALIZER_POS] kitty ${BW_OPTS} --title 'visualizer' -o background_opacity=0.0 -o window_padding_width=5 ~/.local/share/terminal-widgets/target/release/tw-visualizer"
sleep 0.15
