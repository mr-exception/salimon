//! Expanded compact habitation: physical mechanisms and supported player postures.
#[cfg(test)]
mod tests;
use super::{GameplaySession, frames::character_ship_frame};
use salimon_math::{length, sub};
use salimon_persistence::{SaveError, Validate, invalid};
use salimon_ship::mechanism::Mechanism;
use std::time::Duration;
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub(crate) enum Furniture {
    Table,
    Seat,
    Screen,
    Shutter,
    Locker,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub(crate) enum RestKind {
    Seat,
    Bunk,
}
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Rest {
    pub kind: RestKind,
    pub standing_eye: [f64; 3],
    pub from_eye: [f64; 3],
    pub fraction: f64,
    pub entering: bool,
}
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Cabin {
    pub motions: [Mechanism; 5],
    pub seat_harness: bool,
    pub bunk_harness: bool,
    pub rest: Option<Rest>,
}
impl Default for Cabin {
    fn default() -> Self {
        Self {
            motions: [
                Default::default(),
                Default::default(),
                Mechanism::open(),
                Mechanism::open(),
                Default::default(),
            ],
            seat_harness: false,
            bunk_harness: false,
            rest: None,
        }
    }
}
impl Validate for Cabin {
    fn validate(&self) -> Result<(), SaveError> {
        for m in self.motions {
            m.validate()?;
        }
        if let Some(r) = &self.rest {
            if !r.fraction.is_finite()
                || !(0.0..=1.0).contains(&r.fraction)
                || r.standing_eye
                    .iter()
                    .chain(r.from_eye.iter())
                    .any(|v| !v.is_finite() || v.abs() > 20.)
            {
                return Err(invalid("supported posture transition"));
            }
            if r.kind == RestKind::Seat && self.motions[Furniture::Seat as usize].fraction != 1. {
                return Err(invalid("occupied seat is folded"));
            }
        }
        Ok(())
    }
}
type CabinCheckpoint = ([bool; 5], bool, bool, Option<(RestKind, bool)>);

#[derive(Clone, Copy, Debug)]
pub(crate) enum CabinAction {
    Rest,
    Restraint,
}
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Geometry {
    pub table: [f64; 3],
    pub seat: [f64; 3],
    pub seat_base: [f64; 3],
    pub screen: [f64; 3],
    pub shutter: [f64; 3],
    pub locker: [f64; 3],
    pub locker_hinge: [f64; 3],
}
pub(crate) fn geometry() -> &'static Geometry {
    static G: std::sync::LazyLock<Geometry> = std::sync::LazyLock::new(|| {
        serde_json::from_str(include_str!(
            "../../../assets/fixtures/scout-habitation-kit/geometry.json"
        ))
        .expect("published cabin geometry")
    });
    &G
}
pub(crate) fn pose(kind: Furniture, m: Mechanism) -> ([f64; 3], [f64; 4]) {
    let p = m.eased();
    match kind {
        Furniture::Table => {
            let a = -(1. - p) * std::f64::consts::FRAC_PI_2;
            (geometry().table, [(a / 2.).sin(), 0., 0., (a / 2.).cos()])
        }
        Furniture::Seat => {
            let a = (1. - p) * std::f64::consts::FRAC_PI_2;
            (geometry().seat, [(a / 2.).sin(), 0., 0., (a / 2.).cos()])
        }
        Furniture::Shutter => (geometry().shutter, [0., 0., 0., 1.]),
        Furniture::Locker => {
            let a = -p * std::f64::consts::FRAC_PI_2;
            (
                geometry().locker_hinge,
                [0., (a / 2.).sin(), 0., (a / 2.).cos()],
            )
        }
        Furniture::Screen => (geometry().screen, [0., 0., 0., 1.]),
    }
}
fn transformed(p: [f64; 3], q: [f64; 4], b: [f64; 6]) -> [f64; 6] {
    let mut out = [
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::INFINITY,
        f64::NEG_INFINITY,
    ];
    for x in [b[0], b[1]] {
        for y in [b[2], b[3]] {
            for z in [b[4], b[5]] {
                let v = salimon_physics::rotate(q, [x, y, z]);
                for i in 0..3 {
                    out[2 * i] = out[2 * i].min(v[i] + p[i]);
                    out[2 * i + 1] = out[2 * i + 1].max(v[i] + p[i]);
                }
            }
        }
    }
    out
}
pub(crate) fn mechanism_solids(kind: Furniture, m: Mechanism) -> Vec<[f64; 6]> {
    if kind == Furniture::Shutter {
        let phase = m.eased();
        let track = (phase / 0.25).min(1.);
        let lift = ((phase - 0.25) / 0.75).clamp(0., 1.);
        return (0..5)
            .map(|i| {
                let mut p = geometry().shutter;
                let closed = p[1] + (i as f64 - 2.) * 0.29;
                p[1] = closed + (2.6 - closed) * lift;
                p[2] += i as f64 * 0.045 * track;
                transformed(
                    p,
                    [0., 0., 0., 1.],
                    [-1.15, 1.15, -0.145, 0.145, -0.02, 0.02],
                )
            })
            .collect();
    }
    if kind == Furniture::Screen {
        return (0..3)
            .map(|i| {
                let mut p = geometry().screen;
                p[0] = (1. - m.eased()) * (-1.52 + i as f64 * 0.72) + m.eased() * 0.75;
                p[2] += i as f64 * 0.04;
                transformed(
                    p,
                    [0., 0., 0., 1.],
                    [-0.35, 0.35, -0.725, 0.725, -0.0175, 0.0175],
                )
            })
            .collect();
    }
    let (p, q) = pose(kind, m);
    let b = match kind {
        Furniture::Table => [-0.5, 0.5, -0.03, 0.03, 0., 0.8],
        Furniture::Seat => [-0.25, 0.25, -0.1, 0., -0.5, 0.],
        Furniture::Shutter => unreachable!(),
        Furniture::Locker => [0., 0.7, -0.65, 0.65, -0.0175, 0.0175],
        Furniture::Screen => unreachable!(),
    };
    vec![transformed(p, q, b)]
}
fn shell_solids() -> Vec<[f64; 6]> {
    let p = geometry().locker;
    [
        [-0.35, 0.35, -0.65, 0.65, -0.25, -0.21],
        [-0.35, -0.31, -0.65, 0.65, -0.21, 0.23],
        [0.31, 0.35, -0.65, 0.65, -0.21, 0.23],
        [-0.31, 0.31, -0.65, -0.61, -0.21, 0.23],
        [-0.31, 0.31, 0.61, 0.65, -0.21, 0.23],
    ]
    .into_iter()
    .map(|b| transformed(p, [0., 0., 0., 1.], b))
    .collect()
}
impl GameplaySession {
    pub(super) fn cabin_solids(&self) -> Vec<[f64; 6]> {
        self.cabin_solids_except(None, false)
    }
    fn cabin_solids_except(&self, exclude: Option<Furniture>, skip_bunk: bool) -> Vec<[f64; 6]> {
        let Some(c) = &self.habitation.cabin else {
            return Vec::new();
        };
        let mut out = shell_solids();
        if exclude != Some(Furniture::Seat) {
            let b = geometry().seat_base;
            for x in [-0.20, 0.20] {
                for z in [-0.20, 0.20] {
                    out.push([
                        b[0] + x - 0.0225,
                        b[0] + x + 0.0225,
                        b[1],
                        b[1] + 0.4,
                        b[2] + z - 0.0225,
                        b[2] + z + 0.0225,
                    ]);
                }
            }
            out.push([
                b[0] - 0.04,
                b[0] + 0.04,
                b[1],
                b[1] + 0.8,
                b[2] + 0.26,
                b[2] + 0.34,
            ]);
            let mut back = geometry().seat;
            back[2] += 0.05;
            out.push(transformed(
                back,
                [0., 0., 0., 1.],
                [-0.25, 0.25, 0.11, 0.75, -0.03, 0.03],
            ));
        }
        if !skip_bunk {
            let a = self.habitation.bunk_open_fraction * std::f64::consts::FRAC_PI_2;
            for bounds in [
                [-1.1, 1.1, 0., 0.85, -0.05, 0.05],
                [-1., 1., 0.05, 0.8, -0.115, -0.055],
            ] {
                out.push(transformed(
                    salimon_character::habitation_geometry::BUNK_HINGE_METERS,
                    [(a / 2.).sin(), 0., 0., (a / 2.).cos()],
                    bounds,
                ));
            }
        }
        for k in [
            Furniture::Table,
            Furniture::Seat,
            Furniture::Screen,
            Furniture::Shutter,
            Furniture::Locker,
        ] {
            if exclude != Some(k) {
                out.extend(mechanism_solids(k, c.motions[k as usize]));
            }
        }
        out
    }
    fn cabin_bounds_clear(&self, b: [f64; 6], ignore_player: bool) -> bool {
        let s = self.snapshot();
        let frame = character_ship_frame(s.ship.pose);
        let eye = frame.world_to_local(s.character.eye_position_meters);
        if !ignore_player
            && s.character.location == salimon_character::CharacterLocation::InsideShip
        {
            let actor = self
                .habitation
                .cabin
                .as_ref()
                .and_then(|c| c.rest.as_ref())
                .map_or(
                    [
                        eye[0] - salimon_character::PLAYER_RADIUS_METERS,
                        eye[0] + salimon_character::PLAYER_RADIUS_METERS,
                        eye[1] - salimon_character::PLAYER_EYE_HEIGHT_METERS,
                        eye[1] + 0.05,
                        eye[2] - salimon_character::PLAYER_RADIUS_METERS,
                        eye[2] + salimon_character::PLAYER_RADIUS_METERS,
                    ],
                    |r| rest_body(r, eye),
                );
            if boxes_overlap(actor, b, 0.0) {
                return false;
            }
        }
        !self.mining.session.fragments().iter().any(|f| {
            salimon_character::cargo::sphere_overlaps_box(
                frame.world_to_local(f.transform().position().meters()),
                crate::fragment_physics::sphere_radius(*f),
                b,
            )
        })
    }
    pub(super) fn toggle_cabin_fixture(&mut self, kind: Furniture) {
        let Some(c) = &mut self.habitation.cabin else {
            return;
        };
        if kind == Furniture::Seat && c.rest.as_ref().is_some_and(|r| r.kind == RestKind::Seat) {
            self.feedback = Some("Stand before folding the occupied seat");
            return;
        }
        c.motions[kind as usize].reverse();
        self.feedback = Some("Manual mechanism moving; press E again to reverse");
    }
    pub(super) fn advance_cabin(&mut self, delta: Duration) {
        if self.habitation.cabin.is_none() {
            return;
        }
        for k in [
            Furniture::Table,
            Furniture::Seat,
            Furniture::Screen,
            Furniture::Shutter,
            Furniture::Locker,
        ] {
            let old = self.habitation.cabin.as_ref().unwrap().motions[k as usize];
            let duration = Duration::from_millis(match k {
                Furniture::Table => 700,
                Furniture::Seat => 800,
                Furniture::Screen => 1000,
                Furniture::Shutter => 1200,
                Furniture::Locker => 650,
            });
            let next = old.next(delta, duration);
            if old == next {
                continue;
            }
            let clear = swept_solids(k, old, next).into_iter().all(|sweep| {
                self.cabin_bounds_clear(sweep, false)
                    && self
                        .cabin_solids_except(Some(k), false)
                        .iter()
                        .all(|other| !boxes_overlap(sweep, *other, 0.001))
            });
            if clear {
                self.habitation.cabin.as_mut().unwrap().motions[k as usize] = next;
            } else {
                self.feedback = Some("Fixture obstructed; reverse or clear player and belongings");
            }
        }
        self.advance_rest(delta);
    }
    pub(crate) fn cabin_action(&mut self, action: CabinAction) {
        if self.building.active {
            self.feedback = Some("Close the blueprint tool before using furniture");
            return;
        }
        if !self.admit_survival_action(salimon_survival::restrictions::Action::Menu) {
            return;
        }
        let Some(c) = &self.habitation.cabin else {
            return;
        };
        if matches!(action, CabinAction::Rest) && c.rest.is_some() {
            self.stand_from_furniture();
            return;
        }
        let s = self.snapshot();
        let frame = character_ship_frame(s.ship.pose);
        let target = self.available_target(
            s.character.location,
            frame.world_to_local(s.character.eye_position_meters),
            frame.world_to_local(s.character.look_target_meters),
        );
        let kind = if let Some(r) = &c.rest {
            Some(r.kind)
        } else {
            match target {
                Some(super::interaction::InteractionTarget::Bunk) => Some(RestKind::Bunk),
                Some(super::interaction::InteractionTarget::CabinSeat) => Some(RestKind::Seat),
                _ => None,
            }
        };
        let Some(kind) = kind else {
            self.feedback = Some("Aim at the bunk or padded seat");
            return;
        };
        if matches!(action, CabinAction::Restraint) {
            if c.rest
                .as_ref()
                .is_some_and(|r| !r.entering || r.fraction < 1.)
                || (kind == RestKind::Bunk && self.habitation.bunk_open_fraction != 1.)
                || (kind == RestKind::Seat && c.motions[1].fraction != 1.)
            {
                self.feedback = Some("Finish deploying or settling before securing a restraint");
                return;
            }
            let c = self.habitation.cabin.as_mut().unwrap();
            let h = if kind == RestKind::Bunk {
                &mut c.bunk_harness
            } else {
                &mut c.seat_harness
            };
            *h = !*h;
            self.feedback = Some(if *h {
                "Physical restraint secured - U to release"
            } else {
                "Physical restraint released"
            });
            return;
        }
        if self.mining.session.carried_object().is_some() {
            self.feedback = Some("Put the carried object down before resting");
            return;
        }
        let c = self.habitation.cabin.as_ref().unwrap();
        if (kind == RestKind::Bunk
            && (self.habitation.bunk_open_fraction != 1.
                || c.motions[Furniture::Screen as usize].fraction != 1.))
            || (kind == RestKind::Seat && c.motions[Furniture::Seat as usize].fraction != 1.)
        {
            self.feedback = Some("Fully deploy the fixture and open the privacy screen first");
            return;
        }
        if !s.ship.power_available
            && !(if kind == RestKind::Seat {
                c.seat_harness
            } else {
                c.bunk_harness
            })
        {
            self.feedback = Some("Secure the physical restraint before resting in zero-G");
            return;
        }
        let eye = frame.world_to_local(s.character.eye_position_meters);
        self.clear_held_controls();
        self.habitation.cabin.as_mut().unwrap().rest = Some(Rest {
            kind,
            standing_eye: eye,
            from_eye: eye,
            fraction: 0.,
            entering: true,
        });
        self.feedback = Some("Settling into furniture - free look; E to get up, U restraint");
    }
    pub(super) fn stand_from_furniture(&mut self) {
        let Some(c) = &self.habitation.cabin else {
            return;
        };
        let Some(r) = &c.rest else {
            return;
        };
        if (r.kind == RestKind::Seat && c.seat_harness)
            || (r.kind == RestKind::Bunk && c.bunk_harness)
        {
            self.feedback = Some("Release physical restraint with U before getting up");
            return;
        }
        if r.kind == RestKind::Bunk && c.motions[Furniture::Screen as usize].fraction != 1. {
            self.feedback = Some("Open the privacy screen before getting up");
            return;
        }
        let eye = self
            .snapshot()
            .character
            .local_ship_position_meters
            .unwrap();
        self.clear_held_controls();
        let r = self
            .habitation
            .cabin
            .as_mut()
            .unwrap()
            .rest
            .as_mut()
            .unwrap();
        r.entering = false;
        r.from_eye = eye;
        r.fraction = 0.;
        self.feedback = Some("Getting up - motion stops if the exit is obstructed");
    }
    fn advance_rest(&mut self, delta: Duration) {
        let Some(c) = &self.habitation.cabin else {
            return;
        };
        let Some(r) = &c.rest else {
            return;
        };
        let secured = if r.kind == RestKind::Seat {
            c.seat_harness
        } else {
            c.bunk_harness
        };
        if !self.ship.snapshot().power_available && !secured && r.entering {
            let eye = self
                .character
                .local_ship_position()
                .expect("supported player inside cabin");
            let r = self
                .habitation
                .cabin
                .as_mut()
                .unwrap()
                .rest
                .as_mut()
                .unwrap();
            r.from_eye = eye;
            r.entering = false;
            r.fraction = 0.;
            self.feedback = Some("Zero-G: leaving unrestrained support");
            return;
        }
        let mut next = r.clone();
        next.fraction = (next.fraction + delta.as_secs_f64() / 0.35).min(1.);
        let goal = if next.entering {
            rest_eye(next.kind)
        } else {
            next.standing_eye
        };
        let t = next.fraction * next.fraction * (3. - 2. * next.fraction);
        let eye = std::array::from_fn(|i| next.from_eye[i] + t * (goal[i] - next.from_eye[i]));
        let b = rest_body(&next, eye);
        let exclude = (next.kind == RestKind::Seat).then_some(Furniture::Seat);
        if !self.cabin_bounds_clear(b, true)
            || self
                .cabin_solids_except(exclude, next.kind == RestKind::Bunk)
                .iter()
                .any(|other| boxes_overlap(b, *other, 0.001))
        {
            self.feedback = Some("Rest transition obstructed; clear belongings");
            return;
        }
        if !self.character.set_supported_pose(eye) {
            return;
        }
        self.habitation.cabin.as_mut().unwrap().rest = if !next.entering && next.fraction == 1. {
            None
        } else {
            Some(next)
        };
    }
    pub(super) fn validate_cabin_restore(&self) -> Result<(), SaveError> {
        if let Some(c) = &self.habitation.cabin {
            c.validate()?;
            if let Some(r) = &c.rest {
                if salimon_character::ship_floor_placement(
                    r.standing_eye,
                    salimon_character::PLAYER_RADIUS_METERS,
                )
                .is_none()
                {
                    return Err(invalid("supported posture has unsafe standing return"));
                }
                if self.character.location() != salimon_character::CharacterLocation::InsideShip {
                    return Err(invalid("supported posture outside cabin"));
                }
                if r.kind == RestKind::Bunk && self.habitation.bunk_open_fraction != 1. {
                    return Err(invalid("occupied bunk not deployed"));
                }
                let goal = if r.entering {
                    rest_eye(r.kind)
                } else {
                    r.standing_eye
                };
                let t = r.fraction * r.fraction * (3. - 2. * r.fraction);
                let eye = std::array::from_fn::<_, 3, _>(|i| {
                    r.from_eye[i] + t * (goal[i] - r.from_eye[i])
                });
                if self
                    .character
                    .local_ship_position()
                    .is_none_or(|p| length(sub(p, eye)) > 1e-7)
                {
                    return Err(invalid("rest/character pose mismatch"));
                }
            }
        }
        Ok(())
    }
}
pub(crate) fn rest_eye(kind: RestKind) -> [f64; 3] {
    match kind {
        RestKind::Seat => [0.8, 1.45, -1.05],
        RestKind::Bunk => [-0.8, 1.02, -3.45],
    }
}

