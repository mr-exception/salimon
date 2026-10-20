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
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            version: 1,
            camera_motion: true,
            condition_audio: true,
            vignette: true,
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
            if s.version != 1 {
                return Err(io::Error::other("unsupported settings version"));
            }
            Ok(s)
        }
    }
}
pub(super) fn save(path: &Path, s: Settings) -> io::Result<()> {
    use std::io::Write;
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
}
