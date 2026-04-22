use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::Alignment,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use reqwest::blocking::Client;
use serde_json::Value;
use std::{
    error::Error,
    io,
    process::Command,
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

// ── Offset de sincronización (segundos) ──────────────────────────────────────
// Chromium reporta la canción antes de que suene. Ajustar este valor si las
// letras van adelantadas (+) o atrasadas (-).
const SYNC_OFFSET: f64 = -3.0;

// ── Estructura de una línea sincronizada ─────────────────────────────────────
#[derive(Clone)]
struct LrcLine {
    time_secs: f64,
    text: String,
}

// ── Estado compartido entre hilos ────────────────────────────────────────────
#[derive(Clone)]
struct AppState {
    artist: String,
    title: String,
    lines: Vec<LrcLine>,  // letras sincronizadas
    plain: String,         // fallback sin sync
    status: String,        // mensaje de estado
    has_sync: bool,
    // Posición interna del reloj
    position_secs: f64,
    position_updated_at: Instant,
    is_playing: bool,
    // Para detectar cambio de canción entre hilos
    song_generation: u64,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            artist: String::new(),
            title: String::from("Detectando audio..."),
            lines: vec![],
            plain: String::new(),
            status: String::from("Esperando reproductor..."),
            has_sync: false,
            position_secs: 0.0,
            position_updated_at: Instant::now(),
            is_playing: false,
            song_generation: 0,
        }
    }
}

// ── Limpieza de metadatos de YouTube ─────────────────────────────────────────
fn clean_metadata(raw_artist: &str, raw_title: &str) -> (String, String) {
    let looks_like_channel = raw_artist.ends_with("VEVO")
        || raw_artist.ends_with("Official")
        || raw_artist.contains("Channel")
        || raw_artist.contains("TV")
        || raw_artist.is_empty();

    let strip_suffixes = |s: &str| -> String {
        s.replace("(Official Video)", "")
         .replace("[Official Video]", "")
         .replace("(Lyric Video)", "")
         .replace("[Lyric Video]", "")
         .replace("(Lyrics)", "")
         .replace("[Lyrics]", "")
         .replace("(Official Music Video)", "")
         .replace("[Official Music Video]", "")
         .replace("(Official Audio)", "")
         .replace("[Official Audio]", "")
         .replace("(Audio)", "")
         .replace("[Audio]", "")
         .replace("(Live)", "")
         .replace("[Live]", "")
         .replace("ft.", "feat.")
         .trim()
         .to_string()
    };

    if looks_like_channel {
        if let Some(idx) = raw_title.find(" - ") {
            let artist = raw_title[..idx].trim().to_string();
            let title = strip_suffixes(&raw_title[idx + 3..]);
            return (artist, title);
        }
    }

    // Si el título contiene "Artist - Title", pero el artista ya se conoce,
    // intentar extraer solo la parte del título
    if !raw_artist.is_empty() && raw_title.contains(" - ") {
        if let Some(idx) = raw_title.find(" - ") {
            let maybe_title = strip_suffixes(&raw_title[idx + 3..]);
            if !maybe_title.is_empty() {
                return (raw_artist.to_string(), maybe_title);
            }
        }
    }

    (raw_artist.to_string(), strip_suffixes(raw_title))
}

// ── Obtener canción actual ────────────────────────────────────────────────────
fn get_current_song() -> Option<(String, String)> {
    let out = Command::new("playerctl")
        .args(["metadata", "--format", "{{ artist }}|{{ title }}"])
        .output().ok()?;
    if out.status.success() {
        let text = String::from_utf8_lossy(&out.stdout);
        let parts: Vec<&str> = text.trim().split('|').collect();
        if parts.len() == 2 && !parts[1].is_empty() {
            let (a, t) = clean_metadata(parts[0].trim(), parts[1].trim());
            if !t.is_empty() { return Some((a, t)); }
        }
    }
    None
}

// ── Obtener estado del reproductor ───────────────────────────────────────────
fn get_player_playing() -> bool {
    Command::new("playerctl").arg("status").output().ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().eq_ignore_ascii_case("Playing"))
        .unwrap_or(false)
}

// ── Encodificador URL simple ──────────────────────────────────────────────────
fn urlencode(s: &str) -> String {
    s.chars().map(|c| match c {
        ' ' => "%20".to_string(),
        '&' => "%26".to_string(),
        '+' => "%2B".to_string(),
        '#' => "%23".to_string(),
        '?' => "%3F".to_string(),
        _ => c.to_string(),
    }).collect()
}

