//! One continuous CPAL device stream for observational cabin and survival audio.
mod mix;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
pub(crate) use mix::Control;
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64, Ordering::Relaxed},
    },
    time::{Duration, Instant},
};
#[derive(Default)]
struct Counters {
    callbacks: AtomicU64,
    frames: AtomicU64,
    max_callback_ns: AtomicU64,
    max_frames: AtomicU64,
    overruns: AtomicU64,
}
#[derive(Default)]
pub(crate) struct Output {
    stream: Option<cpal::Stream>,
    mailbox: Arc<mix::Mailbox>,
    error: Arc<AtomicBool>,
    counters: Arc<Counters>,
    device_id: Option<String>,
    checked: Option<Instant>,
    origin: Option<Instant>,
    failed: bool,
    rate: u32,
    channels: u16,
    format: Option<String>,
}
impl Output {
    pub(crate) fn unavailable(&self) -> bool {
        self.failed || self.error.load(Relaxed)
    }
    pub(crate) fn silence(&mut self, epoch: u64) {
        self.mailbox.publish(
            Control {
                epoch,
                ..Default::default()
            },
            self.elapsed_ms(),
        );
    }
    fn elapsed_ms(&self) -> u64 {
        self.origin.map_or(0, |t| t.elapsed().as_millis() as u64)
    }
    pub(crate) fn update(&mut self, control: Control) {
        let now = Instant::now();
        if self.error.swap(false, Relaxed) {
            self.stream = None;
            self.failed = true;
        }
        if self
            .checked
            .is_none_or(|t| now.duration_since(t) >= Duration::from_secs(1))
        {
            self.checked = Some(now);
            let device = cpal::default_host().default_output_device();
            let id = device.as_ref().map(|d| format!("{:?}", d.id()));
            if id != self.device_id || self.stream.is_none() || self.error.load(Relaxed) {
                self.stream = None;
                self.device_id = id;
                self.error.store(false, Relaxed);
                self.origin = Some(now);
                self.mailbox = Arc::default();
                // A recreated device starts from current loops, never old transients.
                let mut fresh = control;
                fresh.events = [false; mix::VOICES];
                self.mailbox.publish(fresh, 0);
                let result = device
                    .ok_or_else(|| "no default audio output".to_string())
                    .and_then(|d| self.open(d));
                match result {
                    Ok(s) => {
                        self.stream = Some(s);
                        self.failed = false;
                    }
                    Err(e) => {
                        if !self.failed {
                            log::warn!("continuous audio unavailable: {e}");
                        }
                        self.failed = true;
                    }
                }
            }
        }
        if self.stream.is_some() {
            self.mailbox.publish(control, self.elapsed_ms());
        }
    }
    fn open(&mut self, device: cpal::Device) -> Result<cpal::Stream, String> {
        let supported = device.default_output_config().map_err(|e| e.to_string())?;
        self.rate = supported.sample_rate();
        self.channels = supported.channels();
        self.format = Some(format!("{:?}", supported.sample_format()));
        if self.channels == 0 || self.rate < 8000 {
            return Err("unsupported output channel/rate".into());
        }
        let config = supported.config();
        macro_rules! build {
            ($t:ty) => {
                self.build::<$t>(&device, &config)
            };
        }
        let stream = match supported.sample_format() {
            cpal::SampleFormat::F32 => build!(f32),
            cpal::SampleFormat::F64 => build!(f64),
            cpal::SampleFormat::I8 => build!(i8),
            cpal::SampleFormat::I16 => build!(i16),
            cpal::SampleFormat::I24 => build!(cpal::I24),
            cpal::SampleFormat::I32 => build!(i32),
            cpal::SampleFormat::I64 => build!(i64),
            cpal::SampleFormat::U8 => build!(u8),
            cpal::SampleFormat::U16 => build!(u16),
            cpal::SampleFormat::U24 => build!(cpal::U24),
            cpal::SampleFormat::U32 => build!(u32),
            cpal::SampleFormat::U64 => build!(u64),
            _ => Err("unsupported output format".into()),
        }?;
        stream.play().map_err(|e| e.to_string())?;
        Ok(stream)
    }
    fn build<T>(
        &self,
        device: &cpal::Device,
        config: &cpal::StreamConfig,
    ) -> Result<cpal::Stream, String>
    where
        T: cpal::SizedSample + cpal::FromSample<f32>,
    {
        let mut mixer = mix::Mix::new(self.rate);
        let shared = self.mailbox.clone();
        let error = self.error.clone();
        let counts = self.counters.clone();
        let origin = self.origin.expect("stream clock");
        let channels = usize::from(self.channels);
        let rate = self.rate;
        device
            .build_output_stream(
                *config,
                move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
                    let started = Instant::now();
                    mixer.begin(&shared, origin.elapsed().as_millis() as u64);
                    for frame in data.chunks_mut(channels) {
                        let s = mixer.sample();
                        for (i, out) in frame.iter_mut().enumerate() {
                            *out = T::from_sample(if channels == 1 {
                                (s[0] + s[1]) * 0.5
                            } else if i < 2 {
                                s[i]
                            } else {
                                0.
                            });
                        }
                    }
                    if started.elapsed().as_secs_f64()
                        > (data.len() / channels) as f64 / f64::from(rate)
                    {
                        counts.overruns.fetch_add(1, Relaxed);
                    }
                    counts.callbacks.fetch_add(1, Relaxed);
                    counts
                        .frames
                        .fetch_add((data.len() / channels) as u64, Relaxed);
                    counts
                        .max_frames
                        .fetch_max((data.len() / channels) as u64, Relaxed);
                    counts
                        .max_callback_ns
                        .fetch_max(started.elapsed().as_nanos() as u64, Relaxed);
                },
                move |_| {
                    error.store(true, Relaxed);
                },
                None,
            )
            .map_err(|e| e.to_string())
    }
    pub(crate) fn telemetry(&self) -> serde_json::Value {
        serde_json::json!({"backend":"cpal-0.18.2","available":self.stream.is_some()&&!self.unavailable(),"device_id":self.device_id,"sample_rate":self.rate,"sample_format":self.format,"channels":self.channels,"callbacks":self.counters.callbacks.load(Relaxed),"frames":self.counters.frames.load(Relaxed),"max_callback_ns":self.counters.max_callback_ns.load(Relaxed),"max_callback_frames":self.counters.max_frames.load(Relaxed),"callback_cpu_overruns":self.counters.overruns.load(Relaxed),"voices":mix::VOICES,"mailbox_bytes":std::mem::size_of::<mix::Mailbox>(),"mixer_bytes":std::mem::size_of::<mix::Mix>()})
    }
}

