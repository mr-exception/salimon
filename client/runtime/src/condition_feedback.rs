//! Snapshot-only, bounded sensory presentation. Never changes session authority.
#[cfg(test)]
mod audio;
mod settings;
use crate::session::{GameplaySession, ViewMode};
use salimon_renderer::{CameraFrame, OverlayImage, OverlayPlacement};
pub(crate) use settings::Settings;
use std::time::Duration;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Cue {
    Heart,
    Hunger,
    Breath,
}
#[derive(Clone, Copy)]
pub(crate) struct Observation {
    values: [f64; 3],
    active: bool,
    walking: bool,
    now: Duration,
    serial: u64,
}
impl Observation {
    pub(crate) fn capture(s: &GameplaySession, focused: bool, occluded: bool) -> Self {
        let v = s.survival.snapshot();
        Self {
            values: [v.health.value(), v.hunger.value(), v.fatigue.value()],
            active: focused
                && !occluded
                && !s.is_paused()
                && !s.crafting_ui.open
                && !v.terminal
                && s.view_mode == ViewMode::Gameplay,
            walking: s.character.location() != salimon_character::CharacterLocation::Cockpit
                && (s.input.movement().forward
                    || s.input.movement().backward
                    || s.input.movement().left
                    || s.input.movement().right),
            now: v.active_time,
            serial: s.run.serial,
        }
    }
}
#[derive(Default)]
pub(crate) struct Presenter {
    pub(crate) settings: Settings,
    pub(crate) settings_open: bool,
    pub(crate) message: String,
    path: Option<std::path::PathBuf>,
    pending_audio: Option<Cue>,
    low: [bool; 3],
    fired: [bool; 3],
    last: Option<(u64, Duration)>,
    weight: f64,
    motion: f64,
    phase: f64,
    alpha: u8,
    pixels: Vec<u8>,
}
impl Presenter {
    pub(crate) fn open() -> Self {
        let mut p = Self::default();
        match settings::path().and_then(|path| {
            p.path = Some(path.clone());
            settings::load(&path)
        }) {
            Ok(s) => p.settings = s,
            Err(e) => p.message = format!("Settings: {e}"),
        }
        p
    }
    /// Return one bounded cue per threshold crossing, never an endless loop.
    fn observe(&mut self, o: Observation) -> Option<Cue> {
        let dt = match self.last.replace((o.serial, o.now)) {
            Some((serial, old)) if serial == o.serial && o.now >= old => {
                (o.now - old).as_secs_f64().min(0.1)
            }
            _ => {
                self.low = [false; 3];
                self.fired = [false; 3];
                self.weight = 0.0;
                self.motion = 0.0;
                0.0
            }
        };
        self.phase = o.now.as_secs_f64();
        for (i, threshold) in [25.0, 20.0, 20.0].into_iter().enumerate() {
            if o.values[i] < threshold {
                self.low[i] = true;
            } else if o.values[i] >= threshold + 3.0 {
                self.low[i] = false;
                self.fired[i] = false;
            }
        }
        if !o.active {
            self.weight = 0.0;
            self.motion = 0.0;
            self.pending_audio = None;
            return None;
        }
        let fade = 1.0 - (-dt * 3.0).exp();
        self.weight += (f64::from(self.low[0] && self.settings.vignette) - self.weight) * fade;
        let target = if self.low[2] && o.walking && self.settings.camera_motion {
            if o.values[2] == 0.0 { 1.0 } else { 0.5 }
        } else {
            0.0
        };
        self.motion += (target - self.motion) * fade;
        if self.settings.condition_audio && !self.settings.master_mute {
            for (i, cue) in [Cue::Heart, Cue::Hunger, Cue::Breath]
                .into_iter()
                .enumerate()
            {
                if self.low[i] && !self.fired[i] {
                    self.fired[i] = true;
                    return Some(cue);
                }
            }
        }
        None
    }
    pub(crate) fn update(&mut self, o: Observation) {
        self.pending_audio = self.observe(o);
    }
    pub(crate) fn take_audio_cue(&mut self) -> Option<Cue> {
        self.pending_audio.take()
    }
    pub(crate) fn camera(&self, camera: &mut CameraFrame) {
        // Roll only the presentation up vector. The center ray/aim is unchanged.
        let forward = salimon_math::sub(camera.target_meters, camera.position_meters);
        let side = salimon_math::cross(forward, camera.up.map(f64::from));
        let n = salimon_math::length(side);
        if n > 0.0 {
            let roll = self.motion * 0.003 * (self.phase * 2.4).sin();
            camera.up =
                std::array::from_fn(|i| (f64::from(camera.up[i]) + side[i] / n * roll) as f32);
        }
    }
    pub(crate) fn image(&mut self) -> Option<OverlayImage<'_>> {
        let alpha = (self.weight * 30.0).round() as u8;
        if alpha == 0 {
            return None;
        }
        if self.alpha != alpha || self.pixels.is_empty() {
            self.alpha = alpha;
            self.pixels = vec![0; 128 * 128 * 4];
            for y in 0..128 {
                for x in 0..128 {
                    let r = (((x as f64 - 63.5) / 63.5).powi(2)
                        + ((y as f64 - 63.5) / 63.5).powi(2))
                    .sqrt();
                    let edge = ((r - 0.65) / 0.65).clamp(0.0, 1.0);
                    let i = (y * 128 + x) * 4;
                    self.pixels[i..i + 4].copy_from_slice(&[
                        110,
                        15,
                        22,
                        (edge * edge * f64::from(alpha)) as u8,
                    ]);
                }
            }
        }
        Some(OverlayImage {
            width: 128,
            height: 128,
            rgba8: &self.pixels,
            revision: u64::from(alpha),
            placement: OverlayPlacement::Fullscreen,
        })
    }
    pub(crate) fn toggle_setting(&mut self, key: usize) {
        let mut next = self.settings;
        match key {
            0 => next.camera_motion = !next.camera_motion,
            1 => next.condition_audio = !next.condition_audio,
            2 => next.vignette = !next.vignette,
            3 => next.master_mute = !next.master_mute,
            4 => next.scene_audio = !next.scene_audio,
            5 => next.scene_volume_percent = (next.scene_volume_percent + 20) % 120,
            _ => return,
        }
        match self
            .path
            .as_deref()
            .ok_or_else(|| "Settings storage unavailable".to_owned())
            .and_then(|p| settings::save(p, next).map_err(|e| e.to_string()))
        {
            Ok(()) => {
                self.settings = next;
                self.pending_audio = None;
                self.message = "Settings saved".into();
            }
            Err(e) => self.message = format!("Save failed: {e}"),
        }
    }

    pub(crate) fn menu_text(&self) -> String {
        let on = |v| if v { "ON" } else { "OFF" };
        format!(
            "SALIMON / ACCESSIBILITY\n\nC   CAMERA MOTION: {}\nA   CONDITION AUDIO: {}\nV   HEALTH VIGNETTE: {}\nM   MASTER MUTE: {}\nB   SHIP AUDIO: {}\nG   SHIP VOLUME: {}%\n\nS   BACK TO PAUSE MENU\nENTER / ESC   RESUME\n\n{}",
            on(self.settings.camera_motion),
            on(self.settings.condition_audio),
            on(self.settings.vignette),
            on(self.settings.master_mute),
            on(self.settings.scene_audio),
            self.settings.scene_volume_percent,
            self.message.chars().take(65).collect::<String>()
        )
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn threshold_hysteresis_bounded_audio_fade_and_menu_suppression() {
        let mut p = Presenter::default();
        let mut o = Observation {
            values: [24., 19., 19.],
            active: true,
            walking: true,
            now: Duration::ZERO,
            serial: 1,
        };
        assert_eq!(p.observe(o), Some(Cue::Heart));
        o.now += Duration::from_secs(1);
        assert_eq!(p.observe(o), Some(Cue::Hunger));
        o.now += Duration::from_secs(1);
        assert_eq!(p.observe(o), Some(Cue::Breath));
        o.now += Duration::from_secs(1);
        assert_eq!(p.observe(o), None);
        assert!(p.weight > 0.0 && p.motion > 0.0);
        o.values = [26., 21., 21.];
        assert_eq!(p.observe(o), None);
        o.active = false;
        p.observe(o);
        assert!(p.image().is_none());
        assert_eq!(p.motion, 0.0);
        o.values = [30., 25., 25.];
        p.observe(o);
        o.active = true;
        o.values = [24., 19., 19.];
        assert_eq!(p.observe(o), Some(Cue::Heart));
    }
    #[test]
    fn disabled_effects_preserve_camera_and_center_ray() {
        let mut p = Presenter {
            settings: Settings {
                version: 1,
                camera_motion: false,
                condition_audio: false,
                vignette: false,
                ..Default::default()
            },
            ..Default::default()
        };
        for i in 0..100 {
            assert_eq!(
                p.observe(Observation {
                    values: [1.; 3],
                    active: true,
                    walking: true,
                    now: Duration::from_millis(i * 16),
                    serial: 1
                }),
                None
            );
        }
        assert!(p.image().is_none());
        let mut c = CameraFrame {
            position_meters: [0.; 3],
            target_meters: [1., 0., 0.],
            up: [0., 1., 0.],
            vertical_fov_radians: 1.,
            near_plane_meters: 0.05,
        };
        p.camera(&mut c);
        assert_eq!(c.up, [0., 1., 0.]);
        p.motion = 1.;
        p.phase = 1.;
        p.camera(&mut c);
        assert_eq!(c.position_meters, [0.; 3]);
        assert_eq!(c.target_meters, [1., 0., 0.]);
        assert!((c.up[2] as f64).abs() <= 0.003);
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    #[test]
    fn snapshots_do_not_mutate_authority_and_preview_has_clear_center() {
        let s = GameplaySession::new_run();
        let before = s.save_bytes(123).unwrap();
        let mut p = Presenter::default();
        p.update(Observation::capture(&s, true, false));
        assert_eq!(s.save_bytes(123).unwrap(), before);
        p.weight = 1.0;
        let image = p.image().unwrap();
        assert_eq!(image.rgba8[(64 * 128 + 64) * 4 + 3], 0);
        assert!(image.rgba8.as_chunks::<4>().0.iter().all(|p| p[3] <= 30));
        if let Some(dir) = std::env::var_os("SALIMON_FEEDBACK_PREVIEW") {
            let dir = std::path::PathBuf::from(dir);
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(dir.join("vignette.rgba"), image.rgba8).unwrap();
            let (_, _, pixels) = crate::action_bar::rasterize_action_bar(&p.menu_text(), 2);
            let (w, h, _) = crate::action_bar::rasterize_action_bar(&p.menu_text(), 2);
            std::fs::write(dir.join("settings.rgba"), pixels).unwrap();
            std::fs::write(dir.join("settings-size.txt"), format!("{w} {h}")).unwrap();
        }
    }
    #[test]
    fn master_mute_suppresses_existing_survival_cues() {
        let mut p = Presenter {
            settings: Settings {
                master_mute: true,
                ..Default::default()
            },
            ..Default::default()
        };
        let o = Observation {
            values: [1.; 3],
            active: true,
            walking: true,
            now: Duration::ZERO,
            serial: 1,
        };
        assert_eq!(p.observe(o), None);
        p.settings.master_mute = false;
        assert_eq!(p.observe(o), Some(Cue::Heart));
    }
    #[test]
    fn mix_controls_persist_separately_from_active_run() {
        let path = std::env::temp_dir().join(format!(
            "salimon-audio-controls-{}.json",
            std::process::id()
        ));
        let mut p = Presenter {
            path: Some(path.clone()),
            ..Default::default()
        };
        p.toggle_setting(3);
        p.toggle_setting(4);
        p.toggle_setting(5);
        let saved = settings::load(&path).unwrap();
        assert!(saved.master_mute);
        assert!(!saved.scene_audio);
        assert_eq!(saved.scene_volume_percent, 80);
        for _ in 0..8 {
            p.toggle_setting(5);
            assert!(p.settings.scene_volume_percent <= 100);
        }
        assert_eq!(settings::load(&path).unwrap(), p.settings);
        std::fs::remove_file(path).unwrap();
    }
}