// ── Parser de formato LRC [mm:ss.cs] texto ────────────────────────────────────
fn parse_lrc(lrc: &str) -> Vec<LrcLine> {
    let mut lines: Vec<LrcLine> = lrc.lines().filter_map(|line| {
        // Formato: [01:23.45] texto
        if line.starts_with('[') {
            if let Some(end) = line.find(']') {
                let tag = &line[1..end];
                let text = line[end+1..].trim().to_string();
                // Parsear mm:ss.cs
                let parts: Vec<&str> = tag.split(':').collect();
                if parts.len() == 2 {
                    if let (Ok(mm), Ok(rest)) = (
                        parts[0].parse::<f64>(),
                        parts[1].parse::<f64>(),
                    ) {
                        return Some(LrcLine {
                            time_secs: mm * 60.0 + rest,
                            text,
                        });
                    }
                }
            }
        }
        None
    }).collect();
    lines.sort_by(|a, b| a.time_secs.partial_cmp(&b.time_secs).unwrap());
    lines
}

// ── Fetch de letras con prioridad agresiva en syncedLyrics ─────────────────
fn fetch_lyrics(artist: &str, title: &str) -> Option<(bool, String)> {
    let client = Client::builder().timeout(Duration::from_secs(10)).build().ok()?;

    let urls = [
        format!("https://lrclib.net/api/search?artist_name={}&track_name={}", urlencode(artist), urlencode(title)),
        format!("https://lrclib.net/api/search?q={}+{}", urlencode(artist), urlencode(title)),
        format!("https://lrclib.net/api/search?q={}", urlencode(title)),
    ];

    let mut responses = Vec::new();

    for url in &urls {
        if let Ok(resp) = client.get(url).send() {
            if let Ok(arr) = resp.json::<Vec<Value>>() {
                if !arr.is_empty() {
                    responses.push(arr);
                }
            }
        }
    }

    // Prioridad máxima: syncedLyrics en todas las respuestas
    for arr in &responses {
        for track in arr {
            if let Some(lrc) = track.get("syncedLyrics").and_then(|l| l.as_str()) {
                if !lrc.is_empty() {
                    return Some((true, lrc.to_string()));
                }
            }
        }
    }

    // Fallback: plainLyrics
    for arr in &responses {
        for track in arr {
            if let Some(plain) = track.get("plainLyrics").and_then(|l| l.as_str()) {
                if !plain.is_empty() {
                    return Some((false, plain.to_string()));
                }
            }
        }
    }

    None
}