#[cfg(test)]
pub(crate) fn fixture_wav(mut control: Control) -> Vec<u8> {
    let mailbox = mix::Mailbox::default();
    let mut mixer = mix::Mix::new(48000);
    let mut pcm = Vec::new();
    for block in 0..120 {
        mailbox.publish(control, block * 1000 / 60);
        mixer.begin(&mailbox, block * 1000 / 60);
        control.events = [false; mix::VOICES];
        for _ in 0..800 {
            for v in mixer.sample() {
                pcm.extend_from_slice(&((v * 32767.) as i16).to_le_bytes());
            }
        }
    }
    let mut wav = Vec::new();
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&(36 + pcm.len() as u32).to_le_bytes());
    wav.extend_from_slice(b"WAVEfmt ");
    wav.extend_from_slice(&16u32.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes());
    wav.extend_from_slice(&2u16.to_le_bytes());
    wav.extend_from_slice(&48000u32.to_le_bytes());
    wav.extend_from_slice(&192000u32.to_le_bytes());
    wav.extend_from_slice(&4u16.to_le_bytes());
    wav.extend_from_slice(&16u16.to_le_bytes());
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&(pcm.len() as u32).to_le_bytes());
    wav.extend(pcm);
    wav
}

/// Original continuous callback PCM from 16 ms production control observations.
#[cfg(test)]
pub(crate) fn sequence_wav(controls: &[Control]) -> (Vec<u8>, serde_json::Value) {
    let mailbox = mix::Mailbox::default();
    let mut mixer = mix::Mix::new(48000);
    let mut pcm = Vec::new();
    let mut peak = 0f32;
    let start = Instant::now();
    for (tick, c) in controls.iter().enumerate() {
        mailbox.publish(*c, tick as u64 * 16);
        mixer.begin(&mailbox, tick as u64 * 16);
        for _ in 0..768 {
            for v in mixer.sample() {
                peak = peak.max(v.abs());
                pcm.extend_from_slice(&((v * 32767.) as i16).to_le_bytes());
            }
        }
    }
    let timing = serde_json::json!({"frames":pcm.len()/4,"sample_rate":48000,"cpu_ns":start.elapsed().as_nanos(),"peak":peak,"mailbox_bytes":std::mem::size_of::<mix::Mailbox>(),"mixer_bytes":std::mem::size_of::<mix::Mix>(),"voices":mix::VOICES});
    let mut wav = Vec::new();
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&(36 + pcm.len() as u32).to_le_bytes());
    wav.extend_from_slice(b"WAVEfmt ");
    wav.extend_from_slice(&16u32.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes());
    wav.extend_from_slice(&2u16.to_le_bytes());
    wav.extend_from_slice(&48000u32.to_le_bytes());
    wav.extend_from_slice(&192000u32.to_le_bytes());
    wav.extend_from_slice(&4u16.to_le_bytes());
    wav.extend_from_slice(&16u16.to_le_bytes());
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&(pcm.len() as u32).to_le_bytes());
    wav.extend(pcm);
    (wav, timing)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore = "opens the actual default audio device for native callback/lifecycle telemetry"]
    fn native_continuous_output_582() {
        let root = std::path::PathBuf::from(std::env::var_os("SALIMON_AUDIO_EVIDENCE").unwrap());
        std::fs::create_dir_all(&root).unwrap();
        let mut output = Output::default();
        let mut c = Control {
            enabled: 3,
            ..Default::default()
        };
        c.gains[0] = 0.02;
        c.gains[1] = 0.015;
        for _ in 0..120 {
            output.update(c);
            std::thread::sleep(Duration::from_millis(16));
        }
        let active = output.telemetry();
        output.silence(1);
        std::thread::sleep(Duration::from_millis(100));
        let muted = output.telemetry();
        std::fs::write(root.join("native-output.json"),serde_json::to_vec_pretty(&serde_json::json!({"active":active,"muted":muted,"scope":"actual default-device callbacks; not a microphone/loopback recording or physical device-switch qualification"})).unwrap()).unwrap();
        assert!(
            !output.unavailable(),
            "no native device; retain telemetry rather than calling it a pass"
        );
        assert!(active["callbacks"].as_u64().unwrap() > 2);
        assert!(muted["callbacks"].as_u64().unwrap() > active["callbacks"].as_u64().unwrap());
    }
}