pub(crate) fn default_position(target: super::interaction::InteractionTarget) -> [f64; 3] {
    use super::interaction::InteractionTarget::*;
    match target {
        CabinTable => geometry().table,
        CabinSeat => geometry().seat,
        PrivacyScreen => geometry().screen,
        ObservationShutter => geometry().shutter,
        CabinLocker => geometry().locker_hinge,
        _ => unreachable!("not expanded cabin fixture"),
    }
}
impl GameplaySession {
    pub(crate) fn cabin_interaction_anchor(
        &self,
        target: super::interaction::InteractionTarget,
    ) -> Option<[f64; 3]> {
        use super::interaction::InteractionTarget::*;
        let c = self.habitation.cabin.as_ref()?;
        let (kind, local) = match target {
            CabinTable => (Furniture::Table, [0., 0., 0.35]),
            CabinSeat => (Furniture::Seat, [0., -0.05, -0.25]),
            PrivacyScreen => {
                return Some([
                    (1. - c.motions[2].eased()) * (-0.8) + c.motions[2].eased() * 0.75,
                    geometry().screen[1],
                    geometry().screen[2] + 0.04,
                ]);
            }
            ObservationShutter => return Some(shutter_handle()),
            CabinLocker => (Furniture::Locker, [0.60, 0., 0.02]),
            _ => return None,
        };
        let (p, q) = pose(kind, c.motions[kind as usize]);
        let v = salimon_physics::rotate(q, local);
        Some(std::array::from_fn(|i| p[i] + v[i]))
    }
    pub(crate) fn available_target(
        &self,
        location: salimon_character::CharacterLocation,
        eye: [f64; 3],
        look: [f64; 3],
    ) -> Option<super::interaction::InteractionTarget> {
        use super::interaction::InteractionTarget::*;
        let base = super::interaction::available_interaction_target(location, eye, look);
        if location != salimon_character::CharacterLocation::InsideShip {
            return base;
        }
        let direction = sub(look, eye);
        [
            CabinTable,
            CabinSeat,
            PrivacyScreen,
            ObservationShutter,
            CabinLocker,
        ]
        .into_iter()
        .filter_map(|target| {
            let p = self.cabin_interaction_anchor(target)?;
            let delta = sub(p, eye);
            let distance = length(delta);
            let aim =
                salimon_math::dot(delta, direction) / (distance * length(direction)).max(1e-12);
            let visible = salimon_character::habitation_sight_obstruction(
                eye,
                p,
                super::door_passable(self.ship.snapshot()),
            )
            .is_none_or(|d| d >= distance - 0.03)
                && (self.cargo_door.open_fraction == 1.
                    || salimon_character::cargo::cargo_sight_obstruction(eye, p)
                        .is_none_or(|d| d >= distance - 0.03));
            let kind = match target {
                CabinTable => Furniture::Table,
                CabinSeat => Furniture::Seat,
                PrivacyScreen => Furniture::Screen,
                ObservationShutter => Furniture::Shutter,
                CabinLocker => Furniture::Locker,
                _ => unreachable!(),
            };
            let visible = visible
                && self.cabin_solids_except(Some(kind), false).iter().all(|b| {
                    salimon_character::cargo::box_sight_obstruction(eye, p, *b)
                        .is_none_or(|hit| hit >= distance - 0.06)
                });
            (distance <= 2.2 && aim >= 0.985 && visible).then_some((target, distance))
        })
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|v| v.0)
        .or(base)
    }
    pub(super) fn cabin_checkpoint(&self) -> Option<CabinCheckpoint> {
        self.habitation.cabin.as_ref().map(|c| {
            (
                c.motions.map(|m| m.opening),
                c.seat_harness,
                c.bunk_harness,
                c.rest.as_ref().map(|r| (r.kind, r.entering)),
            )
        })
    }
    pub(super) fn cabin_fixture_nodes(&self, powered: bool) -> Vec<salimon_renderer::FixtureNode> {
        use salimon_renderer::{FixtureMesh as Mesh, FixtureNode, FixturePose};
        let Some(c) = &self.habitation.cabin else {
            return Vec::new();
        };
        let mut nodes = Vec::new();
        for (id, k, mesh) in [
            (3001, Furniture::Table, Mesh::Table),
            (3002, Furniture::Seat, Mesh::SeatPad),
            (3004, Furniture::Locker, Mesh::LockerDoor),
        ] {
            let (p, q) = pose(k, c.motions[k as usize]);
            nodes.push(FixtureNode {
                id,
                parent: None,
                mesh,
                local: FixturePose {
                    position_meters: p,
                    orientation_xyzw: q,
                    scale: 1.,
                },
                tint: [1.; 3],
                powered,
            });
        }
        for (i, b) in mechanism_solids(Furniture::Shutter, c.motions[3])
            .into_iter()
            .enumerate()
        {
            nodes.push(FixtureNode {
                id: 3040 + i as u64,
                parent: None,
                mesh: Mesh::Shutter,
                local: FixturePose {
                    position_meters: [(b[0] + b[1]) / 2., (b[2] + b[3]) / 2., (b[4] + b[5]) / 2.],
                    ..Default::default()
                },
                tint: [1.; 3],
                powered,
            });
        }
        nodes.push(FixtureNode {
            id: 3050,
            parent: None,
            mesh: Mesh::ShutterHandle,
            local: FixturePose {
                position_meters: shutter_handle(),
                ..Default::default()
            },
            tint: [1.; 3],
            powered,
        });
        for (i, b) in mechanism_solids(Furniture::Screen, c.motions[2])
            .into_iter()
            .enumerate()
        {
            nodes.push(FixtureNode {
                id: 3010 + i as u64,
                parent: None,
                mesh: Mesh::ScreenPanel,
                local: FixturePose {
                    position_meters: [(b[0] + b[1]) / 2., (b[2] + b[3]) / 2., (b[4] + b[5]) / 2.],
                    ..Default::default()
                },
                tint: [1.; 3],
                powered,
            });
        }
        let mut back = geometry().seat;
        back[2] += 0.05;
        for (id, mesh, p) in [
            (3020, Mesh::LockerShell, geometry().locker),
            (3021, Mesh::SeatBase, geometry().seat_base),
            (3022, Mesh::SeatBack, back),
        ] {
            nodes.push(FixtureNode {
                id,
                parent: None,
                mesh,
                local: FixturePose {
                    position_meters: p,
                    ..Default::default()
                },
                tint: [1.; 3],
                powered,
            });
        }
        for (id, enabled, p, q) in [
            (3030, c.seat_harness, [0.8, 0.9, -1.0], [0., 0., 0., 1.]),
            (
                3031,
                c.bunk_harness,
                [-0.8, 0.82, -3.4],
                [
                    std::f64::consts::FRAC_1_SQRT_2,
                    0.,
                    0.,
                    std::f64::consts::FRAC_1_SQRT_2,
                ],
            ),
        ] {
            if enabled {
                nodes.push(FixtureNode {
                    id,
                    parent: None,
                    mesh: Mesh::Harness,
                    local: FixturePose {
                        position_meters: p,
                        orientation_xyzw: q,
                        scale: 1.,
                    },
                    tint: [1.; 3],
                    powered,
                });
            }
        }
        nodes
    }
}

