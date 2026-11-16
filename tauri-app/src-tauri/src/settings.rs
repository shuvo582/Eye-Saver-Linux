use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Reminders {
    pub stretch: bool,
    pub water: bool,
}

impl Default for Reminders {
    fn default() -> Self {
        Self { stretch: true, water: true }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub breaks_enabled: bool,
    pub break_duration_minutes: u32,
    pub break_interval_minutes: u32,
    pub enable_skipping: bool,
    pub disable_skipping_seconds: u32,
    pub micro_pauses_enabled: bool,
    pub micro_pause_interval_minutes: u32,
    pub micro_pause_duration_seconds: u32,
    pub micro_pauses_sound: bool,
    pub blue_filter_enabled: bool,
    pub blue_filter_intensity: u32,
    pub grayscale_enabled: bool,
    pub invert_enabled: bool,
    pub overlay_enabled: bool,
    pub overlay_opacity: f64,
    pub overlay_color: String,
    pub auto_set_max_brightness: bool,
    pub prevent_sleeping: bool,
    pub autostart: bool,
    pub reset_timers_after_idle: bool,
    pub idle_reset_minutes: u32,
    pub reminders: Reminders,
    pub last_break_at: u64,
    pub last_micro_at: u64,
}

impl Default for Settings {
    fn default() -> Self {
        let now = now_ms();
        Self {
            breaks_enabled: true,
            break_duration_minutes: 20,
            break_interval_minutes: 20,
            enable_skipping: true,
            disable_skipping_seconds: 0,
            micro_pauses_enabled: true,
            micro_pause_interval_minutes: 5,
            micro_pause_duration_seconds: 20,
            micro_pauses_sound: true,
            blue_filter_enabled: false,
            blue_filter_intensity: 40,
            grayscale_enabled: false,
            invert_enabled: false,
            overlay_enabled: false,
            overlay_opacity: 0.25,
            overlay_color: "#0b1e3a".into(),
            auto_set_max_brightness: false,
            prevent_sleeping: false,
            autostart: false,
            reset_timers_after_idle: true,
            idle_reset_minutes: 5,
            reminders: Reminders::default(),
            last_break_at: now,
            last_micro_at: now,
        }
    }
}

pub fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

pub fn config_dir() -> PathBuf {
    let base = std::env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .ok()
        .filter(|p| p.is_absolute())
        .unwrap_or_else(|| {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
            PathBuf::from(home).join(".config")
        });
    base.join("eye-saver-linux")
}

pub fn settings_path() -> PathBuf {
    config_dir().join("settings.json")
}

pub fn load() -> Settings {
    let p = settings_path();
    if let Ok(raw) = fs::read_to_string(&p) {
        if let Ok(mut s) = serde_json::from_str::<Settings>(&raw) {
            // merge is handled by serde default on missing fields when full struct;
            // for partial files, re-parse with defaults under it
            if s.break_interval_minutes == 0 {
                s = Settings::default();
            }
            return s;
        }
    }
    Settings::default()
}

pub fn save(settings: &Settings) -> std::io::Result<()> {
    let dir = config_dir();
    fs::create_dir_all(&dir)?;
    let json = serde_json::to_string_pretty(settings)?;
    fs::write(settings_path(), json)
}

pub fn pick_reminder(s: &Settings) -> String {
    let mut opts: Vec<&str> = Vec::new();
    if s.reminders.stretch {
        opts.push("Sitting too long is not healthy. Do some stretching.");
    }
    if s.reminders.water {
        opts.push("Drink more water!");
    }
    if opts.is_empty() {
        opts.push("Time for a break — rest your eyes.");
    }
    opts[(now_ms() as usize) % opts.len()].to_string()
}
