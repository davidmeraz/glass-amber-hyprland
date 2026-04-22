#!/bin/bash
# ═══════════════════════════════════════════════════════════
#  Power Menu — Amber Glass Pill
#  Compact power menu for Hyprland
# ═══════════════════════════════════════════════════════════

# Opciones con Nerd Font icons
lock="   Bloquear"
suspend="   Suspender"
logout="   Cerrar sesión"
reboot="   Reiniciar"
shutdown="   Apagar"

options="$lock\n$suspend\n$logout\n$reboot\n$shutdown"

chosen=$(echo -e "$options" | rofi -dmenu \
    -i \
    -theme ~/.config/rofi/powermenu.rasi \
    -p "◈  SESIÓN" \
    -no-fixed-num-lines \
    -selected-row 0)

case "$chosen" in
    "$lock")
        pidof hyprlock || hyprlock
        ;;
    "$suspend")
        systemctl suspend
        ;;
    "$logout")
        hyprctl dispatch exit
        ;;
    "$reboot")
        systemctl reboot
        ;;
    "$shutdown")
        systemctl poweroff
        ;;
esac
