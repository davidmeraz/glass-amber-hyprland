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
    io,
    time::{Duration, Instant},
};
use rand::Rng;

struct Blip {
    x: f64,
    y: f64,
    life: f64,
    discovered: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;

    let mut angle: f64 = 0.0;
    let mut blips: Vec<Blip> = Vec::new();
    let mut rng = rand::thread_rng();

    // Precálculo de los anillos del radar de MUY ALTA DEFINICIÓN
    let grid_circles: Vec<Vec<(f64, f64)>> = (1..=10) // 10 anillos concéntricos
        .map(|i| {
            let r = i as f64 * 13.0; // Distancia súper cerrada entre anillos
            let mut points = Vec::new();
            for s in 0..=72 { // 72 vértices por anillo para lucir como círculos perfectos
                let a = (s as f64) * std::f64::consts::PI * 2.0 / 72.0;
                points.push((r * a.cos(), r * a.sin()));
            }
            points
        })
        .collect();

    let tick_rate = Duration::from_millis(30);
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|f| {
            let canvas = Canvas::default()
                .block(Block::default().borders(Borders::NONE))
                // Braille proporciona la mejor resolución para las líneas del radar
                .marker(Marker::Braille)
                .x_bounds([-195.0, 195.0]) // Al tener -195 y -140, ensanchamos significativamente el círculo en la terminal
                .y_bounds([-140.0, 140.0])
                .paint(|ctx| {
                    // Dibujar anillos
                    for circle in &grid_circles {
                        for i in 0..circle.len() - 1 {
                            ctx.draw(&Line {
                                x1: circle[i].0,
                                y1: circle[i].1,
                                x2: circle[i + 1].0,
                                y2: circle[i + 1].1,
                                color: Color::DarkGray,
                            });
                        }
                    }

                    // Cuadrícula táctica ortogonal súper fina/fantasma (casi invisible) sólo para textura
                    for step in (-130..=130).step_by(26) {
                        let c = step as f64;
                        // Líneas Verticales
                        ctx.draw(&Line { x1: c, y1: -130.0, x2: c, y2: 130.0, color: Color::Rgb(15, 15, 15) });
                        // Líneas Horizontales
                        ctx.draw(&Line { x1: -130.0, y1: c, x2: 130.0, y2: c, color: Color::Rgb(15, 15, 15) });
                    }

                    // Ejes centrales (Cruz más amplia y brillante)
                    ctx.draw(&Line {
                        x1: -135.0,
                        y1: 0.0,
                        x2: 135.0,
                        y2: 0.0,
                        color: Color::Gray,
                    });
                    ctx.draw(&Line {
                        x1: 0.0,
                        y1: -135.0,
                        x2: 0.0,
                        y2: 135.0,
                        color: Color::DarkGray,
                    });

                    // Aguja del radar (Sweeper súper fino y cortante de alta definición)
                    for i in 0..3 {
                        let a = angle - (i as f64 * 0.015);
                        let color = if i == 0 {
                            Color::White
                        } else {
                            Color::DarkGray
                        };
                        ctx.draw(&Line {
                            x1: 0.0,
                            y1: 0.0,
                            x2: 128.0 * a.cos(),
                            y2: 128.0 * a.sin(),
                            color,
                        });
                    }

                    // Puntos (Blips)
                    for blip in &blips {
                        if blip.discovered && blip.life > 0.0 {
                            let size = blip.life * 3.5;
                            let color = if blip.life > 0.7 {
                                Color::White
                            } else {
                                Color::DarkGray
                            };
                            // Dibujar como una X pequeña (simulando objetivos hacker)
                            ctx.draw(&Line {
                                x1: blip.x - size,
                                y1: blip.y - size,
                                x2: blip.x + size,
                                y2: blip.y + size,
                                color,
                            });
                            ctx.draw(&Line {
                                x1: blip.x + size,
                                y1: blip.y - size,
                                x2: blip.x - size,
                                y2: blip.y + size,
                                color,
                            });
                        }
                    }
                });
            f.render_widget(canvas, f.area());
        })?;

        // Interfaz de salida
        if event::poll(Duration::from_millis(5))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') {
                    break;
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            angle += 0.06;
            if angle > std::f64::consts::PI * 2.0 {
                angle = 0.0;
            }

            // Aleatoriamente aparecer nuevos objetivos/puntos de red
            if rng.gen_bool(0.04) {
                let r = rng.gen_range(15.0..115.0);
                let t = rng.gen_range(0.0..std::f64::consts::PI * 2.0);
                blips.push(Blip {
                    x: r * t.cos(),
                    y: r * t.sin(),
                    life: 1.5, // Vive bastante tiempo hasta apagarse
                    discovered: false, // Oculto hasta que lo pasa el barrido
                });
            }

            // Verificar si el radar descubre objetivos
            for blip in &mut blips {
                if !blip.discovered {
                    let mut blip_angle = blip.y.atan2(blip.x);
                    if blip_angle < 0.0 {
                        blip_angle += std::f64::consts::PI * 2.0;
                    }

                    let mut curr_angle = angle;
                    if curr_angle < 0.0 {
                        curr_angle += std::f64::consts::PI * 2.0;
                    }

                    // Distancia circular entre ángulos
                    let mut diff = (curr_angle - blip_angle).abs();
                    if diff > std::f64::consts::PI {
                        diff = std::f64::consts::PI * 2.0 - diff;
                    }

                    // Si la aguja pasó cerca, lo iluminamos
                    if diff < 0.15 {
                        blip.discovered = true;
                    }
                }

                // Desteñir gradualmente si ya fue descubierto
                if blip.discovered {
                    blip.life -= 0.015;
                }
            }

            blips.retain(|b| b.life > 0.0);

            last_tick = Instant::now();
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}