// ── Main ──────────────────────────────────────────────────────────────────────
fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;

    let state = Arc::new(Mutex::new(AppState::default()));
    let state_bg = state.clone();
    let state_pos = state.clone();

    // ── Hilo de red: detecta canciones y busca letras ────────────────────────
    thread::spawn(move || {
        let mut last_song: (String, String) = (String::new(), String::new());
        loop {
            if let Some((artist, title)) = get_current_song() {
                if (artist.clone(), title.clone()) != last_song {
                    {
                        let mut s = state_bg.lock().unwrap();
                        s.artist = artist.clone();
                        s.title = title.clone();
                        s.lines = vec![];
                        s.plain = String::new();
                        s.has_sync = false;
                        s.status = "⟳ Buscando letras...".to_string();
                        // Señal de cambio de canción → resetear reloj
                        s.song_generation += 1;
                        s.position_secs = 0.0;
                        s.position_updated_at = Instant::now();
                    }

                    match fetch_lyrics(&artist, &title) {
                        Some((true, lrc)) => {
                            let parsed = parse_lrc(&lrc);
                            let mut s = state_bg.lock().unwrap();
                            if s.artist == artist && s.title == title {
                                s.has_sync = true;
                                s.lines = parsed;
                                s.status = "♪ Sincronizado".to_string();
                            }
                        }
                        Some((false, plain)) => {
                            let mut s = state_bg.lock().unwrap();
                            if s.artist == artist && s.title == title {
                                s.has_sync = false;
                                s.plain = plain;
                                s.status = "♪ Sin timestamps".to_string();
                            }
                        }
                        None => {
                            let mut s = state_bg.lock().unwrap();
                            if s.artist == artist && s.title == title {
                                s.status = "[ Letras no encontradas ]".to_string();
                            }
                        }
                    }
                    last_song = (artist, title);
                }
            } else {
                if !last_song.0.is_empty() || !last_song.1.is_empty() {
                    let mut s = state_bg.lock().unwrap();
                    *s = AppState::default();
                    last_song = (String::new(), String::new());
                }
            }
            thread::sleep(Duration::from_secs(2));
        }
    });

    // ── Hilo de posición: reloj interno que NO depende de playerctl position ─
    //
    // Chromium MPRIS tiene un bug conocido: playerctl position devuelve un
    // valor congelado. Por eso usamos un reloj interno (Instant) que avanza
    // cuando el reproductor está en "Playing" y se pausa junto con él.
    thread::spawn(move || {
        let mut was_playing = false;
        let mut last_generation: u64 = 0;
        let mut internal_pos: f64 = 0.0;
        let mut clock = Instant::now();

        loop {
            let playing = get_player_playing();

            {
                let mut s = state_pos.lock().unwrap();

                // ¿Cambió la canción? → resetear reloj interno
                if s.song_generation != last_generation {
                    last_generation = s.song_generation;
                    internal_pos = 0.0;
                    clock = Instant::now();
                    was_playing = false;
                }

                if playing && !was_playing {
                    // Reanudó → reiniciar reloj desde posición acumulada
                    clock = Instant::now();
                } else if !playing && was_playing {
                    // Pausó → acumular el tiempo transcurrido
                    internal_pos += clock.elapsed().as_secs_f64();
                    clock = Instant::now();
                }

                // Calcular posición actual
                let current_pos = if playing {
                    internal_pos + clock.elapsed().as_secs_f64()
                } else {
                    internal_pos
                };

                s.position_secs = current_pos;
                s.position_updated_at = Instant::now();
                s.is_playing = playing;
            }

            was_playing = playing;
            thread::sleep(Duration::from_millis(200));
        }
    });

    // ── Loop de render ───────────────────────────────────────────────────────
    loop {
        // Posición interpolada con offset de sincronización
        let pos = {
            let s = state.lock().unwrap();
            let raw = if s.is_playing {
                s.position_secs + s.position_updated_at.elapsed().as_secs_f64()
            } else {
                s.position_secs
            };
            (raw + SYNC_OFFSET).max(0.0)
        };

        terminal.draw(|f| {
            let s = state.lock().unwrap();
            let area = f.area();

            // Header: artista — canción
            let header = if s.artist.is_empty() {
                Line::from(Span::styled(
                    &s.title,
                    Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD),
                ))
            } else {
                Line::from(vec![
                    Span::styled(" ♪ ", Style::default().fg(Color::Rgb(255, 179, 0))), // Icono Ámbar
                    Span::styled(
                        format!("{} — {}", s.artist, s.title),
                        Style::default().fg(Color::Rgb(255, 255, 255)).add_modifier(Modifier::BOLD), // Texto Blanco
                    ),
                ])
            };

            if s.has_sync && !s.lines.is_empty() {
                // ── Modo sincronizado ───────────────────────────────────────
                let current_idx = s.lines.iter().rposition(|l| l.time_secs <= pos).unwrap_or(0);

                // Centrar la línea activa con padding
                let visible_lines = (area.height as usize).saturating_sub(2);
                let half = visible_lines / 2;

                let mut text_lines: Vec<Line> = Vec::new();

                // Padding superior para centrar
                for _ in 0..half {
                    text_lines.push(Line::from(""));
                }

                // Líneas de la letra con highlighting
                for (i, l) in s.lines.iter().enumerate() {
                    let diff = if i >= current_idx { i - current_idx } else { current_idx - i };
                    if i == current_idx {
                        text_lines.push(Line::from(Span::styled(
                            format!("▶ {}", l.text),
                            Style::default().fg(Color::Rgb(255, 179, 0)).add_modifier(Modifier::BOLD), // Línea activa en Ámbar intenso
                        )));
                    } else if diff == 1 {
                        text_lines.push(Line::from(Span::styled(
                            format!("  {}", l.text),
                            Style::default().fg(Color::Rgb(255, 255, 255)), // Siguiente línea en Blanco Puro
                        )));
                    } else if diff == 2 {
                        text_lines.push(Line::from(Span::styled(
                            format!("  {}", l.text),
                            Style::default().fg(Color::DarkGray),
                        )));
                    } else {
                        text_lines.push(Line::from(Span::styled(
                            format!("  {}", l.text),
                            Style::default().fg(Color::Rgb(40, 40, 40)),
                        )));
                    }
                }

                // Padding inferior
                for _ in 0..half {
                    text_lines.push(Line::from(""));
                }

                let scroll = current_idx as u16;

                let para = Paragraph::new(text_lines)
                    .block(Block::default().borders(Borders::NONE).title(header).title_alignment(Alignment::Center))
                    .alignment(Alignment::Center)
                    .scroll((scroll, 0));
                f.render_widget(para, area);

            } else if !s.plain.is_empty() {
                // ── Modo sin sync (texto plano) ─────────────────────────────
                let para = Paragraph::new(s.plain.clone())
                    .block(Block::default().borders(Borders::NONE).title(header).title_alignment(Alignment::Center))
                    .style(Style::default().fg(Color::Gray))
                    .alignment(Alignment::Center);
                f.render_widget(para, area);

            } else {
                // ── Estado / mensaje ────────────────────────────────────────
                let para = Paragraph::new(s.status.clone())
                    .block(Block::default().borders(Borders::NONE).title(header).title_alignment(Alignment::Center))
                    .style(Style::default().fg(Color::DarkGray))
                    .alignment(Alignment::Center);
                f.render_widget(para, area);
            }
        })?;

        if event::poll(Duration::from_millis(30))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') { break; }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}
