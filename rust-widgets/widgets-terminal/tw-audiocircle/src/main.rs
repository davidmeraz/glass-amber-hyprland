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
        canvas::{Canvas, Line},
        Block, Borders,
    },
    Terminal,
};
use std::{
    error::Error,
    fs::File,
    io::{self, BufRead, BufReader, Write},
    process::{Command, Stdio},
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

const BANDS: usize = 240; // 240 líneas de espectro de audio individuales, resolución ultra-alta
const MAX_BAR: usize = 100;
const SMOOTH: f64 = 0.35; // Menor suavidad = reacciones más rápidas y agresivas frente al bajo
const CAVA_CFG: &str = "/tmp/tw-cava-circle-term.cfg";

fn write_cava_config() -> Result<(), Box<dyn Error>> {
    let cfg = format!(
        "[general]\nbars = {}\nframerate = 60\n[input]\nmethod = pulse\n[output]\nmethod = raw\nraw_target = /dev/stdout\ndata_format = ascii\nascii_max_range = {}\n[color]\n",
        BANDS, MAX_BAR
    );
    let mut file = File::create(CAVA_CFG)?;
    file.write_all(cfg.as_bytes())?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    write_cava_config()?;

    let mut child = Command::new("cava")
        .arg("-p")
        .arg(CAVA_CFG)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("Asegúrate de que 'cava' esté instalado en tu sistema");

    let stdout = child.stdout.take().unwrap();
    let mut reader = BufReader::new(stdout);

    enable_raw_mode()?;
    let mut out = io::stdout();
    execute!(out, EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(out))?;

    // Estado compartido para mover datos desde Cava asíncronamente
    let bands_arc = Arc::new(Mutex::new(vec![0.0; BANDS]));
    let bands_clone = bands_arc.clone();

    // Hilo de captura de sonido
    thread::spawn(move || {
        for line in reader.lines() {
            let line = match line {
                Ok(l) => l,
                Err(_) => break,
            };
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let raw: Vec<f64> = trimmed.split(';').filter_map(|s| s.parse::<f64>().ok()).collect();
            if raw.len() < BANDS {
                continue;
            }

            let mut b = bands_clone.lock().unwrap();
            for i in 0..BANDS {
                // Filtro para relajar los valores extremos
                b[i] = SMOOTH * b[i] + (1.0 - SMOOTH) * raw[i];
            }
        }
    });

    loop {
        terminal.draw(|f| {
            let b = bands_arc.lock().unwrap().clone();

            let canvas = Canvas::default()
                .block(Block::default().borders(Borders::NONE))
                .marker(Marker::Braille) // Usar matriz de puntos precisos
                .x_bounds([-200.0, 200.0]) // Ampliado levemente para achicar el render general
                .y_bounds([-110.0, 110.0])
                .paint(|ctx| {
                    // Calcular el promedio de las frecuencias bajas (Bass) para hacer latir el núcleo de forma agresiva
                    let mut bass_sum = 0.0;
                    for i in 0..20 {
                        bass_sum += b[i];
                    }
                    let pulse = (bass_sum / 20.0) * 0.70; // Latido mucho más agresivo basado en graves
                    let inner_radius = 38.0 + pulse; // Círculo central más pequeño

                    // Dibujar el anillo central palpitante
                    let mut ring_coords = Vec::new();
                    for deg in (0..360).step_by(2) {
                        let rad = (deg as f64).to_radians();
                        ring_coords.push((inner_radius * rad.cos(), inner_radius * rad.sin()));
                    }
                    ctx.draw(&ratatui::widgets::canvas::Points {
                        coords: &ring_coords,
                        color: Color::Rgb(255, 255, 255), // Anillo central blanco puro
                    });

                    for (i, &amp) in b.iter().enumerate() {
                        let angle_deg = (i as f64 / BANDS as f64) * 360.0 - 90.0;
                        let angle_rad = angle_deg.to_radians();

                        // Amplitud base intensificada
                        let bar_len = amp * 1.35; // Rayos exteriores más largos y explosivos

                        let x1 = inner_radius * angle_rad.cos();
                        let y1 = inner_radius * angle_rad.sin();

                        let x2 = (inner_radius + bar_len) * angle_rad.cos();
                        let y2 = (inner_radius + bar_len) * angle_rad.sin();

                        // Calcular punto medio para rayo bicolor
                        let mid_bar_len = bar_len * 0.65; // El 65% interno del rayo será blanco
                        let mx = (inner_radius + mid_bar_len) * angle_rad.cos();
                        let my = (inner_radius + mid_bar_len) * angle_rad.sin();

                        // Base del rayo en Blanco Puro
                        ctx.draw(&Line {
                            x1,
                            y1,
                            x2: mx,
                            y2: my,
                            color: Color::Rgb(255, 255, 255),
                        });

                        // Punta del rayo en Ámbar Intenso
                        ctx.draw(&Line {
                            x1: mx,
                            y1: my,
                            x2,
                            y2,
                            color: Color::Rgb(255, 179, 0),
                        });
                    }
                });
            f.render_widget(canvas, f.area());
        })?;

        if event::poll(Duration::from_millis(15))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') {
                    break;
                }
            }
        }
    }

    let _ = child.kill();
    disable_raw_mode()?;
    let _ = execute!(terminal.backend_mut(), LeaveAlternateScreen);
    terminal.show_cursor()?;
    Ok(())
}