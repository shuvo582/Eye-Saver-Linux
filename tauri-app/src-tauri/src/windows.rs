use crate::settings::Settings;
use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

pub const BREAK_LABEL: &str = "break";
pub const EFFECTS_LABEL: &str = "effects";

fn hex_to_rgba(hex: &str, alpha: f64) -> String {
    let h = hex.trim_start_matches('#');
    let full = if h.len() == 3 {
        h.chars().map(|c| format!("{c}{c}")).collect::<String>()
    } else {
        h.to_string()
    };
    let n = u32::from_str_radix(&full, 16).unwrap_or(0x0b1e3a);
    let r = (n >> 16) & 0xff;
    let g = (n >> 8) & 0xff;
    let b = n & 0xff;
    format!("rgba({r},{g},{b},{alpha})")
}

pub fn show_break(app: &AppHandle, reason: &str, remaining: u64, message: &str) -> tauri::Result<()> {
    if let Some(w) = app.get_webview_window(BREAK_LABEL) {
        let _ = w.set_focus();
        let _ = w.emit_to(BREAK_LABEL, "tick", remaining);
        return Ok(());
    }

    let (width, height) = primary_size(app);
    let url = WebviewUrl::App(
        format!("break.html?reason={reason}&total={remaining}&message={}", urlenc(message)).into(),
    );

    let win = WebviewWindowBuilder::new(app, BREAK_LABEL, url)
        .title("Break")
        .inner_size(width, height)
        .position(0.0, 0.0)
        .decorations(false)
        .resizable(false)
        .fullscreen(true)
        .always_on_top(true)
        .skip_taskbar(true)
        .focused(true)
        .visible(true)
        .build()?;

    let _ = win.set_always_on_top(true);
    let _ = win.set_fullscreen(true);
    Ok(())
}

pub fn close_break(app: &AppHandle) {
    if let Some(w) = app.get_webview_window(BREAK_LABEL) {
        let _ = w.close();
    }
}

pub fn break_open(app: &AppHandle) -> bool {
    app.get_webview_window(BREAK_LABEL).is_some()
}

pub fn apply_effects(app: &AppHandle, s: &Settings) {
    let needs = s.blue_filter_enabled
        || s.grayscale_enabled
        || s.invert_enabled
        || s.overlay_enabled;

    if !needs {
        if let Some(w) = app.get_webview_window(EFFECTS_LABEL) {
            let _ = w.close();
        }
        return;
    }

    let mut css = String::new();
    if s.blue_filter_enabled {
        let a = (s.blue_filter_intensity as f64 / 100.0).clamp(0.05, 0.7);
        css.push_str(&format!("background: rgba(255, 120, 20, {a});"));
    } else if s.overlay_enabled {
        let a = s.overlay_opacity.clamp(0.0, 0.8);
        css.push_str(&format!(
            "background: {};",
            hex_to_rgba(&s.overlay_color, a)
        ));
    }

    let mut filters: Vec<String> = Vec::new();
    if s.grayscale_enabled {
        filters.push("grayscale(1)".into());
    }
    if s.invert_enabled {
        filters.push("invert(1)".into());
    }
    let backdrop = if filters.is_empty() {
        "none".to_string()
    } else {
        filters.join(" ")
    };

    let html = format!(
        r#"<!DOCTYPE html><html><head><meta charset="utf-8"><style>
html,body{{margin:0;padding:0;width:100%;height:100%;overflow:hidden;pointer-events:none;background:transparent;}}
#fx{{position:fixed;inset:0;{css}}}
#fx2{{position:fixed;inset:0;backdrop-filter:{backdrop};-webkit-backdrop-filter:{backdrop};pointer-events:none;}}
</style></head><body><div id="fx"></div><div id="fx2"></div></body></html>"#,
        css = css,
        backdrop = backdrop
    );

    // Recreate or update effects window
    if let Some(w) = app.get_webview_window(EFFECTS_LABEL) {
        let _ = w.close();
    }

    let (width, height) = screen_bounds(app);
    let url = WebviewUrl::App(format!("effects.html?css={}", urlenc(&css)).into());
    // Simpler: load data via query is limited; use effects.html that reads body styles from query pieces
    let _ = url;
    let html_url = WebviewUrl::App(format!(
        "effects.html?css={}&backdrop={}",
        urlenc(&css),
        urlenc(&backdrop)
    )
    .into());

    let _ = WebviewWindowBuilder::new(app, EFFECTS_LABEL, html_url)
        .title("Eye Saver Effects")
        .inner_size(width, height)
        .position(0.0, 0.0)
        .decorations(false)
        .resizable(false)
        .transparent(true)
        .always_on_top(true)
        .skip_taskbar(true)
        .focused(false)
        .visible(true)
        .build()
        .map(|w| {
            let _ = w.set_always_on_top(true);
        });

    // Fallback if effects.html missing styles — inject via eval
    if let Some(w) = app.get_webview_window(EFFECTS_LABEL) {
        let js = format!(
            "document.getElementById('fx').style.cssText = `{css}`; document.getElementById('fx2').style.backdropFilter = `{backdrop}`;"
        );
        let _ = w.eval(&js);
        let _ = html; // silence
    }
}

fn primary_size(app: &AppHandle) -> (f64, f64) {
    if let Ok(Some(m)) = app.primary_monitor() {
        let s = m.size();
        return (s.width as f64, s.height as f64);
    }
    (1920.0, 1080.0)
}

fn screen_bounds(app: &AppHandle) -> (f64, f64) {
    primary_size(app)
}

fn urlenc(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => out.push(c),
            ' ' => out.push_str("%20"),
            _ => {
                for b in c.to_string().as_bytes() {
                    out.push_str(&format!("%{b:02X}"));
                }
            }
        }
    }
    out
}

/// Called when break window is closed — update last timestamps in caller.
pub fn emit_tick(app: &AppHandle, remaining: u64) {
    if let Some(w) = app.get_webview_window(BREAK_LABEL) {
        let _ = w.emit("tick", remaining);
    }
}
