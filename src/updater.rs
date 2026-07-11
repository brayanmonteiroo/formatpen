use appimageupdate::Updater;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

const CONFIG_FILE: &str = "update.conf";
const AUTO_UPDATE_KEY: &str = "auto_update";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutoUpdatePref {
    NotAsked,
    Enabled,
    Disabled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdateOutcome {
    NotApplicable,
    UpToDate,
    Updated { path: PathBuf },
    Failed(String),
}

pub fn is_appimage() -> bool {
    std::env::var("APPIMAGE").is_ok()
}

pub fn appimage_path() -> Option<PathBuf> {
    std::env::var("APPIMAGE").ok().map(PathBuf::from)
}

pub fn updates_disabled() -> bool {
    matches!(
        std::env::var("FORMATPEN_NO_UPDATE").as_deref(),
        Ok("1") | Ok("true") | Ok("yes")
    ) || matches!(
        std::env::var("APPIMAGE_UPDATE_DISABLE").as_deref(),
        Ok("1") | Ok("true") | Ok("yes")
    )
}

fn config_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("FORMATPEN_CONFIG_DIR") {
        return PathBuf::from(dir);
    }
    std::env::var("HOME")
        .map(|home| home.into())
        .unwrap_or_else(|_| PathBuf::from("/tmp"))
        .join(".config/com.formatpen.FormatPen")
}

fn config_path() -> PathBuf {
    config_dir().join(CONFIG_FILE)
}

pub fn load_auto_update_pref() -> AutoUpdatePref {
    let path = config_path();
    let Ok(content) = fs::read_to_string(&path) else {
        return AutoUpdatePref::NotAsked;
    };

    for line in content.lines() {
        let line = line.trim();
        if let Some(value) = line.strip_prefix(&format!("{AUTO_UPDATE_KEY}=")) {
            return match value.trim() {
                "true" | "1" | "yes" => AutoUpdatePref::Enabled,
                "false" | "0" | "no" => AutoUpdatePref::Disabled,
                _ => AutoUpdatePref::NotAsked,
            };
        }
    }

    AutoUpdatePref::NotAsked
}

pub fn save_auto_update_pref(enabled: bool) -> Result<(), String> {
    let dir = config_dir();
    fs::create_dir_all(&dir).map_err(|e| format!("Não foi possível criar {dir:?}: {e}"))?;

    let path = config_path();
    let mut file =
        fs::File::create(&path).map_err(|e| format!("Não foi possível gravar {path:?}: {e}"))?;
    writeln!(file, "{AUTO_UPDATE_KEY}={enabled}")
        .map_err(|e| format!("Não foi possível gravar preferência: {e}"))?;
    Ok(())
}

pub fn check_and_apply_update(appimage: &Path) -> UpdateOutcome {
    if !is_appimage() || updates_disabled() {
        return UpdateOutcome::NotApplicable;
    }

    let progress = Arc::new(Mutex::new((0_u64, 0_u64)));
    let progress_cb = progress.clone();

    let updater = match Updater::new(appimage)
        .map(|u| u.overwrite(true))
        .and_then(|builder| {
            Ok(builder.progress_callback(move |done, total| {
                if let Ok(mut slot) = progress_cb.lock() {
                    *slot = (done, total);
                }
            }))
        }) {
        Ok(u) => u,
        Err(e) => return UpdateOutcome::Failed(e.to_string()),
    };

    match updater.check_for_update() {
        Ok(false) => return UpdateOutcome::UpToDate,
        Ok(true) => {}
        Err(e) => return UpdateOutcome::Failed(e.to_string()),
    }

    match updater.perform_update() {
        Ok((path, stats)) => {
            if let Some(backup) = stats.backup_path {
                let _ = fs::remove_file(backup);
            }
            UpdateOutcome::Updated { path }
        }
        Err(e) => UpdateOutcome::Failed(e.to_string()),
    }
}

pub fn restart_appimage(appimage: &Path) -> Result<(), String> {
    std::process::Command::new(appimage)
        .spawn()
        .map_err(|e| format!("Não foi possível reiniciar o FormatPen: {e}"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};

    fn test_lock() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|e| e.into_inner())
    }

    fn with_temp_config<F: FnOnce()>(f: F) {
        let _guard = test_lock();
        let temp = std::env::temp_dir().join(format!("formatpen-updater-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp);
        fs::create_dir_all(&temp).expect("temp config dir");
        std::env::set_var("FORMATPEN_CONFIG_DIR", &temp);
        f();
        std::env::remove_var("FORMATPEN_CONFIG_DIR");
        let _ = fs::remove_dir_all(&temp);
    }

    #[test]
    fn updates_disabled_respeita_env() {
        std::env::set_var("FORMATPEN_NO_UPDATE", "1");
        assert!(updates_disabled());
        std::env::remove_var("FORMATPEN_NO_UPDATE");

        std::env::set_var("APPIMAGE_UPDATE_DISABLE", "true");
        assert!(updates_disabled());
        std::env::remove_var("APPIMAGE_UPDATE_DISABLE");
    }

    #[test]
    fn pref_ausente_e_not_asked() {
        with_temp_config(|| {
            assert_eq!(load_auto_update_pref(), AutoUpdatePref::NotAsked);
        });
    }

    #[test]
    fn salva_e_carrega_pref() {
        with_temp_config(|| {
            save_auto_update_pref(true).unwrap();
            assert_eq!(load_auto_update_pref(), AutoUpdatePref::Enabled);

            save_auto_update_pref(false).unwrap();
            assert_eq!(load_auto_update_pref(), AutoUpdatePref::Disabled);
        });
    }

    #[test]
    fn check_sem_appimage_retorna_not_applicable() {
        std::env::remove_var("APPIMAGE");
        let outcome = check_and_apply_update(Path::new("/tmp/fake.AppImage"));
        assert_eq!(outcome, UpdateOutcome::NotApplicable);
    }
}
