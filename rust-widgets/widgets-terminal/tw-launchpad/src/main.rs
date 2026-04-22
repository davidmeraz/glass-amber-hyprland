use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph},
    Frame, Terminal,
};
use std::{
    env, fs,
    io::{self, BufRead, BufReader},
    process::{Command, Stdio},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use rand::Rng;

const ROWS: usize = 16;
const COLS: usize = 16;
const BANDS: usize = 16;

struct Ripple {
    x: f32,
    y: f32,
    radius: f32,
    intensity: f32,
    color: (u8, u8, u8),
    shape: u8,
}

struct App {
    grid: [[(f32, (u8, u8, u8)); COLS]; ROWS],
    ripples: Vec<Ripple>,
}

impl App {
    fn new() -> App {
        App {
            grid: [[(0.0, (20, 20, 20)); COLS]; ROWS],
            ripples: Vec::new(),
        }
    }

    fn on_tick(&mut self, audio_bands: &[f64; BANDS]) {
        let mut rng = rand::thread_rng();

        // 1. Decaimiento natural
        for ripple in &mut self.ripples {
            ripple.radius += 0.8; // Expansión suave
            ripple.intensity *= 0.88;
        }
        self.ripples.retain(|r| r.intensity > 0.05);

        for y in 0..ROWS {
            for x in 0..COLS {
                self.grid[y][x].0 *= 0.80; // decaimiento general
            }
        }

        // 2. Interacción Musical / Reposo
        let is_silent = audio_bands.iter().all(|&v| v < 0.05);

        if is_silent && rng.gen_bool(0.16) {
            let cx = rng.gen_range(0..COLS) as f32;
            let cy = rng.gen_range(0..ROWS) as f32;
            let color = if rng.gen_bool(0.65) { (255, 179, 0) } else { (255, 255, 255) };
            let shape = rng.gen_range(0..5);
            let (spawn_x, spawn_y) = if rng.gen_bool(0.4) { (7.5, 7.5) } else { (cx, cy) };
            
            self.ripples.push(Ripple { x: spawn_x, y: spawn_y, radius: 0.0, intensity: 1.0, color, shape });
        }

        if !is_silent {
            for x in 0..COLS {
                let level = audio_bands[x] as f32; 
                
                let max_y = (level * ROWS as f32).round() as usize; 
                
                for y in 0..max_y.min(ROWS) {
                    let grid_y = (ROWS - 1) - y; 
                    
                    let target_brightness = level * 1.8; 
                    if target_brightness > self.grid[grid_y][x].0 {
                        self.grid[grid_y][x].0 = target_brightness.min(1.0);
                        // Bajos a la izquierda (ámbar), altos a la derecha (blanco)
                        self.grid[grid_y][x].1 = if x < (COLS / 2) { (255, 179, 0) } else { (255, 255, 255) };
                    }
                }
                
                // Efecto Ripple por bombo
                if x < 4 && level > 0.85 && rng.gen_bool(0.20) {
                    self.ripples.push(Ripple {
                        x: rng.gen_range(0..COLS) as f32,
                        y: rng.gen_range(0..ROWS) as f32,
                        radius: 0.0,
                        intensity: level,
                        color: (255, 179, 0),
                        shape: rng.gen_range(0..5),
                    });
                }
            }
        }

        // 3. Aplicar luces de las olas a la matriz
        for ripple in &self.ripples {
            for y in 0..ROWS {
                for x in 0..COLS {
                    let dx = x as f32 - ripple.x;
                    let dy = y as f32 - ripple.y;
                    
                    let dist = match ripple.shape {
                        0 => (dx * dx + dy * dy).sqrt(),
                        1 => dx.abs().max(dy.abs()),    
                        2 => dy.abs(),                  
                        3 => dx.abs(),                  
                        _ => dx.abs() + dy.abs(),       
                    };
                    
                    if (dist - ripple.radius).abs() < 1.8 { // Más grueso para grid más grande
                        if ripple.intensity > self.grid[y][x].0 {
                            self.grid[y][x].0 = ripple.intensity;
                            self.grid[y][x].1 = ripple.color;
                        }
                    }
                }
            }
        }
    }
}

fn main() -> io::Result<()> {
    let cava_cfg_path = env::temp_dir().join("tw_launchpad.conf");
    let cava_cfg = format!(
        "[general]\nframerate = 60\nbars = {}\nautosens = 1\novershoot = 0\n\n[output]\nmethod = raw\nraw_target = /dev/stdout\ndata_format = ascii\nascii_max_range = 1000\n\n[smoothing]\nmonstercat = 1\nwaves = 0\nnoise_reduction = 10\ngravity = 120\n",
        BANDS
    );
    let _ = fs::write(&cava_cfg_path, cava_cfg);

    let mut cava = Command::new("cava")
        .arg("-p")
        .arg(&cava_cfg_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("cava debe estar instalado");

    let stdout_pipe = cava.stdout.take().expect("No se pudo conectar stdout de cava");
    let audio_state = Arc::new(Mutex::new([0.0f64; BANDS]));
    let audio_state_writer = audio_state.clone();

    std::thread::spawn(move || {
        let reader = BufReader::new(stdout_pipe);
        for line in reader.lines().flatten() {
            let vals: Vec<f64> = line
                .split(';')
                .filter_map(|s| s.parse::<f64>().ok())
                .map(|v| v / 1000.0)
                .collect();
            if vals.len() >= BANDS {
                if let Ok(mut st) = audio_state_writer.lock() {
                    for i in 0..BANDS { st[i] = vals[i]; }
                }
            }
        }
    });

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    let tick_rate = Duration::from_millis(40); // Animación más fluida
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|f| ui(f, &app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') || key.code == KeyCode::Esc || key.code == KeyCode::Char('c') {
                    break;
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            let current_bands = {
                let st = audio_state.lock().unwrap();
                *st
            };
            app.on_tick(&current_bands);
            last_tick = Instant::now();
        }
    }

    let _ = cava.kill();
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}

fn ui(f: &mut Frame, app: &App) {
    let size = f.area();
    
    // Altura exacta: 16 filas para los 16 anillos
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(ROWS as u16), 
            Constraint::Min(0),
        ])
        .split(size);
        
    let middle = layout[1];
    let mut lines = Vec::new();
    
    let pad_width = 3; // Ocupa 3 caracteres " ◯ "
    let w = middle.width as i32;
    let padding_w = (w - (COLS as i32 * pad_width)).max(0) / 2;
    
    for y in 0..ROWS {
        let mut span_1 = Vec::new(); 
        
        for _ in 0..padding_w {
            span_1.push(Span::raw(" "));
        }
        
        for x in 0..COLS {
            let (brightness, col) = app.grid[y][x];
            
            // Color de anillo inactivo
            let mut r = 60.0; 
            let mut g = 60.0;
            let mut b = 60.0;
            
            if brightness > 0.02 {
                r = r + (col.0 as f32 - r) * brightness.min(1.0);
                g = g + (col.1 as f32 - g) * brightness.min(1.0);
                b = b + (col.2 as f32 - b) * brightness.min(1.0);
            }
            
            let style = Style::default().fg(Color::Rgb(r as u8, g as u8, b as u8));
            
            let char_circle = if brightness > 0.75 {
                " ⬤ " // Círculo brillante relleno
            } else if brightness > 0.4 {
                " ◉ " // Círculo medio relleno
            } else if brightness > 0.15 {
                " ◌ " // Borde punteado
            } else {
                " ◯ " // Anillo de base
            };
            
            span_1.push(Span::styled(char_circle, style));
        }
        
        lines.push(Line::from(span_1));
    }
    
    let p = Paragraph::new(lines).alignment(Alignment::Left).block(Block::default());
    f.render_widget(p, middle);
}
