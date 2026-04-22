#!/usr/bin/env zsh

# ============================================================================
# Dynamic Island Manager (EWW Native)
# Manages two windows: island-bar + dynamic-island
# ============================================================================

ISLAND_DIR="${0:A:h}"
EWW_BIN="${HOME}/.local/bin/eww"
EWW_CONFIG="${ISLAND_DIR}/eww"

start() {
    if ! pgrep -f "eww.*daemon" >/dev/null 2>&1; then
        ${EWW_BIN} daemon --config "${EWW_CONFIG}" &!
        sleep 0.5
    fi
    ${EWW_BIN} open island-bar --config "${EWW_CONFIG}"
    ${EWW_BIN} open dynamic-island --config "${EWW_CONFIG}"
}

stop() {
    ${EWW_BIN} close dynamic-island --config "${EWW_CONFIG}" 2>/dev/null
    ${EWW_BIN} close island-bar --config "${EWW_CONFIG}" 2>/dev/null
}

toggle() {
    if ${EWW_BIN} active-windows --config "${EWW_CONFIG}" 2>/dev/null | grep -q "island-bar"; then
        stop
    else
        start
    fi
}

expand() {
    ${EWW_BIN} update island_expanded=true --config "${EWW_CONFIG}"
}

collapse() {
    ${EWW_BIN} update island_expanded=false --config "${EWW_CONFIG}"
}

toggle_expand() {
    local state=$(${EWW_BIN} get island_expanded --config "${EWW_CONFIG}" 2>/dev/null)
    if [[ "$state" == "true" ]]; then
        collapse
    else
        expand
    fi
}

reload() {
    stop; sleep 0.3
    ${EWW_BIN} kill --config "${EWW_CONFIG}" 2>/dev/null; sleep 0.3
    start
}

case "${1}" in
    start)          start ;;
    stop)           stop ;;
    toggle)         toggle ;;
    expand)         expand ;;
    collapse)       collapse ;;
    toggle-expand)  toggle_expand ;;
    reload)         reload ;;
    *)
        echo "Dynamic Island"
        echo "Usage: $(basename $0) {start|stop|toggle|expand|collapse|toggle-expand|reload}"
        ;;
esac
