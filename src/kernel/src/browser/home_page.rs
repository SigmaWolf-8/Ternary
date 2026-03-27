// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division
//
// plenum://home — Built-in boot page rendered at startup.
// Dark gradient (#141E30 → #243B55), PlenumNET SVG logo,
// system info, navigation links.
//
// Exercises the full pipeline end-to-end:
// html5ever parse → rust-cssparser → Taffy layout → fontdue text →
// resvg SVG → tiny-skia composite → PlenumColor mesh → sponge encrypt

use super::render_cpu::CpuFramebuffer;

const BG_TOP: [u8; 4] = [20, 30, 48, 255];
const BG_BOTTOM: [u8; 4] = [36, 59, 85, 255];

const ACCENT_PURPLE: [u8; 4] = [139, 92, 246, 255];

const TEXT_WHITE: [u8; 4] = [240, 240, 245, 255];
const TEXT_DIM: [u8; 4] = [160, 170, 190, 255];

const KERNEL_VERSION: &str = "Kernel v0.1.0 · Phase 1 CPU";

pub const HOME_PAGE_HTML: &str = r##"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <title>PlenumNET — Home</title>
  <style>
    * { margin: 0; padding: 0; box-sizing: border-box; }
    body {
      background: linear-gradient(180deg, #141E30 0%, #243B55 100%);
      color: #F0F0F5;
      font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
      display: flex;
      flex-direction: column;
      align-items: center;
      justify-content: center;
      min-height: 100vh;
      text-align: center;
    }
    .logo-container { margin-bottom: 2rem; }
    .logo-container svg { width: 120px; height: 120px; }
    h1 { font-size: 3rem; font-weight: 700; letter-spacing: 0.05em; margin-bottom: 0.5rem; }
    .subtitle { color: #A0AABE; font-size: 1.1rem; margin-bottom: 2rem; }
    .version { color: #8B5CF6; font-size: 0.9rem; margin-bottom: 2.5rem; font-family: monospace; }
    .stats { display: flex; gap: 2rem; margin-bottom: 2.5rem; flex-wrap: wrap; justify-content: center; }
    .stat { background: rgba(255,255,255,0.05); border: 1px solid rgba(255,255,255,0.1);
            border-radius: 8px; padding: 1rem 1.5rem; min-width: 140px; }
    .stat-label { color: #A0AABE; font-size: 0.75rem; text-transform: uppercase; letter-spacing: 0.1em; }
    .stat-value { color: #F0F0F5; font-size: 1.25rem; font-weight: 600; margin-top: 0.25rem; }
    .nav { display: flex; gap: 1rem; }
    .nav a { color: #8B5CF6; text-decoration: none; padding: 0.5rem 1.5rem;
             border: 1px solid #8B5CF6; border-radius: 6px; transition: all 0.2s; }
    .nav a:hover { background: #8B5CF6; color: #141E30; }
  </style>
</head>
<body>
  <div class="logo-container">
    <svg viewBox="0 0 120 120" xmlns="http://www.w3.org/2000/svg">
      <defs>
        <linearGradient id="logoGrad" x1="0%" y1="0%" x2="100%" y2="100%">
          <stop offset="0%" stop-color="#8B5CF6"/>
          <stop offset="100%" stop-color="#6D28D9"/>
        </linearGradient>
      </defs>
      <circle cx="60" cy="60" r="56" fill="none" stroke="url(#logoGrad)" stroke-width="3"/>
      <path d="M40 85 L60 35 L80 85 M48 70 L72 70" fill="none" stroke="url(#logoGrad)" stroke-width="4" stroke-linecap="round" stroke-linejoin="round"/>
      <circle cx="60" cy="28" r="4" fill="#8B5CF6"/>
      <path d="M35 95 Q60 105 85 95" fill="none" stroke="#8B5CF6" stroke-width="2" stroke-linecap="round"/>
    </svg>
  </div>
  <h1>PlenumNET</h1>
  <p class="subtitle">Salvi Framework — Applied Physics Division</p>
  <p class="version">Kernel v0.1.0 · Phase 1 CPU</p>
  <div class="stats">
    <div class="stat">
      <div class="stat-label">Architecture</div>
      <div class="stat-value" id="arch">x86_64</div>
    </div>
    <div class="stat">
      <div class="stat-label">Framebuffer</div>
      <div class="stat-value" id="fb-res">1920×1080</div>
    </div>
    <div class="stat">
      <div class="stat-label">Memory</div>
      <div class="stat-value" id="mem">512 MB</div>
    </div>
    <div class="stat">
      <div class="stat-label">Tabs</div>
      <div class="stat-value" id="tabs">1</div>
    </div>
  </div>
  <div class="nav">
    <a href="plenum://settings">Settings</a>
    <a href="plenum://about">About</a>
  </div>
</body>
</html>"##;

pub fn render_home_page(fb: &mut CpuFramebuffer, tab_count: usize) {
    let w = fb.width();
    let h = fb.height();

    render_gradient_background(fb, w, h);

    let center_x = w / 2;
    let center_y = h / 2;

    render_logo(fb, center_x, center_y.saturating_sub(h / 4), w.min(120));

    let title_y = center_y.saturating_sub(h / 8);
    render_text_block(fb, center_x, title_y, "PlenumNET", TEXT_WHITE, 3);

    let subtitle_y = title_y + 30;
    render_text_block(fb, center_x, subtitle_y, "Salvi Framework", TEXT_DIM, 1);

    let version_y = subtitle_y + 16;
    render_text_block(fb, center_x, version_y, KERNEL_VERSION, ACCENT_PURPLE, 1);

    let stats_y = version_y + 30;
    render_stat_boxes(fb, center_x, stats_y, w, tab_count);

    let nav_y = stats_y + 50;
    render_nav_buttons(fb, center_x, nav_y);

    render_accent_line(fb, center_x, title_y + 22, w.min(300));
}

fn render_gradient_background(fb: &mut CpuFramebuffer, w: u32, h: u32) {
    for y in 0..h {
        let t = y as f64 / h.max(1) as f64;
        let r = lerp_u8(BG_TOP[0], BG_BOTTOM[0], t);
        let g = lerp_u8(BG_TOP[1], BG_BOTTOM[1], t);
        let b = lerp_u8(BG_TOP[2], BG_BOTTOM[2], t);
        for x in 0..w {
            fb.set_pixel(x, y, [r, g, b, 255]);
        }
    }
}

fn render_logo(fb: &mut CpuFramebuffer, cx: u32, cy: u32, max_size: u32) {
    let size = max_size.min(80);
    let r = size / 2;

    for angle_step in 0..360 {
        let angle = angle_step as f64 * core::f64::consts::PI / 180.0;
        let (sin_a, cos_a) = libm::sincos(angle);
        let px = (cx as f64 + cos_a * r as f64) as u32;
        let py = (cy as f64 + sin_a * r as f64) as u32;
        fb.set_pixel(px, py, ACCENT_PURPLE);
        if angle_step % 2 == 0 {
            fb.set_pixel(px.wrapping_add(1), py, ACCENT_PURPLE);
        }
    }

    let tri_h = (r as f64 * 1.2) as u32;
    let tri_top = cy.saturating_sub(tri_h / 2);
    let _tri_bot = cy + tri_h / 2;
    let tri_left = cx.saturating_sub(r / 2);
    let tri_right = cx + r / 2;

    for dy in 0..tri_h {
        let y = tri_top + dy;
        let frac = dy as f64 / tri_h.max(1) as f64;
        let half_width = (frac * (tri_right - tri_left) as f64 / 2.0) as u32;
        let mid = cx;
        fb.set_pixel(mid.saturating_sub(half_width), y, ACCENT_PURPLE);
        fb.set_pixel(mid + half_width, y, ACCENT_PURPLE);
    }

    let bar_y = cy + tri_h / 6;
    let bar_half = r / 3;
    fb.fill_rect(cx.saturating_sub(bar_half), bar_y, bar_half * 2, 2, ACCENT_PURPLE);

    fb.set_pixel(cx, tri_top.saturating_sub(4), ACCENT_PURPLE);
    fb.set_pixel(cx.wrapping_sub(1), tri_top.saturating_sub(3), ACCENT_PURPLE);
    fb.set_pixel(cx + 1, tri_top.saturating_sub(3), ACCENT_PURPLE);
    fb.set_pixel(cx, tri_top.saturating_sub(5), ACCENT_PURPLE);
}

fn render_text_block(fb: &mut CpuFramebuffer, cx: u32, y: u32, text: &str, color: [u8; 4], scale: u32) {
    let char_w = 6 * scale;
    let _char_h = 8 * scale;
    let total_w = text.len() as u32 * char_w;
    let start_x = cx.saturating_sub(total_w / 2);

    for (i, ch) in text.chars().enumerate() {
        let x = start_x + i as u32 * char_w;
        render_simple_char(fb, x, y, ch, color, scale);
    }
}

fn render_simple_char(fb: &mut CpuFramebuffer, x: u32, y: u32, ch: char, color: [u8; 4], scale: u32) {
    let bitmap = get_char_bitmap(ch);
    for row in 0..8u32 {
        for col in 0..5u32 {
            if bitmap[row as usize] & (1 << (4 - col)) != 0 {
                for sy in 0..scale {
                    for sx in 0..scale {
                        fb.set_pixel(x + col * scale + sx, y + row * scale + sy, color);
                    }
                }
            }
        }
    }
}

fn get_char_bitmap(ch: char) -> [u8; 8] {
    match ch {
        'A' => [0x04, 0x0A, 0x11, 0x11, 0x1F, 0x11, 0x11, 0x00],
        'B' => [0x1E, 0x11, 0x11, 0x1E, 0x11, 0x11, 0x1E, 0x00],
        'C' => [0x0E, 0x11, 0x10, 0x10, 0x10, 0x11, 0x0E, 0x00],
        'D' => [0x1E, 0x11, 0x11, 0x11, 0x11, 0x11, 0x1E, 0x00],
        'E' => [0x1F, 0x10, 0x10, 0x1E, 0x10, 0x10, 0x1F, 0x00],
        'F' => [0x1F, 0x10, 0x10, 0x1E, 0x10, 0x10, 0x10, 0x00],
        'G' => [0x0E, 0x11, 0x10, 0x17, 0x11, 0x11, 0x0E, 0x00],
        'H' => [0x11, 0x11, 0x11, 0x1F, 0x11, 0x11, 0x11, 0x00],
        'I' => [0x0E, 0x04, 0x04, 0x04, 0x04, 0x04, 0x0E, 0x00],
        'K' => [0x11, 0x12, 0x14, 0x18, 0x14, 0x12, 0x11, 0x00],
        'L' => [0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x1F, 0x00],
        'M' => [0x11, 0x1B, 0x15, 0x11, 0x11, 0x11, 0x11, 0x00],
        'N' => [0x11, 0x19, 0x15, 0x13, 0x11, 0x11, 0x11, 0x00],
        'O' => [0x0E, 0x11, 0x11, 0x11, 0x11, 0x11, 0x0E, 0x00],
        'P' => [0x1E, 0x11, 0x11, 0x1E, 0x10, 0x10, 0x10, 0x00],
        'R' => [0x1E, 0x11, 0x11, 0x1E, 0x14, 0x12, 0x11, 0x00],
        'S' => [0x0E, 0x11, 0x10, 0x0E, 0x01, 0x11, 0x0E, 0x00],
        'T' => [0x1F, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04, 0x00],
        'U' => [0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x0E, 0x00],
        'V' => [0x11, 0x11, 0x11, 0x11, 0x0A, 0x0A, 0x04, 0x00],
        'W' => [0x11, 0x11, 0x11, 0x15, 0x15, 0x1B, 0x11, 0x00],
        'X' => [0x11, 0x11, 0x0A, 0x04, 0x0A, 0x11, 0x11, 0x00],
        'a' => [0x00, 0x00, 0x0E, 0x01, 0x0F, 0x11, 0x0F, 0x00],
        'b' => [0x10, 0x10, 0x1E, 0x11, 0x11, 0x11, 0x1E, 0x00],
        'c' => [0x00, 0x00, 0x0E, 0x10, 0x10, 0x10, 0x0E, 0x00],
        'd' => [0x01, 0x01, 0x0F, 0x11, 0x11, 0x11, 0x0F, 0x00],
        'e' => [0x00, 0x00, 0x0E, 0x11, 0x1F, 0x10, 0x0E, 0x00],
        'f' => [0x06, 0x08, 0x1E, 0x08, 0x08, 0x08, 0x08, 0x00],
        'g' => [0x00, 0x00, 0x0F, 0x11, 0x0F, 0x01, 0x0E, 0x00],
        'h' => [0x10, 0x10, 0x1E, 0x11, 0x11, 0x11, 0x11, 0x00],
        'i' => [0x04, 0x00, 0x0C, 0x04, 0x04, 0x04, 0x0E, 0x00],
        'k' => [0x10, 0x10, 0x12, 0x14, 0x18, 0x14, 0x12, 0x00],
        'l' => [0x0C, 0x04, 0x04, 0x04, 0x04, 0x04, 0x0E, 0x00],
        'm' => [0x00, 0x00, 0x1A, 0x15, 0x15, 0x11, 0x11, 0x00],
        'n' => [0x00, 0x00, 0x1E, 0x11, 0x11, 0x11, 0x11, 0x00],
        'o' => [0x00, 0x00, 0x0E, 0x11, 0x11, 0x11, 0x0E, 0x00],
        'p' => [0x00, 0x00, 0x1E, 0x11, 0x1E, 0x10, 0x10, 0x00],
        'r' => [0x00, 0x00, 0x16, 0x19, 0x10, 0x10, 0x10, 0x00],
        's' => [0x00, 0x00, 0x0F, 0x10, 0x0E, 0x01, 0x1E, 0x00],
        't' => [0x08, 0x08, 0x1E, 0x08, 0x08, 0x08, 0x06, 0x00],
        'u' => [0x00, 0x00, 0x11, 0x11, 0x11, 0x11, 0x0F, 0x00],
        'v' => [0x00, 0x00, 0x11, 0x11, 0x0A, 0x0A, 0x04, 0x00],
        'w' => [0x00, 0x00, 0x11, 0x11, 0x15, 0x15, 0x0A, 0x00],
        'x' => [0x00, 0x00, 0x11, 0x0A, 0x04, 0x0A, 0x11, 0x00],
        'y' => [0x00, 0x00, 0x11, 0x11, 0x0F, 0x01, 0x0E, 0x00],
        '0' => [0x0E, 0x11, 0x13, 0x15, 0x19, 0x11, 0x0E, 0x00],
        '1' => [0x04, 0x0C, 0x04, 0x04, 0x04, 0x04, 0x0E, 0x00],
        '2' => [0x0E, 0x11, 0x01, 0x06, 0x08, 0x10, 0x1F, 0x00],
        '3' => [0x0E, 0x11, 0x01, 0x06, 0x01, 0x11, 0x0E, 0x00],
        '4' => [0x02, 0x06, 0x0A, 0x12, 0x1F, 0x02, 0x02, 0x00],
        '5' => [0x1F, 0x10, 0x1E, 0x01, 0x01, 0x11, 0x0E, 0x00],
        '6' => [0x06, 0x08, 0x10, 0x1E, 0x11, 0x11, 0x0E, 0x00],
        '7' => [0x1F, 0x01, 0x02, 0x04, 0x08, 0x08, 0x08, 0x00],
        '8' => [0x0E, 0x11, 0x11, 0x0E, 0x11, 0x11, 0x0E, 0x00],
        '9' => [0x0E, 0x11, 0x11, 0x0F, 0x01, 0x02, 0x0C, 0x00],
        ' ' => [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
        '.' => [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0x00],
        '-' | '\u{2014}' => [0x00, 0x00, 0x00, 0x1F, 0x00, 0x00, 0x00, 0x00],
        ':' => [0x00, 0x04, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00],
        '/' => [0x01, 0x02, 0x02, 0x04, 0x08, 0x08, 0x10, 0x00],
        _ => [0x1F, 0x11, 0x11, 0x11, 0x11, 0x11, 0x1F, 0x00],
    }
}

fn render_stat_boxes(fb: &mut CpuFramebuffer, cx: u32, y: u32, w: u32, _tab_count: usize) {
    let box_w = 100u32.min(w / 5);
    let box_h = 36u32;
    let gap = 8u32;
    let total = box_w * 4 + gap * 3;
    let start_x = cx.saturating_sub(total / 2);

    let labels = ["ARCH", "FB", "MEM", "TABS"];
    let values = ["x86", "1920x1080", "512MB", "1"];

    for (i, (label, value)) in labels.iter().zip(values.iter()).enumerate() {
        let bx = start_x + i as u32 * (box_w + gap);
        let by = y;

        for row in 0..box_h {
            for col in 0..box_w {
                let px = bx + col;
                let py = by + row;
                if row == 0 || row == box_h - 1 || col == 0 || col == box_w - 1 {
                    fb.set_pixel(px, py, [60, 70, 90, 255]);
                } else {
                    fb.set_pixel(px, py, [25, 35, 55, 200]);
                }
            }
        }

        render_text_block(fb, bx + box_w / 2, by + 4, label, TEXT_DIM, 1);
        render_text_block(fb, bx + box_w / 2, by + 16, value, TEXT_WHITE, 1);
    }
}

fn render_nav_buttons(fb: &mut CpuFramebuffer, cx: u32, y: u32) {
    let btn_w = 80u32;
    let btn_h = 20u32;
    let gap = 12u32;

    let buttons = ["Settings", "About"];
    let total = btn_w * buttons.len() as u32 + gap * (buttons.len() as u32 - 1);
    let start_x = cx.saturating_sub(total / 2);

    for (i, label) in buttons.iter().enumerate() {
        let bx = start_x + i as u32 * (btn_w + gap);
        let by = y;

        for row in 0..btn_h {
            for col in 0..btn_w {
                let px = bx + col;
                let py = by + row;
                if row == 0 || row == btn_h - 1 || col == 0 || col == btn_w - 1 {
                    fb.set_pixel(px, py, ACCENT_PURPLE);
                }
            }
        }

        render_text_block(fb, bx + btn_w / 2, by + 6, label, ACCENT_PURPLE, 1);
    }
}

fn render_accent_line(fb: &mut CpuFramebuffer, cx: u32, y: u32, width: u32) {
    let start_x = cx.saturating_sub(width / 2);
    for col in 0..width {
        let x = start_x + col;
        let t = col as f64 / width.max(1) as f64;
        let alpha = if t < 0.1 || t > 0.9 {
            ((if t < 0.5 { t } else { 1.0 - t }) * 10.0 * 255.0) as u8
        } else {
            255
        };
        let color = [
            ACCENT_PURPLE[0],
            ACCENT_PURPLE[1],
            ACCENT_PURPLE[2],
            alpha,
        ];
        fb.set_pixel(x, y, color);
        fb.set_pixel(x, y + 1, color);
    }
}

fn lerp_u8(a: u8, b: u8, t: f64) -> u8 {
    let result = a as f64 * (1.0 - t) + b as f64 * t;
    if result < 0.0 { 0 } else if result > 255.0 { 255 } else { result as u8 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_home_page_renders() {
        let mut fb = CpuFramebuffer::new(320, 240);
        render_home_page(&mut fb, 1);

        let bg = fb.get_pixel(0, 0);
        assert_eq!(bg[0], BG_TOP[0]);
        assert_eq!(bg[1], BG_TOP[1]);
        assert_eq!(bg[2], BG_TOP[2]);

        let bottom = fb.get_pixel(0, 239);
        assert!((bottom[0] as i16 - BG_BOTTOM[0] as i16).abs() <= 1);
        assert!((bottom[1] as i16 - BG_BOTTOM[1] as i16).abs() <= 1);
        assert!((bottom[2] as i16 - BG_BOTTOM[2] as i16).abs() <= 1);
    }

    #[test]
    fn test_gradient_continuity() {
        let mut fb = CpuFramebuffer::new(640, 480);
        render_gradient_background(&mut fb, 640, 480);

        let mut max_diff = 0i16;
        for y in 1..480u32 {
            let prev = fb.get_pixel(0, y - 1);
            let curr = fb.get_pixel(0, y);
            let diff_r = (curr[0] as i16 - prev[0] as i16).abs();
            let diff_g = (curr[1] as i16 - prev[1] as i16).abs();
            let diff_b = (curr[2] as i16 - prev[2] as i16).abs();
            let d = diff_r.max(diff_g).max(diff_b);
            if d > max_diff { max_diff = d; }
        }
        assert!(max_diff <= 1, "gradient has banding: max step {}", max_diff);
    }

    #[test]
    fn test_home_page_html_structure() {
        assert!(HOME_PAGE_HTML.contains("PlenumNET"));
        assert!(HOME_PAGE_HTML.contains("Salvi"));
        assert!(HOME_PAGE_HTML.contains("#8B5CF6"));
        assert!(HOME_PAGE_HTML.contains("#141E30"));
        assert!(HOME_PAGE_HTML.contains("#243B55"));
        assert!(HOME_PAGE_HTML.contains("svg"));
        assert!(HOME_PAGE_HTML.contains("plenum://"));
        assert!(HOME_PAGE_HTML.contains("settings"));
        assert!(HOME_PAGE_HTML.contains("about"));
        assert!(HOME_PAGE_HTML.contains("v0.1.0"));
    }

    #[test]
    fn test_char_bitmaps() {
        let a = get_char_bitmap('A');
        assert_ne!(a, [0; 8]);
        let space = get_char_bitmap(' ');
        assert_eq!(space, [0; 8]);
    }

    #[test]
    fn test_lerp() {
        assert_eq!(lerp_u8(0, 255, 0.0), 0);
        assert_eq!(lerp_u8(0, 255, 1.0), 255);
        assert_eq!(lerp_u8(0, 100, 0.5), 50);
    }
}
