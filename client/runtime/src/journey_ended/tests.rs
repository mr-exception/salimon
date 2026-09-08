use super::*;
use crate::session::GameplaySession;
use salimon_survival::{
    Points,
    effects::{Damage, Rules, SourceRule},
};
fn ended(source: Source) -> GameplaySession {
    let mut s = GameplaySession::default();
    if let Source::Registered(_) = source {
        let mut rules = Rules::default();
        rules.sources.push(SourceRule::unmodified(source, 10));
        s.survival = salimon_survival::Survival::with_rules(Default::default(), rules).unwrap();
    }
    s.advance_game(Duration::from_millis(1234));
    s.run.journey.distance_meters = 12345.67;
    s.survival
        .damage(Damage {
            source,
            amount: Points::FULL,
            wound: false,
        })
        .unwrap();
    s
}
#[test]
fn journey_ended_report_and_archive_share_typed_causes_statistics_and_location() {
    for (source, name) in [
        (Source::Starvation, "STARVATION"),
        (Source::Radiation, "RADIATION"),
        (Source::Burning, "BURNING"),
        (Source::Fall, "FALL"),
        (Source::Environment, "ENVIRONMENT"),
        (Source::Registered(42), "REGISTERED HAZARD #42"),
    ] {
        let s = ended(source);
        let before = s.save_bytes(1).unwrap();
        let run = s.ended_run().unwrap();
        let ui = Report::from(&run);
        let archive: serde_json::Value =
            serde_json::from_slice(&s.ended_run_bytes().unwrap()).unwrap();
        assert_eq!(ui.cause, name);
        assert_eq!(archive["cause"], serde_json::to_value(source).unwrap());
        assert_eq!(ui.distance, "12.35 KM");
        assert_eq!(
            ui.location,
            archive["final_location"]["name"]
                .as_str()
                .unwrap()
                .to_ascii_uppercase()
        );
        assert_eq!(ui.duration, "00H 00M 01S");
        assert_eq!(s.save_bytes(1).unwrap(), before);
    }
    assert!(GameplaySession::default().ended_run().is_err());
}
#[test]
fn journey_ended_layout_button_matches_rendered_pixels_and_bounds_at_dpi_resize_and_aspects() {
    let report = Report::from(&ended(Source::Starvation).ended_run().unwrap());
    for surface in [
        [640, 360],
        [641, 361],
        [1280, 720],
        [1920, 1080],
        [2560, 1440],
        [1080, 1920],
    ] {
        for dpi in [1., 1.5, 2., f64::NAN] {
            let l = Layout::new(surface, dpi).unwrap();
            let [x, y, w, h] = l.button_bounds();
            let point = [x + w / 2., y + h / 2.];
            assert!(l.contains(point));
            assert!(!l.contains([x - 1., y]));
            assert!(!l.contains([x + w, y]));
            assert!(!l.contains([x, y + h]));
            assert!(!l.contains([f64::NAN, y]));
            let pixels = raster::draw(&report, l, true, false);
            // Pick the button's plain left interior, away from centered glyphs.
            let px = (x + 4.) - (f64::from(surface[0] - l.canvas[0]) / 2.);
            let py = (y + 4.) - (f64::from(surface[1] - l.canvas[1]) / 2.);
            let i = (py.floor() as usize * l.canvas[0] as usize + px.floor() as usize) * 4;
            assert_eq!(&pixels[i..i + 4], &[64, 215, 191, 255]);
            assert_eq!(pixels.len(), (l.canvas[0] * l.canvas[1] * 4) as usize);
            assert!(l.canvas[0] <= surface[0] - 32 && l.canvas[1] <= surface[1] - 32);
        }
    }
    assert!(Layout::new([0, 0], 1.).is_none());
    let large = Layout::new([u32::MAX, u32::MAX], 2.).unwrap();
    assert!(u64::from(large.canvas[0]) * u64::from(large.canvas[1]) <= 16 * 1024 * 1024);
}
#[test]
fn journey_ended_pointer_cache_failure_and_live_reset_are_observational() {
    let mut screen = Screen::default();
    let surface = [1280, 720];
    let report = Report::from(&ended(Source::Fall).ended_run().unwrap());
    let a = screen
        .image(Some(report.clone()), surface, 1.)
        .unwrap()
        .revision;
    assert_eq!(
        screen
            .image(Some(report.clone()), surface, 1.)
            .unwrap()
            .revision,
        a
    );
    let [x, y, w, h] = Layout::new(surface, 1.).unwrap().button_bounds();
    screen.pointer_moved([x + w / 2., y + h / 2.]);
    assert!(screen.clicked_restart(surface, 1.));
    let hover = screen
        .image(Some(report.clone()), surface, 1.)
        .unwrap()
        .revision;
    assert_ne!(hover, a);
    screen.clear_pointer();
    assert!(!screen.clicked_restart(surface, 1.));
    screen.restart_failed();
    let fail = screen
        .image(Some(report.clone()), surface, 1.)
        .unwrap()
        .revision;
    assert_ne!(fail, a);
    assert!(screen.image(None, surface, 1.).is_none());
    assert!(!screen.failed);
    assert_eq!(screen.image(Some(report), surface, 1.).unwrap().revision, a);
}
#[test]
fn journey_ended_rounding_is_bounded_for_subsecond_long_runs_and_large_distances() {
    assert_eq!(format_duration(Duration::from_millis(5)), "0.005S");
    assert_eq!(
        format_duration(Duration::from_secs(90061)),
        "1D 01H 01M 01S"
    );
    assert!(format_duration(Duration::MAX).len() <= 30);
    for distance in [0., 999., 1000., 999999999., 1e12, f64::MAX] {
        assert!(format_distance(distance).len() <= 16);
    }
}
#[test]
fn journey_ended_raster_evidence() {
    if let Ok(path) = std::env::var("SALIMON_JOURNEY_EVIDENCE_DIR") {
        let root = std::path::Path::new(&path);
        std::fs::create_dir_all(root).unwrap();
        let report = Report::from(&ended(Source::Starvation).ended_run().unwrap());
        for (name, surface, dpi, failed) in [
            ("journey-ended", [1280, 720], 1., false),
            ("restart-failed", [640, 360], 1., true),
            ("journey-ended-hidpi", [2560, 1440], 2., false),
        ] {
            let mut screen = Screen::default();
            if failed {
                screen.restart_failed();
            }
            let image = screen.image(Some(report.clone()), surface, dpi).unwrap();
            std::fs::write(root.join(format!("{name}.rgba")), image.rgba8).unwrap();
            std::fs::write(
                root.join(format!("{name}.json")),
                format!("{{\"width\":{},\"height\":{}}}", image.width, image.height),
            )
            .unwrap();
        }
    }
}
