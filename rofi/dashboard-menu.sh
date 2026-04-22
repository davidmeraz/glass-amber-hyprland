#!/bin/bash

# ─── MC20 Dashboard Control Menu ───

show="  Mostrar Dashboard"
hide="  Ocultar Dashboard"
stress_seq="  Estrés Secuencial"
stress_rand="  Estrés Aleatorio"
stress_stop="  Detener Estrés"

options="$show\n$hide\n$stress_seq\n$stress_rand\n$stress_stop"

chosen=$(echo -e "$options" | rofi -dmenu -i -theme ~/.config/rofi/dashboard-menu.rasi -p "◈  Dashboard")

case "$chosen" in
    "$show")
        ~/.config/hypr/scripts/dashboard.sh default
        ;;
    "$hide")
        for title in radar visualizer candles audiocircle donut net specs lyrics nowplaying musica map terminal launchpad; do
            hyprctl dispatch closewindow title:$title > /dev/null 2>&1 || true
        done
        pkill -f "tw-visualizer" > /dev/null 2>&1 || true
        pkill -f "tw-ramdump" > /dev/null 2>&1 || true
        pkill -f "tw-audiocircle" > /dev/null 2>&1 || true
        pkill -f "tw-map" > /dev/null 2>&1 || true
        pkill -f "tw-nowplaying" > /dev/null 2>&1 || true
        pkill -f "tw-life" > /dev/null 2>&1 || true
        pkill -f "donut" > /dev/null 2>&1 || true
        notify-send "Dashboard" "Widgets ocultos" -t 2000
        ;;
    "$stress_seq")
        killall yes 2>/dev/null
        notify-send "CPU Stress" "Estrés secuencial: 1 núcleo cada 3s" -t 3000
        for i in $(seq 0 15); do
            taskset -c $i yes > /dev/null &
            sleep 3
        done
        sleep 15
        killall yes 2>/dev/null
        notify-send "CPU Stress" "Estrés finalizado" -t 2000
        ;;
    "$stress_rand")
        killall yes 2>/dev/null
        notify-send "CPU Stress" "Estrés aleatorio: 45 segundos" -t 3000
        for round in $(seq 1 30); do
            core=$((RANDOM % 16))
            if [ $((RANDOM % 10)) -lt 6 ]; then
                taskset -c $core yes > /dev/null &
            else
                pid=$(pgrep -x "yes" | shuf -n 1)
                [ -n "$pid" ] && kill $pid 2>/dev/null
            fi
            sleep 1.5
        done
        killall yes 2>/dev/null
        notify-send "CPU Stress" "Estrés aleatorio finalizado" -t 2000
        ;;
    "$stress_stop")
        killall yes 2>/dev/null
        notify-send "CPU Stress" "Estrés detenido" -t 2000
        ;;
esac
