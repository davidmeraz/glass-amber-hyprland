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

fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut out = io::stdout();
    execute!(out, EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(out))?;

    let start_time = Instant::now();

    loop {
        let elapsed = start_time.elapsed().as_secs_f64();
        // Velocidad de rotación
        let time = elapsed * 1.8; 

        terminal.draw(|f| {
            let area = f.area();
            let width = area.width as f64 * 2.0;
            let height = area.height as f64 * 4.0;
            
            let half_w = width / 2.0;
            let half_h = height / 2.0;

            let canvas = Canvas::default()
                .block(Block::default().borders(Borders::NONE))
                .marker(Marker::Braille)
                .x_bounds([-half_w, half_w])
                .y_bounds([-half_h, half_h])
                .paint(|ctx| {
                    let num_rungs = 14; 
                    
                    // Aseguramos proporciones simétricas perfectas
                    let min_dim = half_w.min(half_h);
                    
                    // La altura de la espiral
                    let helix_height = half_h * 0.85; 
                    
                    // El radio de la espiral es fijo respecto a su altura para mantenerla esbelta y perfecta
                    let helix_radius = helix_height * 0.55; 

                    let mut strand1 = Vec::new();
                    let mut strand2 = Vec::new();
                    let mut rungs = Vec::new();

                    let steps = 300; 
                    let spatial_freq = std::f64::consts::PI * 2.5; // 1.25 vueltas completas

                    // Calcular todos los puntos matemáticos 3D
                    for i in 0..=steps {
                        let t = i as f64 / steps as f64;
                        let y = helix_height - (t * helix_height * 2.0);
                        
                        let angle1 = t * spatial_freq + time;
                        let angle2 = angle1 + std::f64::consts::PI;

                        let x1 = helix_radius * angle1.cos();
                        let z1 = angle1.sin(); // Profundidad (-1 a 1)

                        let x2 = helix_radius * angle2.cos();
                        let z2 = angle2.sin();

                        strand1.push((x1, y, z1));
                        strand2.push((x2, y, z2));

                        if i % (steps / num_rungs) == 0 {
                            rungs.push((x1, x2, y, z1));
                        }
                    }

                    // -- RENDERIZADO CON Z-BUFFER PARA PERFECCIÓN 3D --

                    // 1. Dibujar las partes que están "atrás" primero
                    let color_back = Color::Rgb(120, 120, 120); // Gris/blanco apagado para profundidad
                    for i in 0..steps {
                        if strand1[i].2 < 0.0 {
                            ctx.draw(&Line { x1: strand1[i].0, y1: strand1[i].1, x2: strand1[i+1].0, y2: strand1[i+1].1, color: color_back });
                        }
                        if strand2[i].2 < 0.0 {
                            ctx.draw(&Line { x1: strand2[i].0, y1: strand2[i].1, x2: strand2[i+1].0, y2: strand2[i+1].1, color: color_back });
                        }
                    }

                    // 2. Dibujar los enlaces genéticos (rungs)
                    let color_rung = Color::Rgb(255, 255, 255); // Blanco puro para los enlaces
                    for (x1, x2, y, _z) in &rungs {
                        ctx.draw(&Line {
                            x1: *x1,
                            y1: *y,
                            x2: *x2,
                            y2: *y,
                            color: color_rung,
                        });
                    }

                    // 3. Dibujar las partes que están "al frente" para que se superpongan perfectamente
                    let color_front = Color::Rgb(255, 179, 0); // Ámbar intenso
                    for i in 0..steps {
                        if strand1[i].2 >= 0.0 {
                            ctx.draw(&Line { x1: strand1[i].0, y1: strand1[i].1, x2: strand1[i+1].0, y2: strand1[i+1].1, color: color_front });
                        }
                        if strand2[i].2 >= 0.0 {
                            ctx.draw(&Line { x1: strand2[i].0, y1: strand2[i].1, x2: strand2[i+1].0, y2: strand2[i+1].1, color: color_front });
                        }
                    }
                });
            f.render_widget(canvas, area);
        })?;

        if event::poll(Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') {
                    break;
                }
            }
        }
    }

    disable_raw_mode()?;
    let _ = execute!(terminal.backend_mut(), LeaveAlternateScreen);
    terminal.show_cursor()?;
    Ok(())
}
