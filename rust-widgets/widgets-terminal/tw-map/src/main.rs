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
        canvas::{Canvas, Line, Map, MapResolution},
        Block, Borders,
    },
    Terminal,
};
use std::{
    error::Error,
    io,
    time::{Duration, Instant},
};
use chrono::Local;

// ── Ubicación aproximada según offset UTC del sistema ──────────
// Mapea el offset UTC a coordenadas geográficas conocidas
fn timezone_to_coords() -> (f64, f64) {
    let offset_secs = Local::now().offset().local_minus_utc();
    let offset_hours = offset_secs as f64 / 3600.0;

    // Longitud: cada hora de offset ≈ 15° de longitud
    // Latitud: aproximación basada en zonas horarias comunes
    match offset_hours as i32 {
        -12 => (-172.0, 28.0),   // Baker Island
        -11 => (-170.0, -14.0),  // Samoa
        -10 => (-155.5, 19.9),   // Honolulu, Hawaii
        -9  => (-149.9, 61.2),   // Anchorage, Alaska
        -8  => (-118.2, 34.0),   // Los Angeles, California
        -7  => (-110.3, 24.1),   // La Paz, Baja California Sur
        -6  => (-96.8, 32.8),    // Dallas, Texas
        -5  => (-74.0, 40.7),    // New York
        -4  => (-66.9, 10.5),    // Caracas
        -3  => (-43.2, -22.9),   // Rio de Janeiro
        -2  => (-30.0, 38.7),    // Azores
        -1  => (-16.9, 32.6),    // Cape Verde
        0   => (-0.1, 51.5),     // London
        1   => (2.3, 48.9),      // Paris
        2   => (13.4, 52.5),     // Berlin
        3   => (37.6, 55.8),     // Moscow
        4   => (51.4, 35.7),     // Tehran
        5   => (69.2, 41.3),     // Tashkent
        6   => (77.2, 28.6),     // Delhi
        7   => (100.5, 13.8),    // Bangkok
        8   => (116.4, 39.9),    // Beijing
        9   => (139.7, 35.7),    // Tokyo
        10  => (151.2, -33.9),   // Sydney
        11  => (166.5, -22.3),   // Noumea
        12  => (174.8, -41.3),   // Wellington
        _   => (0.0, 0.0),       // Fallback: meridiano de Greenwich
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut out = io::stdout();
    execute!(out, EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(out))?;

    let mut coords = timezone_to_coords();
    let mut last_tz_check = Instant::now();

    let tick_rate = Duration::from_millis(50); // 20 FPS
    let mut last_tick = Instant::now();
    let mut frame: u64 = 0;

    loop {
        // Recalcular ubicación cada 5 segundos (detecta cambios de zona horaria)
        if last_tz_check.elapsed() >= Duration::from_secs(5) {
            coords = timezone_to_coords();
            last_tz_check = Instant::now();
        }
        let (lon, lat) = coords;

        terminal.draw(|f| {
            // Animación del pulso: 3 anillos que se expanden y desvanecen en ciclo
            let t = (frame as f64) * 0.08;

            let canvas = Canvas::default()
                .block(Block::default().borders(Borders::NONE))
                .marker(Marker::Braille)
                .paint(|ctx| {
                    // ── MAPA MUNDIAL ───────────────────────────────────
                    ctx.draw(&Map {
                        resolution: MapResolution::High,
                        color: Color::Rgb(255, 255, 255), // Blanco puro y brillante para que no se vea apagado
                    });

                    // ── PULSO DE RECONOCIMIENTO ───────────────────────
                    // 3 anillos concéntricos que se expanden y desvanecen
                    for ring in 0..3 {
                        let phase = (t + ring as f64 * 2.1) % 6.3;
                        let radius = phase * 4.5; // Expansión progresiva
                        let fade = 1.0 - (phase / 6.3); // Se desvanece al expandirse

                        if fade <= 0.0 { continue; }

                        let brightness = (fade * 255.0) as u8;
                        // Tono ámbar profundo para el pulso: RGB(255, 179, 0)
                        let color = Color::Rgb(brightness, (brightness as f32 * 0.7) as u8, 0);

                        // Dibujar anillo como polígono de 48 lados
                        let segments = 48;
                        for s in 0..segments {
                            let a1 = s as f64 * std::f64::consts::PI * 2.0 / segments as f64;
                            let a2 = (s + 1) as f64 * std::f64::consts::PI * 2.0 / segments as f64;
                            ctx.draw(&Line {
                                x1: lon + radius * a1.cos(),
                                y1: lat + radius * a1.sin(),
                                x2: lon + radius * a2.cos(),
                                y2: lat + radius * a2.sin(),
                                color,
                            });
                        }
                    }

                    // ── PUNTO CENTRAL (siempre visible, pulso suave) ──
                    let core_pulse = (t * 1.5).sin() * 0.3 + 1.0;
                    let core_size = core_pulse;
                    // Cruz central brillante
                    ctx.draw(&Line {
                        x1: lon - core_size, y1: lat,
                        x2: lon + core_size, y2: lat,
                        color: Color::Rgb(255, 179, 0), // Ámbar intenso
                    });
                    ctx.draw(&Line {
                        x1: lon, y1: lat - core_size,
                        x2: lon, y2: lat + core_size,
                        color: Color::Rgb(255, 179, 0),
                    });
                    // X diagonal para dar densidad al punto
                    let diag = core_size * 0.7;
                    ctx.draw(&Line {
                        x1: lon - diag, y1: lat - diag,
                        x2: lon + diag, y2: lat + diag,
                        color: Color::Rgb(255, 179, 0),
                    });
                    ctx.draw(&Line {
                        x1: lon + diag, y1: lat - diag,
                        x2: lon - diag, y2: lat + diag,
                        color: Color::Rgb(255, 179, 0),
                    });
                })
                .x_bounds([-160.0, 160.0])
                .y_bounds([-65.0, 80.0]);

            f.render_widget(canvas, f.area());
        })?;

        if event::poll(Duration::from_millis(5))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') { break; }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            frame += 1;
            last_tick = Instant::now();
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}