fn boxes_overlap(a: [f64; 6], b: [f64; 6], tolerance: f64) -> bool {
    (0..3).all(|i| a[2 * i] < b[2 * i + 1] - tolerance && a[2 * i + 1] > b[2 * i] + tolerance)
}
fn rest_body(r: &Rest, eye: [f64; 3]) -> [f64; 6] {
    let blend = if r.entering {
        r.fraction
    } else {
        1. - r.fraction
    };
    let blend = blend * blend * (3. - 2. * blend);
    let (width, below) = match r.kind {
        RestKind::Seat => (0.27, 0.8),
        RestKind::Bunk => (0.90, 0.25),
    };
    let x = 0.24 + (width - 0.24) * blend;
    let bottom = salimon_character::PLAYER_EYE_HEIGHT_METERS
        + (below - salimon_character::PLAYER_EYE_HEIGHT_METERS) * blend;
    [
        eye[0] - x,
        eye[0] + x,
        eye[1] - bottom,
        eye[1] + 0.20,
        eye[2] - 0.24,
        eye[2] + 0.24,
    ]
}

/// Exact monotone translation/quarter-turn swept AABBs, including interior arc extrema.
fn swept_solids(k: Furniture, old: Mechanism, next: Mechanism) -> Vec<[f64; 6]> {
    let mut out = mechanism_solids(k, old);
    let merge = |a: &mut [f64; 6], b: [f64; 6]| {
        for i in 0..6 {
            a[i] = if i % 2 == 0 {
                a[i].min(b[i])
            } else {
                a[i].max(b[i])
            };
        }
    };
    for (a, b) in out.iter_mut().zip(mechanism_solids(k, next)) {
        merge(a, b);
    }
    if matches!(k, Furniture::Table | Furniture::Seat | Furniture::Locker) {
        let bounds: [f64; 6] = match k {
            Furniture::Table => [-0.5, 0.5, -0.03, 0.03, 0., 0.8],
            Furniture::Seat => [-0.25, 0.25, -0.1, 0., -0.5, 0.],
            Furniture::Locker => [0., 0.7, -0.65, 0.65, -0.0175, 0.0175],
            _ => unreachable!(),
        };
        let angle = |m: Mechanism| match k {
            Furniture::Table => -(1. - m.eased()) * std::f64::consts::FRAC_PI_2,
            Furniture::Seat => (1. - m.eased()) * std::f64::consts::FRAC_PI_2,
            _ => -m.eased() * std::f64::consts::FRAC_PI_2,
        };
        let low = angle(old).min(angle(next));
        let high = angle(old).max(angle(next));
        let (p, _) = pose(k, old);
        let first = if k == Furniture::Locker {
            [bounds[0], bounds[1]]
        } else {
            [bounds[2], bounds[3]]
        };
        for u in first {
            for v in [bounds[4], bounds[5]] {
                let roots = if k == Furniture::Locker {
                    [v.atan2(u), (-u).atan2(v)]
                } else {
                    [(-v).atan2(u), u.atan2(v)]
                };
                for root in roots {
                    for half in -2..=2 {
                        let a = root + f64::from(half) * std::f64::consts::PI;
                        if a >= low && a <= high {
                            let q = if k == Furniture::Locker {
                                [0., (a / 2.).sin(), 0., (a / 2.).cos()]
                            } else {
                                [(a / 2.).sin(), 0., 0., (a / 2.).cos()]
                            };
                            merge(&mut out[0], transformed(p, q, bounds));
                        }
                    }
                }
            }
        }
    }
    for a in &mut out {
        for (i, v) in a.iter_mut().enumerate() {
            *v += if i % 2 == 0 { -0.002 } else { 0.002 };
        }
    }
    out
}

