use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Alignment},
    style::{Color, Style, Modifier},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use std::{
    error::Error,
    io,
    time::{Duration, Instant},
    process::Command as SysCommand,
};
use sysinfo::{System, CpuRefreshKind, RefreshKind};

fn get_monitor_fps() -> String {
    if let Ok(output) = SysCommand::new("sh")
        .arg("-c")
        .arg("xrandr 2>/dev/null | grep '*' | head -1 | awk '{print $2}' | tr -d '*+'")
        .output()
    {
        let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if let Ok(val) = s.parse::<f64>() {
            return format!("{:.0}", val);
        }
    }
    "165".to_string()
}

fn format_bytes(bytes: u64) -> String {
    let gb = bytes as f64 / (1024.0 * 1024.0 * 1024.0);
    if gb >= 1.0 {
        format!("{:.1}G", gb)
    } else {
        let mb = bytes as f64 / (1024.0 * 1024.0);
        format!("{:.0}M", mb)
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;

    let mut sys = System::new_with_specifics(
        RefreshKind::new()
            .with_cpu(CpuRefreshKind::everything())
            .with_memory(sysinfo::MemoryRefreshKind::everything()),
    );

    sys.refresh_cpu_all();
    std::thread::sleep(Duration::from_millis(200));

    let fps_str = get_monitor_fps();
    let tick_rate = Duration::from_millis(500);
    let mut last_tick = Instant::now();

    let accent = (255u8, 215u8, 0u8);
    let dim = Color::Rgb(60, 60, 60);
    let label_fg = Color::Rgb(255, 255, 255);

    loop {
        sys.refresh_cpu_all();
        sys.refresh_memory();

        let ram_used = sys.used_memory();
        let ram_total = sys.total_memory();
        let ram_pct = if ram_total > 0 { (ram_used as f64 / ram_total as f64) * 100.0 } else { 0.0 };

        let cpu_global = sys.global_cpu_usage();
        let per_cpu: Vec<f32> = sys.cpus().iter().map(|c| c.cpu_usage()).collect();
        let cpu_count = per_cpu.len();

        terminal.draw(|f| {
            let area = f.area();

            // Sin borde — diseño limpio
            let outer_block = Block::default()
                .borders(Borders::NONE);

            let inner_area = outer_block.inner(area);
            f.render_widget(outer_block, area);

            let main_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(2), // spacer top
                    Constraint::Length(1), // title
                    Constraint::Length(1), // spacer
                    Constraint::Length(1), // RAM
                    Constraint::Length(1), // spacer
                    Constraint::Length(1), // CPU
                    Constraint::Length(2), // spacer grande
                    Constraint::Length(1), // CORES label
                    Constraint::Length(1), // spacer
                    Constraint::Min(0),   // CPU core grid
                ])
                .split(inner_area);

            // ═══════════════════════════════════════
            // TITLE
            // ═══════════════════════════════════════
            let title = Line::from(vec![
                Span::styled("  ◈ ", Style::default().fg(Color::Rgb(255, 179, 0))), // Icono Ámbar
                Span::styled("SYSTEM", Style::default().fg(Color::Rgb(255, 255, 255)).add_modifier(Modifier::BOLD)), // Título Blanco
            ]);
            f.render_widget(Paragraph::new(vec![title]), main_layout[1]);

            // ═══════════════════════════════════════
            // RAM SECTION
            // ═══════════════════════════════════════
            let ram_used_str = format_bytes(ram_used);
            let ram_total_str = format_bytes(ram_total);
            let ram_free_str = format_bytes(ram_total - ram_used);

            let ram_header = Line::from(vec![
                Span::styled("  RAM ", Style::default().fg(label_fg).add_modifier(Modifier::BOLD)),
                Span::styled(
                    format!("{} / {}  ", ram_used_str, ram_total_str),
                    Style::default().fg(Color::Rgb(255, 179, 0)), // Datos en Ámbar
                ),
                Span::styled(
                    format!("▸ free {}  ", ram_free_str),
                    Style::default().fg(Color::Rgb(150, 150, 150)), // Detalles en gris
                ),
                Span::styled(
                    format!("{:.0}%", ram_pct),
                    Style::default().fg(Color::Rgb(255, 179, 0)).add_modifier(Modifier::BOLD), // Datos en Ámbar
                ),
            ]);

            let ram_widget = Paragraph::new(vec![ram_header]);
            f.render_widget(ram_widget, main_layout[3]);

            // ═══════════════════════════════════════
            // CPU SECTION
            // ═══════════════════════════════════════
            let cpu_header = Line::from(vec![
                Span::styled("  CPU ", Style::default().fg(label_fg).add_modifier(Modifier::BOLD)),
                Span::styled(
                    format!("{:.1}%  ", cpu_global),
                    Style::default().fg(Color::Rgb(255, 179, 0)),
                ),
                Span::styled(
                    format!("▸ {} threads", cpu_count),
                    Style::default().fg(Color::Rgb(150, 150, 150)),
                ),
            ]);

            let cpu_widget = Paragraph::new(vec![cpu_header]);
            f.render_widget(cpu_widget, main_layout[5]);


            // ═══════════════════════════════════════
            // CORES LABEL
            // ═══════════════════════════════════════
            let cores_label = Line::from(vec![
                Span::styled("  CORES ", Style::default().fg(label_fg).add_modifier(Modifier::BOLD)),
                Span::styled(
                    "── real-time load ──",
                    Style::default().fg(Color::Rgb(120, 120, 120)),
                ),
            ]);
            f.render_widget(Paragraph::new(vec![cores_label]), main_layout[7]);

            // ═══════════════════════════════════════
            // CPU CORE GRID
            // ═══════════════════════════════════════
            let grid_area = main_layout[9];
            let cols = 8;
            let rows = (cpu_count + cols - 1) / cols;

            let mut grid_lines: Vec<Line> = Vec::new();

            for row in 0..rows {
                let mut line1 = Vec::new();
                let mut line2 = Vec::new();
                let mut line3 = Vec::new();

                let grid_w = cols as i32 * 5;
                let left_pad = ((grid_area.width as i32 - grid_w) / 2).max(1) as usize;

                for _ in 0..left_pad {
                    line1.push(Span::raw(" "));
                    line2.push(Span::raw(" "));
                    line3.push(Span::raw(" "));
                }

                for col in 0..cols {
                    let idx = row * cols + col;
                    let usage = if idx < cpu_count { per_cpu[idx] } else { 0.0 };

                    let intensity = (usage / 100.0).clamp(0.0, 1.0);
                    let v = (30.0 + 225.0 * intensity) as u8;

                    // Bordes: blanco brillante cuando activo, gris oscuro cuando idle
                    let frame_style = if intensity > 0.2 {
                        Style::default().fg(Color::Rgb(255, 255, 255))
                    } else {
                        Style::default().fg(Color::Rgb(80, 80, 80))
                    };
                    // Relleno: Ámbar puro
                    let fill_style = Style::default().fg(Color::Rgb(255, 179, 0));

                    let fill_char = if intensity > 0.7 {
                        "██"
                    } else if intensity > 0.3 {
                        "▓▓"
                    } else if intensity > 0.1 {
                        "▒▒"
                    } else {
                        "░░"
                    };

                    line1.push(Span::styled("╭──╮ ", frame_style));
                    line2.push(Span::styled("│", frame_style));
                    line2.push(Span::styled(fill_char, fill_style));
                    line2.push(Span::styled("│", frame_style));
                    line2.push(Span::raw(" "));
                    line3.push(Span::styled("╰──╯ ", frame_style));
                }
                grid_lines.push(Line::from(line1));
                grid_lines.push(Line::from(line2));
                grid_lines.push(Line::from(line3));
            }

            let grid_widget = Paragraph::new(grid_lines)
                .alignment(Alignment::Left);
            f.render_widget(grid_widget, grid_area);
        })?;

        if event::poll(Duration::from_millis(5))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') || key.code == KeyCode::Esc {
                    break;
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}
