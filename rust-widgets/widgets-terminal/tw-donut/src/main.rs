use crossterm::terminal;
use std::io::{self, Write};
use std::thread;
use std::time::Duration;

fn main() {
    print!("\x1b[2J\x1b[?25l"); // Limpiar pantalla y ocultar cursor

    let mut a: f32 = 0.0;
    let mut b: f32 = 0.0;
    
    // Rastreamos el último tamaño para limpiar pantalla si cambia
    let mut last_w = 0;
    let mut last_h = 0;

    loop {
        // Obtener tamaño de terminal dinámicamente en cada ciclo
        let (width_u16, height_u16) = terminal::size().unwrap_or((80, 24));
        let width = width_u16 as usize;
        let height = height_u16 as usize;

        if width != last_w || height != last_h {
            print!("\x1b[2J"); // Limpiar pantalla completa si hubo un resize
            last_w = width;
            last_h = height;
        }

        // Limitar a un mínimo para no colapsar la matemática
        let width_clamped = if width < 10 { 10 } else { width };
        let height_clamped = if height < 10 { 10 } else { height };

        let mut b_buf = vec![' '; width_clamped * height_clamped];
        let mut z_buf = vec![0.0f32; width_clamped * height_clamped];

        let half_w = width_clamped as f32 / 2.0;
        let half_h = height_clamped as f32 / 2.0;

        // Escalar dinámicamente manteniendo la proporción del donut
        // Las celdas de terminal suelen ser el doble de altas que anchas
        let scale = half_w.min(half_h * 2.0) * 0.85;
        let x_scale = scale;
        let y_scale = scale / 2.0;

        let mut j: f32 = 0.0;
        while j < 6.28 {
            let mut i: f32 = 0.0;
            while i < 6.28 {
                let c = i.sin();
                let d = j.cos();
                let e = a.sin();
                let f = j.sin();
                let g = a.cos();
                let h = d + 2.0;
                let d2 = 1.0 / (c * h * e + f * g + 5.0);
                let l = i.cos();
                let m = b.cos();
                let n = b.sin();
                let t = c * h * g - f * e;

                let x = (half_w + x_scale * d2 * (l * h * m - t * n)) as i32;
                let y = (half_h + y_scale * d2 * (l * h * n + t * m)) as i32;

                let n_lum = 8.0 * ((f * e - c * d * g) * m - c * d * e - f * g - l * d * n);

                if x >= 0 && x < width_clamped as i32 && y >= 0 && y < height_clamped as i32 {
                    let o = x as usize + width_clamped * (y as usize);
                    if d2 > z_buf[o] {
                        z_buf[o] = d2;
                        let lum_idx = n_lum as i32;
                        let lum_chars = b".,-~:;=!*#$@";
                        let lum = if lum_idx > 0 { lum_idx as usize } else { 0 };
                        let lum = if lum > 11 { 11 } else { lum };
                        b_buf[o] = lum_chars[lum] as char;
                    }
                }

                i += 0.02;
            }
            j += 0.07;
        }

        print!("\x1b[H"); // Cursor al inicio (1,1)

        let mut out = String::with_capacity((width_clamped * height_clamped + height_clamped) * 5);
        let mut current_color = 0; // 0: reset, 1: white, 2: amber
        for y in 0..height_clamped {
            for x in 0..width_clamped {
                let ch = b_buf[x + y * width_clamped];
                if ch == ' ' {
                    if current_color != 0 {
                        out.push_str("\x1b[0m");
                        current_color = 0;
                    }
                    out.push(ch);
                } else {
                    let new_color = match ch {
                        '.' | ',' | '-' | '~' | ':' | ';' => 1, // Blanco brillante para las sombras
                        _ => 2, // Ámbar intenso para las luces
                    };
                    if new_color != current_color {
                        if new_color == 1 {
                            out.push_str("\x1b[38;2;255;255;255m"); // Blanco
                        } else {
                            out.push_str("\x1b[38;2;255;179;0m"); // Ámbar intenso
                        }
                        current_color = new_color;
                    }
                    out.push(ch);
                }
            }
            if current_color != 0 {
                out.push_str("\x1b[0m");
                current_color = 0;
            }
            if y < height_clamped - 1 {
                out.push('\n');
            }
        }

        print!("{}", out);
        io::stdout().flush().unwrap();

        a += 0.04;
        b += 0.02;

        thread::sleep(Duration::from_millis(30));
    }
}
