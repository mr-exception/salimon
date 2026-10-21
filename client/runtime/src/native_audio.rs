//! Shared optional OS WAV playback used by survival and Scout presentation.
use std::{
    path::Path,
    process::{Child, Command, Stdio},
};

pub(crate) fn spawn(path: &Path) -> std::io::Result<Child> {
    #[cfg(target_os = "macos")]
    let mut cmd = {
        let mut c = Command::new("afplay");
        c.arg(path);
        c
    };
    #[cfg(target_os = "windows")]
    let mut cmd = {
        let mut c = Command::new("powershell.exe");
        c.args(["-NoProfile", "-NonInteractive", "-Command"]);
        c.arg(format!(
            "(New-Object System.Media.SoundPlayer '{}').PlaySync()",
            path.to_string_lossy().replace('\'', "''")
        ));
        c
    };
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    let mut cmd = {
        let player = std::env::var_os("PATH")
            .into_iter()
            .flat_map(|p| std::env::split_paths(&p).collect::<Vec<_>>())
            .find_map(|dir| {
                ["paplay", "aplay"]
                    .into_iter()
                    .map(|n| dir.join(n))
                    .find(|p| p.is_file())
            })
            .ok_or_else(|| std::io::Error::other("install paplay or aplay"))?;
        let mut c = Command::new(player);
        c.arg(path);
        c
    };
    cmd.stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
}
