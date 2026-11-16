use std::fs;
use std::path::PathBuf;

fn autostart_path() -> PathBuf {
    let base = std::env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .ok()
        .filter(|p| p.is_absolute())
        .unwrap_or_else(|| {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
            PathBuf::from(home).join(".config")
        });
    base.join("autostart").join("eye-saver.desktop")
}

pub fn set_autostart(enabled: bool) {
    let path = autostart_path();
    if enabled {
        let exe = std::env::current_exe()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| "eye-saver".into());
        let content = format!(
            "[Desktop Entry]\nType=Application\nName=Eye Saver\nComment=Eye break reminders\nExec={exe}\nIcon=eye-saver\nTerminal=false\nCategories=Utility;\nX-GNOME-Autostart-enabled=true\n"
        );
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::write(&path, content);
    } else {
        let _ = fs::remove_file(&path);
    }
}