impl GameplaySession {
    pub(crate) fn cabin_hint(&self) -> Option<&'static str> {
        self.habitation
            .cabin
            .as_ref()
            .and_then(|c| c.rest.as_ref())
            .map(|_| "Resting - free look | E/X get up | U secure/release restraint")
    }
}

fn shutter_handle() -> [f64; 3] {
    [
        geometry().shutter[0] - 0.9,
        1.65,
        geometry().shutter[2] + 0.2,
    ]
}

/// Explicit camera/fixture setup; all later inputs use production commands.
pub(crate) fn evidence_initial_conditions(
    s: &mut GameplaySession,
    scenario: &str,
) -> Result<(), String> {
    s.habitation = super::habitation::Habitation::starter();
    use super::interaction::InteractionTarget::*;
    let (eye, target) = match scenario {
        "scout-finishes" => (
            [0.65, 1.9973, 1.8],
            super::interaction::habitation_position(Galley),
        ),
        "scout-cabin-bunk" => ([-0.8, 1.9973, -2.2], [-0.8, 1.05, -2.95]),
        "scout-cabin-locker" => (
            [-4., 1.9973, -2.1],
            s.cabin_interaction_anchor(CabinLocker).unwrap(),
        ),
        "scout-cabin-table" => (
            [1.6, 1.9973, -1.5],
            s.cabin_interaction_anchor(CabinTable).unwrap(),
        ),
        _ => (
            [0.8, 1.9973, -0.2],
            s.cabin_interaction_anchor(CabinSeat).unwrap(),
        ),
    };
    let d = sub(target, eye);
    let mut v = serde_json::to_value(s.character.save_state()).map_err(|e| e.to_string())?;
    v["position"] = serde_json::json!({"Inside":{"local":eye}});
    v["yaw_radians"] = d[2].atan2(d[0]).rem_euclid(std::f64::consts::TAU).into();
    v["pitch_radians"] = (d[1] / length(d)).asin().into();
    s.character = salimon_character::CharacterController::from_save(
        serde_json::from_value(v).map_err(|e| e.to_string())?,
        None,
    )
    .map_err(|e| e.to_string())?;
    s.sync_mount_collision();
    Ok(())
}
