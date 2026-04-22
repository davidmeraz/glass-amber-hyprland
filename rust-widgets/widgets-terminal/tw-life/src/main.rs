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
        canvas::{Canvas, Points},
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

const WIDTH: usize = 280;
const HEIGHT: usize = 140;

fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;

    let mut grid = vec![vec![false; WIDTH]; HEIGHT];
    let mut rng = rand::thread_rng();

    // Patrón inicial: Pikachu (dibujado pixel a pixel)
    let mask = include_str!("pikachu_mask.txt");
    let mask_lines: Vec<&str> = mask.lines().filter(|l| !l.is_empty()).collect();
    let mask_h = mask_lines.len();
    let mask_w = if mask_h > 0 { mask_lines[0].len() } else { 0 };

    let start_y = if HEIGHT > mask_h { (HEIGHT - mask_h) / 2 } else { 0 };
    let start_x = if WIDTH > mask_w { (WIDTH - mask_w) / 2 } else { 0 };

    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            if y >= start_y && y < start_y + mask_h && x >= start_x && x < start_x + mask_w {
                let my = y - start_y;
                let mx = x - start_x;
                if my < mask_lines.len() && mx < mask_lines[my].len() {
                    grid[y][x] = mask_lines[my].as_bytes()[mx] == b'1';
                }
            } else {
                // Ruido de fondo denso para más interacción celular
                grid[y][x] = rng.gen_bool(0.10);
            }
        }
    }

    let mut next_grid = grid.clone();
    let tick_rate = Duration::from_millis(150);
    let mut last_tick = Instant::now();
    let mut points_white: Vec<(f64, f64)> = Vec::with_capacity(WIDTH * HEIGHT);
    let mut points_amber: Vec<(f64, f64)> = Vec::with_capacity(WIDTH * HEIGHT);

    loop {
        // Pre-calcular vector de coordenadas (mucho más rápido para la GPU de ratatui que dibujar shapes)
        points_white.clear();
        points_amber.clear();
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                if grid[y][x] {
                    // Canvas de ratatui tiene Y = 0 abajo, invertimos la altura para mapeo estándar
                    let px = x as f64;
                    let py = (HEIGHT - 1 - y) as f64;
                    if (x * 7 + y * 13) % 3 == 0 {
                        points_amber.push((px, py));
                    } else {
                        points_white.push((px, py));
                    }
                }
            }
        }

        terminal.draw(|f| {
            let canvas = Canvas::default()
                .block(Block::default().borders(Borders::NONE))
                .marker(Marker::Braille) // Usando Braille cuadriplica la resolución visual de los celdas
                .x_bounds([0.0, WIDTH as f64])
                .y_bounds([0.0, HEIGHT as f64])
                .paint(|ctx| {
                    ctx.draw(&Points {
                        coords: &points_white,
                        color: Color::Rgb(255, 255, 255), // Blanco
                    });
                    ctx.draw(&Points {
                        coords: &points_amber,
                        color: Color::Rgb(255, 179, 0), // Ámbar intenso
                    });
                });
            f.render_widget(canvas, f.area());
        })?;

        if event::poll(Duration::from_millis(5))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') {
                    break;
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            for y in 0..HEIGHT {
                for x in 0..WIDTH {
                    let mut neighbors = 0;
                    for dy in -1..=1 {
                        for dx in -1..=1 {
                            if dx == 0 && dy == 0 {
                                continue;
                            }
                            let nx = x as isize + dx;
                            let ny = y as isize + dy;
                            
                            // Wrapping (mundo toroidal) - hace que el juego nunca acabe
                            let wx = (nx.rem_euclid(WIDTH as isize)) as usize;
                            let wy = (ny.rem_euclid(HEIGHT as isize)) as usize;
                            
                            if grid[wy][wx] {
                                neighbors += 1;
                            }
                        }
                    }

                    if grid[y][x] {
                        next_grid[y][x] = neighbors == 2 || neighbors == 3;
                    } else {
                        next_grid[y][x] = neighbors == 3;
                    }
                }
            }
            grid.clone_from(&next_grid);
            last_tick = Instant::now();
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}
