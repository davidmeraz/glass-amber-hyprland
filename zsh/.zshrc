
# Ejecucion 

#screen(){
#    swww img /home/dav3e/Wallpapers/novitec-maserati-3840x2160-17171.jpg --transition-type center
#}

# 1. Inicio rápido (Powerlevel10k deshabilitado)
# --- MC20 CUSTOM PROMPT ---
autoload -Uz vcs_info
precmd() { vcs_info }
zstyle ':vcs_info:git:*' formats ' %F{#555555}git:(%F{#C9A84C}%b%F{#555555})%f'
PROMPT='%F{#555555}╭─%F{#C9A84C}❖%f %F{#C9A84C}%n%f %F{#555555}en%f %B%F{white}%1~%f%b${vcs_info_msg_0_}
%F{#555555}╰─%F{#C9A84C}❯%f '

# 2. Ruta de Oh My Zsh
export ZSH="$HOME/.oh-my-zsh"

# 3. Tema y Plugins
ZSH_THEME="" # Tema en blanco, el PROMPT está definido arriba
plugins=(git zsh-autosuggestions zsh-syntax-highlighting)

source $ZSH/oh-my-zsh.sh

# 4. CONFIGURACIÓN DE COLORES MC20 (LS_COLORS)
# 4. CONFIGURACIÓN DE COLORES MC20 (LS_COLORS)
# Carpetas: Dorado MC20 (Tanto Ícono como Texto) | Archivos: Gris Claro
export LS_COLORS="di=38;2;201;168;76:fi=38;2;220;220;220:ex=38;5;196:ln=38;5;117:*.tsx=38;5;33:*.ts=38;5;111:*.js=38;5;208:*.jsx=38;5;208:*.json=38;5;255:*.md=38;5;248:*.html=38;5;196:*.css=38;5;153:*.py=38;5;82:*.sql=38;5;159:*.sh=38;5;82:*.png=38;5;201:*.jpg=38;5;201:*.svg=38;5;201:*.gitignore=38;5;242:*.dockerfile=38;5;33"

# 5. CONFIGURACIÓN DE COLORES EZA (EZA_COLORS)
# Se silencia la "ensalada de frutas" en Permisos/Usuarios | di = Dorado MC20
export EZA_COLORS="ur=38;5;239:uw=38;5;239:ux=38;5;239:ue=38;5;239:gr=38;5;239:gw=38;5;239:gx=38;5;239:tr=38;5;239:tw=38;5;239:tx=38;5;239:da=38;5;255:sn=38;5;244:sb=38;5;244:uu=38;2;201;168;76:un=38;2;201;168;76:gu=38;5;239:gn=38;5;239:xx=38;5;239:di=38;2;201;168;76"

# 6. MEJORAS DE AUTOCOMPLETADO (TAB)
zstyle ':completion:*' list-colors "${(s.:.)LS_COLORS}"
zstyle ':completion:*' menu select

# 7. ALIAS DEFINITIVOS
alias ls='eza --icons --group-directories-first'
alias ll='eza -l --icons --group-directories-first'
alias la='eza -a --icons --group-directories-first'
alias lla='eza -la --icons --group-directories-first'
alias tree='eza --tree --icons'
alias kt='kitty'
# Funciones de reload para soportar layouts interactivos
function reload-dash() {
    # Usar setsid para que el script no se muera cuando la terminal que lo lanzó se cierre!
    setsid ~/.config/hypr/scripts/dashboard.sh "${1:-default}" > /dev/null 2>&1 &
}

function reload() {
    echo "\e[38;2;201;168;76m◈\e[0m Recargando Nav-bar (Dynamic Island)..."
    ~/Proyectos/zsh/dynamic-island/island.zsh reload
    
    echo "\e[38;2;201;168;76m◈\e[0m Aplicando reglas de Hyprland..."
    hyprctl reload > /dev/null 2>&1

    hyprctl dispatch focuswindow title:terminal > /dev/null 2>&1 || true
    hyprctl dispatch closewindow title:audiocircle > /dev/null 2>&1 || true
    bash /home/dav3e/.config/hypr/scripts/dashboard.sh "${1:-default}"

    echo "\e[38;2;201;168;76m◈\e[0m Recargando configuración de ZSH..."
    source ~/.zshrc

    echo "\e[32m✔ ¡Sistema recargado con éxito!\e[0m"
}

# Ocultar todos los widgets del dashboard (los manda a un workspace especial invisible)
# La terminal se reposiciona centrada debajo del Maserati MC20
function hide() {
    for w in calendar audiocircle donut net specs musica nowplaying map; do
        hyprctl dispatch movetoworkspacesilent special:hidden,title:$w > /dev/null 2>&1 || true
    done
    # Reposicionar terminal: Centrada y ancha debajo del carro (1920-1200)/2=360
    hyprctl dispatch resizewindowpixel exact 1200 300,title:terminal > /dev/null 2>&1 || true
    hyprctl dispatch movewindowpixel exact 360 720,title:terminal > /dev/null 2>&1 || true
    # Limpiar la terminal del dashboard
    hyprctl dispatch focuswindow title:terminal > /dev/null 2>&1 || true
    sleep 0.1
    hyprctl dispatch sendshortcut CONTROL,L,title:terminal > /dev/null 2>&1 || true
    echo "Dashboard oculto."
}

# Mostrar todos los widgets del dashboard con reload p3 para efecto cascada
function show() {
    reload-dash default
    echo "Dashboard visible."
}

# Corrección de typos frecuentes
alias cleara='clear'
alias cleare='clear'
alias clar='clear'

# 8. Cargar P10k (Desactivado por estética MC20)
# [[ ! -f ~/.p10k.zsh ]] || source ~/.p10k.zsh


export NVM_DIR="$HOME/.nvm"
[ -s "$NVM_DIR/nvm.sh" ] && \. "$NVM_DIR/nvm.sh"  # This loads nvm
[ -s "$NVM_DIR/bash_completion" ] && \. "$NVM_DIR/bash_completion"  # This loads nvm bash_completion

# Alias y atajos personalizados para npm según lo solicitaste
npm() {
  case $1 in
    i)
      command npm install "${@:2}" ;;
    st)
      command npm start "${@:2}" ;;
    rd)
      command npm run dev "${@:2}" ;;
    D)
      command npm install --save-dev "${@:2}" ;;
    *)
      command npm "$@" ;;
  esac
}
export PATH="$HOME/.local/bin:$PATH"
export OLLAMA_API_KEY="ollama-local"
export OLLAMA_BASE_URL="http://127.0.0.1:11434"
export EDITOR=antigravity
export VISUAL=antigravity

# Claude Code Config
export ANTHROPIC_API_KEY="your-api-key-here"
export CLAUDE_CODE_MODEL="claude-3-opus-20240229"
# End Claude Code Config
