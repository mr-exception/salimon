use std::{
    io,
    path::{Path, PathBuf},
};
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Settings {
    pub(crate) version: u32,
    pub(crate) camera_motion: bool,
    pub(crate) condition_audio: bool,
    pub(crate) vignette: bool,
    #[serde(default)]
    pub(crate) master_mute: bool,
    #[serde(default = "default_scene_audio")]
    pub(crate) scene_audio: bool,
    #[serde(default = "default_scene_volume")]
    pub(crate) scene_volume_percent: u8,
}
fn default_scene_audio() -> bool {
    true
}
fn default_scene_volume() -> u8 {
    60
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            version: 1,
            camera_motion: true,
            condition_audio: true,
            vignette: true,
            master_mute: false,
            scene_audio: true,
            scene_volume_percent: 60,
        }
    }
}
pub(super) fn path() -> io::Result<PathBuf> {
    if let Some(p) = std::env::var_os("SALIMON_SETTINGS_FILE") {
        return Ok(p.into());
    }
    // Separate from the active-run directory and survives restart/new runs.
    let run = crate::checkpoints::default_directory().map_err(io::Error::other)?;
    Ok(run
        .parent()
        .ok_or_else(|| io::Error::other("settings parent missing"))?
        .join("settings.json"))
}
pub(super) fn load(path: &Path) -> io::Result<Settings> {
    match std::fs::read(path) {
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(Settings::default()),
        Err(e) => Err(e),
        Ok(bytes) => {
            if bytes.len() > 4096 {
                return Err(io::Error::other("settings file too large"));
            }
            let s: Settings = serde_json::from_slice(&bytes).map_err(io::Error::other)?;
            if s.version != 1 || s.scene_volume_percent > 100 {
                return Err(io::Error::other(
                    "unsupported settings version or audio volume",
                ));
            }
            Ok(s)
        }
    }
}
pub(super) fn save(path: &Path, s: Settings) -> io::Result<()> {
    use std::io::Write;
    if s.version != 1 || s.scene_volume_percent > 100 {
        return Err(io::Error::other("invalid settings version or audio volume"));
    }
    let parent = path
        .parent()
        .ok_or_else(|| io::Error::other("settings parent missing"))?;
    std::fs::create_dir_all(parent)?;
    let temp = path.with_extension("json.tmp");
    let mut file = std::fs::File::create(&temp)?;
    file.write_all(&serde_json::to_vec(&s).map_err(io::Error::other)?)?;
    file.sync_all()?;
    drop(file);
    // Native rename is atomic on supported Unix platforms. On Windows the existing
    // file is backed up before replacement, restored if publication fails.
    #[cfg(target_os = "windows")]
    if path.exists() {
        let backup = path.with_extension("json.bak");
        std::fs::copy(path, &backup)?;
        std::fs::remove_file(path)?;
        if let Err(e) = std::fs::rename(&temp, path) {
            let _ = std::fs::copy(backup, path);
            return Err(e);
        }
        return Ok(());
    }
    std::fs::rename(temp, path)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn settings_survive_restart_and_reject_unknown_version_or_corruption() {
        let path =
            std::env::temp_dir().join(format!("salimon-settings-{}.json", std::process::id()));
        let s = Settings {
            camera_motion: false,
            condition_audio: false,
            ..Default::default()
        };
        save(&path, s).unwrap();
        assert_eq!(load(&path).unwrap(), s);
        std::fs::write(&path, b"{\"version\":99}").unwrap();
        assert!(load(&path).is_err());
        std::fs::write(&path, b"broken").unwrap();
        assert!(load(&path).is_err());
        std::fs::remove_file(path).unwrap();
    }
    #[test]
    fn legacy_preferences_gain_audio_defaults_and_new_mix_roundtrips() {
        let path = std::env::temp_dir().join(format!(
            "salimon-audio-settings-{}.json",
            std::process::id()
        ));
        std::fs::write(
            &path,
            br#"{"version":1,"camera_motion":false,"condition_audio":false,"vignette":false}"#,
        )
        .unwrap();
        let legacy = load(&path).unwrap();
        assert!(!legacy.camera_motion && !legacy.condition_audio && !legacy.vignette);
        assert!(!legacy.master_mute && legacy.scene_audio);
        assert_eq!(legacy.scene_volume_percent, 60);
        let changed = Settings {
            master_mute: true,
            scene_audio: false,
            scene_volume_percent: 20,
            ..legacy
        };
        save(&path, changed).unwrap();
        assert_eq!(load(&path).unwrap(), changed);
        let invalid = Settings {
            scene_volume_percent: 101,
            ..changed
        };
        assert!(save(&path, invalid).is_err());
        assert_eq!(load(&path).unwrap(), changed);
        let bytes = serde_json::to_vec(&invalid).unwrap();
        std::fs::write(&path, bytes).unwrap();
        assert!(load(&path).is_err());
        std::fs::remove_file(path).unwrap();
    }
}
