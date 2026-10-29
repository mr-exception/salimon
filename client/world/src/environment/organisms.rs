//! Immutable low-poly organism grammars. Cosmetic density is never physical population.
use super::{
    EnvironmentError as Error,
    ecology::{BiosphereDecision, BiosphereDefinition},
    random,
};
use salimon_universe::synthesis::sha256;
pub const ORGANISM_VERSION: u16 = 1;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Form {
    Mat,
    Column,
    Frond,
    Filament,
    Colony,
    Giant,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Part {
    pub id: u16,
    pub start: [f64; 3],
    pub end: [f64; 3],
    pub radius_start: f64,
    pub radius_end: f64,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Specimen {
    pub id: String,
    pub grammar_id: String,
    pub form: Form,
    pub color: [f32; 3],
    pub parts: Vec<Part>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Grammar {
    id: String,
    seed: u64,
}
fn key(text: &str) -> u64 {
    text.bytes().fold(0xcbf29ce484222325, |a, b| {
        (a ^ u64::from(b)).wrapping_mul(0x100000001b3)
    })
}
impl Grammar {
    pub fn from_biosphere(biosphere: &BiosphereDefinition) -> Option<Self> {
        if biosphere.decision() != BiosphereDecision::Developed {
            return None;
        }
        let id = biosphere.family_grammar_id()?;
        Some(Self { seed: key(&id), id })
    }
    pub fn id(&self) -> &str {
        &self.id
    }
    pub fn specimen(&self, form: Form, variant: u8) -> Result<Specimen, Error> {
        if variant >= 64 {
            return Err(Error::InvalidDefinition);
        }
        let seed = self.seed ^ u64::from(variant);
        let r = |slot| random(seed, slot);
        let variation = 0.85 + r(1) * 0.3;
        let palette = [[0.25, 0.42, 0.39], [0.39, 0.28, 0.43], [0.48, 0.38, 0.22]];
        let base = palette[(self.seed % 3) as usize];
        let color = base.map(|c| (c * (0.9 + r(2) * 0.2)) as f32);
        let mut parts = Vec::new();
        let mut add = |start, end, a, b| {
            parts.push(Part {
                id: parts.len() as u16,
                start,
                end,
                radius_start: a * variation,
                radius_end: b * variation,
            });
        };
        match form {
            Form::Mat => {
                for i in 0..6 {
                    let angle = i as f64 * std::f64::consts::TAU / 6.;
                    let x = angle.cos() * 0.35;
                    let z = angle.sin() * 0.35;
                    add([x, 0., z], [x, 0.12 * variation, z], 0.3, 0.12);
                }
            }
            Form::Column => {
                let height = 2.5 * variation;
                add([0.; 3], [0., height, 0.], 0.32, 0.16);
                for i in 0..4 {
                    let y = height * (i as f64 + 1.) / 5.;
                    add([0., y, 0.], [0., y + 0.15, 0.], 0.45, 0.4);
                }
            }
            Form::Frond => {
                let height = 2. * variation;
                add([0.; 3], [0., height, 0.], 0.09, 0.025);
                for i in 0..6 {
                    let y = (i as f64 + 1.) * height / 7.;
                    let side = if i % 2 == 0 { 1. } else { -1. };
                    add(
                        [0., y, 0.],
                        [side * (0.9 - 0.07 * i as f64), y + 0.35, 0.],
                        0.19,
                        0.012,
                    );
                }
            }
            Form::Filament => {
                for i in 0..5 {
                    let a = i as f64 * 2.4;
                    let end = [
                        a.cos() * 0.25,
                        0.65 * variation + r(i + 10) * 0.3,
                        a.sin() * 0.25,
                    ];
                    add([end[0] * 0.2, 0., end[2] * 0.2], end, 0.025, 0.005);
                }
            }
            Form::Colony => {
                for i in 0..7 {
                    let a = i as f64 * 2.4;
                    let x = a.cos() * 0.6;
                    let z = a.sin() * 0.6;
                    let y = (0.5 + r(i + 20) * 0.7) * variation;
                    add([x, 0., z], [x, y, z], 0.07, 0.04);
                    add([x, y, z], [x, y + 0.15, z], 0.27, 0.08);
                }
            }
            Form::Giant => {
                let height = 12. * variation;
                add([0.; 3], [0., height, 0.], 1.2, 0.3);
                for i in 0..5 {
                    let a = i as f64 * std::f64::consts::TAU / 5.;
                    let root = [a.cos() * 2.5, 0., a.sin() * 2.5];
                    add(root, [0., 3., 0.], 0.3, 0.7);
                    add(
                        [0., height * 0.65, 0.],
                        [a.cos() * 3.5, height, a.sin() * 3.5],
                        0.5,
                        0.08,
                    );
                }
            }
        }
        let id = sha256(
            format!("organism-{ORGANISM_VERSION}:{}:{form:?}:{variant}", self.id).as_bytes(),
        );
        Ok(Specimen {
            id,
            grammar_id: self.id.clone(),
            form,
            color,
            parts,
        })
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CoverTile {
    pub face: u8,
    pub u: u16,
    pub v: u16,
}
#[derive(Clone, Debug, PartialEq)]
pub struct CosmeticCover {
    pub id: String,
    pub position_meters: [f64; 3],
    pub outward_normal: [f64; 3],
    pub specimen: Specimen,
}
/// Stable cube-face tile, independent of viewport/LOD. Quality admits a prefix of
/// small cosmetic candidates; no physical collision/resource/save identity derives from it.
pub fn cosmetic_tile(
    biosphere: &BiosphereDefinition,
    tile: CoverTile,
    candidates: u8,
) -> Result<Vec<CosmeticCover>, Error> {
    if tile.face >= 6 || candidates > 64 {
        return Err(Error::InvalidDefinition);
    }
    let Some(grammar) = Grammar::from_biosphere(biosphere) else {
        return Ok(vec![]);
    };
    let mut out = Vec::new();
    for slot in 0..candidates {
        let id = sha256(
            format!(
                "cover-{ORGANISM_VERSION}:{}:{tile:?}:{slot}",
                biosphere.identity()
            )
            .as_bytes(),
        );
        let seed = key(&id);
        let u =
            (f64::from(tile.u) + (f64::from(slot % 8) + random(seed, 0)) / 8.) / 65536. * 2. - 1.;
        let v =
            (f64::from(tile.v) + (f64::from(slot / 8) + random(seed, 1)) / 8.) / 65536. * 2. - 1.;
        let d = match tile.face {
            0 => [1., u, v],
            1 => [-1., u, -v],
            2 => [u, 1., v],
            3 => [u, -1., -v],
            4 => [u, v, 1.],
            _ => [-u, v, -1.],
        };
        let n = d[0].hypot(d[1]).hypot(d[2]);
        let d = d.map(|x| x / n);
        let ecology = biosphere.sample(d)?;
        if random(seed, 2) > ecology.macro_cover_fraction {
            continue;
        }
        let surface = biosphere.climate().geology().terrain().sample(d)?;
        let form = if ecology.community_weights[1] > ecology.community_weights[0] {
            Form::Filament
        } else {
            Form::Mat
        };
        let specimen = grammar.specimen(form, slot)?;
        out.push(CosmeticCover {
            id,
            position_meters: surface.body_local_position_meters,
            outward_normal: surface.outward_normal,
            specimen,
        });
    }
    Ok(out)
}
#[cfg(test)]
pub(crate) mod tests;
