use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    style::Color,
    symbols::Marker,
    widgets::{
        canvas::{Canvas, Line as CanvasLine, Points},
        Block, Borders,
    },
    Terminal,
};
use std::{
    env, fs,
    io::{self, BufRead, BufReader},
    process::{Command, Stdio},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

const BANDS: usize = 240;
const CAVA_MAX: f64 = 1000.0;
const SMOOTH_ATTACK: f64 = 0.25;
const SMOOTH_DECAY: f64 = 0.60;
const PEAK_HOLD_MS: u64 = 280;
const PEAK_GRAVITY: f64 = 0.012;

struct VisualizerState {
    smoothed: Vec<f64>,
    peaks: Vec<f64>,
    peak_vel: Vec<f64>,
    peak_hold: Vec<Instant>,
}

impl VisualizerState {
    fn new() -> Self {
        let now = Instant::now();
        Self {
            smoothed: vec![0.0; BANDS],
            peaks: vec![0.0; BANDS],
            peak_vel: vec![0.0; BANDS],
            peak_hold: vec![now; BANDS],
        }
    }

    fn update(&mut self, new_raw: &[f64]) {
        let now = Instant::now();
        for i in 0..BANDS.min(new_raw.len()) {
            let target = new_raw[i];
            let alpha = if target > self.smoothed[i] {
                SMOOTH_ATTACK
            } else {
                SMOOTH_DECAY
            };
            self.smoothed[i] = alpha * self.smoothed[i] + (1.0 - alpha) * target;

            if self.smoothed[i] > self.peaks[i] {
                self.peaks[i] = self.smoothed[i];
                self.peak_vel[i] = 0.0;
                self.peak_hold[i] = now;
            } else if now.duration_since(self.peak_hold[i]).as_millis() as u64 > PEAK_HOLD_MS {
                self.peak_vel[i] += PEAK_GRAVITY;
                self.peaks[i] = (self.peaks[i] - self.peak_vel[i]).max(0.0);
            }
        }
    }

    fn interpolated(&self, count: usize) -> Vec<f64> {
        let src = &self.smoothed;
        let n = src.len();
        if n == 0 || count == 0 {
            return vec![0.0; count];
        }
        let mut out = Vec::with_capacity(count);
        for i in 0..count {
            let pos = (i as f64 / count as f64) * (n - 1) as f64;
            let idx = pos.floor() as usize;
            let frac = pos - idx as f64;
            let p0 = src[idx.saturating_sub(1)];
            let p1 = src[idx];
            let p2 = src[(idx + 1).min(n - 1)];
            let p3 = src[(idx + 2).min(n - 1)];
            let val = 0.5
                * ((2.0 * p1)
                    + (-p0 + p2) * frac
                    + (2.0 * p0 - 5.0 * p1 + 4.0 * p2 - p3) * frac * frac
                    + (-p0 + 3.0 * p1 - 3.0 * p2 + p3) * frac * frac * frac);
            out.push(val.max(0.0).min(1.0));
        }
        out
    }

    fn interpolated_peaks(&self, count: usize) -> Vec<f64> {
        let src = &self.peaks;
        let n = src.len();
        if n == 0 || count == 0 {
            return vec![0.0; count];
        }
        let mut out = Vec::with_capacity(count);
        for i in 0..count {
            let pos = (i as f64 / count as f64) * (n - 1) as f64;
            let idx = pos.floor() as usize;
            let frac = pos - idx as f64;
            let a = src[idx];
            let b = src[(idx + 1).min(n - 1)];
            out.push(a + (b - a) * frac);
        }
        out
    }
}

fn gradient_color(ratio: f64) -> Color {
    let val = (155.0 + (100.0 * ratio)) as u8;
    Color::Rgb(val, val, val) // Blanco con gradación de intensidad
}

fn glow_color(_ratio: f64) -> Color {
    Color::Rgb(255, 179, 0) // Ámbar intenso para el resplandor
}

fn peak_color() -> Color {
    Color::Rgb(255, 255, 255) // Blanco puro para los picos
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cava_cfg_path = env::temp_dir().join("tw_visualizer_hd.conf");
    let cava_cfg = format!(
        "[general]\nframerate = 60\nbars = {}\nautosens = 1\novershoot = 0\n\n[output]\nmethod = raw\nraw_target = /dev/stdout\ndata_format = ascii\nascii_max_range = {}\n\n[smoothing]\nmonstercat = 1\nwaves = 0\nnoise_reduction = 10\ngravity = 120\n",
        BANDS,
        CAVA_MAX as u32
    );
    fs::write(&cava_cfg_path, cava_cfg)?;

    let mut cava = Command::new("cava")
        .arg("-p")
        .arg(&cava_cfg_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("cava debe estar instalado (pacman -S cava)");

    let stdout_pipe = cava.stdout.take().expect("No se pudo conectar stdout de cava");

    let state = Arc::new(Mutex::new(VisualizerState::new()));
    let state_writer = state.clone();

    std::thread::spawn(move || {
        let reader = BufReader::new(stdout_pipe);
        for line in reader.lines().flatten() {
            let vals: Vec<f64> = line
                .split(';')
                .filter_map(|s| s.parse::<f64>().ok())
                .map(|v| v / CAVA_MAX)
                .collect();
            if vals.len() >= BANDS {
                if let Ok(mut st) = state_writer.lock() {
                    st.update(&vals);
                }
            }
        }
    });

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let tick_rate = Duration::from_millis(16);
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|f| {
            let area = f.area();
            let st = state.lock().unwrap();

            let visual_cols = (area.width as usize) * 2;
            let visual_rows = (area.height as usize) * 4;

            let bars = st.interpolated(visual_cols);
            let peaks = st.interpolated_peaks(visual_cols);

            let x_range = visual_cols as f64;
            let y_range = visual_rows as f64;
            let half_y = y_range / 2.0;

            let canvas = Canvas::default()
                .block(Block::default().borders(Borders::NONE))
                .marker(Marker::Braille)
                .x_bounds([0.0, x_range])
                .y_bounds([0.0, y_range])
                .paint(move |ctx| {
                    ctx.draw(&CanvasLine {
                        x1: 0.0, y1: half_y, x2: x_range, y2: half_y,
                        color: Color::Rgb(40, 40, 40),
                    });

                    for (i, &val) in bars.iter().enumerate() {
                        let x = i as f64;
                        let bar_h = (val * half_y * 1.6).min(half_y - 1.0);
                        if bar_h < 0.5 { continue; }

                        let segments = 6;
                        for s in 0..segments {
                            let frac_lo = s as f64 / segments as f64;
                            let frac_hi = (s + 1) as f64 / segments as f64;
                            let color = gradient_color(frac_hi);

                            ctx.draw(&CanvasLine {
                                x1: x, y1: half_y + bar_h * frac_lo,
                                x2: x, y2: half_y + bar_h * frac_hi, color,
                            });
                            ctx.draw(&CanvasLine {
                                x1: x, y1: half_y - bar_h * frac_lo,
                                x2: x, y2: half_y - bar_h * frac_hi, color,
                            });
                        }
                    }

                    for (i, &val) in bars.iter().enumerate() {
                        let x = i as f64;
                        let bar_h = (val * half_y * 1.7).min(half_y - 0.5);
                        let glow_h = bar_h * 0.15;
                        if glow_h < 0.3 { continue; }
                        let color = glow_color(val);
                        ctx.draw(&CanvasLine {
                            x1: x, y1: half_y + bar_h,
                            x2: x, y2: half_y + bar_h + glow_h, color,
                        });
                        ctx.draw(&CanvasLine {
                            x1: x, y1: half_y - bar_h,
                            x2: x, y2: half_y - bar_h - glow_h, color,
                        });
                    }

                    let mut peak_top: Vec<(f64, f64)> = Vec::new();
                    let mut peak_bot: Vec<(f64, f64)> = Vec::new();
                    for (i, &pk) in peaks.iter().enumerate() {
                        let x = i as f64;
                        let pk_h = (pk * half_y * 1.6).min(half_y - 1.0);
                        if pk_h > 1.0 {
                            peak_top.push((x, half_y + pk_h + 1.5));
                            peak_bot.push((x, half_y - pk_h - 1.5));
                        }
                    }
                    ctx.draw(&Points { coords: &peak_top, color: peak_color() });
                    ctx.draw(&Points { coords: &peak_bot, color: peak_color() });
                });

            f.render_widget(canvas, area);
        })?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc | KeyCode::Char('c') => break,
                    _ => {}
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }

    let _ = cava.kill();
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}
