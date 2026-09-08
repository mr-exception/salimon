//! Moving spherical solids and authoritative streamed collision triangles.
use super::{Bounds, CubicPath, MotionPath};
use crate::surface::{error, finite};
use salimon_math::{add, cross, dot, length, scale, sub};
use salimon_universe::{CatalogId, Error, coordinates::GalacticPosition};

#[derive(Clone, Debug)]
pub(super) enum Shape {
    Sphere {
        center: CubicPath,
        radius: f64,
        spin: [f64; 3],
    },
    Triangle {
        vertices: Box<[CubicPath; 3]>,
    },
}
#[derive(Clone, Debug)]
pub struct Collider {
    pub(super) body: CatalogId,
    pub(super) primitive: u64,
    pub(super) shape: Shape,
}
impl Collider {
    pub fn sphere(
        body: CatalogId,
        center: CubicPath,
        radius_meters: f64,
        spin_rps: [f64; 3],
    ) -> Result<Self, Error> {
        finite(spin_rps)?;
        if !radius_meters.is_finite() || radius_meters <= 0.0 {
            return Err(error("contact sphere radius"));
        }
        Ok(Self {
            body,
            primitive: 0,
            shape: Shape::Sphere {
                center,
                radius: radius_meters,
                spin: spin_rps,
            },
        })
    }
    /// Stable primitive IDs come from the canonical streamed collision mesh.
    /// Every vertex includes the body's actual motion, including rotation.
    pub fn triangle(
        body: CatalogId,
        primitive: u64,
        vertices: [CubicPath; 3],
    ) -> Result<Self, Error> {
        if primitive == 0
            || vertices.iter().any(|v| {
                v.epoch() != vertices[0].epoch()
                    || v.duration_nanos() != vertices[0].duration_nanos()
            })
        {
            return Err(error("terrain triangle identity/interval"));
        }
        let result = Self {
            body,
            primitive,
            shape: Shape::Triangle {
                vertices: Box::new(vertices),
            },
        };
        for n in [
            0,
            result.path().duration_nanos() / 2,
            result.path().duration_nanos(),
        ] {
            result.closest(result.path().position(n)?, n)?;
        }
        Ok(result)
    }
    pub(super) fn path(&self) -> &CubicPath {
        match &self.shape {
            Shape::Sphere { center, .. } => center,
            Shape::Triangle { vertices } => &vertices[0],
        }
    }
    pub(super) fn bounds(&self, origin: GalacticPosition) -> Result<Bounds, Error> {
        match &self.shape {
            Shape::Sphere { center, radius, .. } => {
                let b = center.bounds(origin)?;
                Ok(Bounds {
                    min: b.min.map(|x| x - radius),
                    max: b.max.map(|x| x + radius),
                })
            }
            Shape::Triangle { vertices } => {
                let bounds = vertices
                    .iter()
                    .map(|v| v.bounds(origin))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Bounds {
                    min: std::array::from_fn(|i| {
                        bounds
                            .iter()
                            .map(|b| b.min[i])
                            .fold(f64::INFINITY, f64::min)
                    }),
                    max: std::array::from_fn(|i| {
                        bounds
                            .iter()
                            .map(|b| b.max[i])
                            .fold(f64::NEG_INFINITY, f64::max)
                    }),
                })
            }
        }
    }
    pub(super) fn interval_clearance_bound(
        &self,
        ship_bounds: Bounds,
        origin: GalacticPosition,
        start: i64,
        end: i64,
        frame: [f64; 3],
        radius: f64,
    ) -> Result<f64, Error> {
        match &self.shape {
            Shape::Sphere {
                center,
                radius: body_radius,
                ..
            } => Ok(
                ship_bounds.distance(center.interval_bounds(origin, start, end, frame)?)
                    - body_radius
                    - radius,
            ),
            Shape::Triangle { vertices } => {
                let bounds = vertices
                    .iter()
                    .map(|v| v.interval_bounds(origin, start, end, frame))
                    .collect::<Result<Vec<_>, _>>()?;
                let whole = Bounds {
                    min: std::array::from_fn(|i| {
                        bounds
                            .iter()
                            .map(|b| b.min[i])
                            .fold(f64::INFINITY, f64::min)
                    }),
                    max: std::array::from_fn(|i| {
                        bounds
                            .iter()
                            .map(|b| b.max[i])
                            .fold(f64::NEG_INFINITY, f64::max)
                    }),
                };
                Ok(ship_bounds.distance(whole) - radius)
            }
        }
    }
    pub(super) fn relative_speed_bound(&self, ship: &dyn MotionPath) -> Result<f64, Error> {
        let frame = self.path().state_at(0)?.velocity_mps;
        let body_bound = match &self.shape {
            Shape::Sphere { center, .. } => center.speed_bound_relative_to(frame)?,
            Shape::Triangle { vertices } => vertices
                .iter()
                .map(|v| v.speed_bound_relative_to(frame))
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .fold(0.0, f64::max),
        };
        let bound = ship.speed_bound_relative_to(frame)? + body_bound;
        if !bound.is_finite() {
            return Err(error("contact relative speed bound"));
        }
        Ok(bound)
    }
    pub(super) fn closest(&self, ship: GalacticPosition, nanos: i64) -> Result<Closest, Error> {
        match &self.shape {
            Shape::Sphere {
                center,
                radius,
                spin,
            } => {
                let c = center.state_at(nanos)?;
                let r = ship.relative_meters(c.position)?;
                let distance = length(r);
                if !distance.is_finite() {
                    return Err(error("contact sphere center penetration"));
                }
                let normal = if distance > 0.0 {
                    scale(r, distance.recip())
                } else {
                    [1.0, 0.0, 0.0]
                };
                let velocity = add(c.velocity_mps, cross(*spin, scale(normal, *radius)));
                finite(velocity)?;
                Ok(Closest {
                    distance: distance - radius,
                    normal,
                    velocity,
                })
            }
            Shape::Triangle { vertices } => {
                let states = vertices
                    .iter()
                    .map(|v| v.state_at(nanos))
                    .collect::<Result<Vec<_>, _>>()?;
                let a = states[0].position.relative_meters(ship)?;
                let b = states[1].position.relative_meters(ship)?;
                let c = states[2].position.relative_meters(ship)?;
                let face = cross(sub(b, a), sub(c, a));
                let area = length(face);
                if !area.is_finite() || area < 1e-12 {
                    return Err(error("degenerate streamed contact triangle"));
                }
                let weights = closest_triangle_weights(a, b, c);
                let nearest = add(
                    add(scale(a, weights[0]), scale(b, weights[1])),
                    scale(c, weights[2]),
                );
                let distance = length(nearest);
                let normal = if distance > 0.0 {
                    scale(nearest, -distance.recip())
                } else {
                    scale(face, area.recip())
                };
                let velocity = (0..3).fold([0.0; 3], |v, i| {
                    add(v, scale(states[i].velocity_mps, weights[i]))
                });
                finite(normal)?;
                finite(velocity)?;
                if !distance.is_finite() {
                    return Err(error("terrain contact distance overflow"));
                }
                Ok(Closest {
                    distance,
                    normal,
                    velocity,
                })
            }
        }
    }
}
pub(super) struct Closest {
    pub(super) distance: f64,
    pub(super) normal: [f64; 3],
    pub(super) velocity: [f64; 3],
}
/// Ericson Voronoi regions, retaining edge/corner contacts and barycentric
/// surface velocity. Input differences are local f64 before any GPU conversion.
fn closest_triangle_weights(a: [f64; 3], b: [f64; 3], c: [f64; 3]) -> [f64; 3] {
    let ab = sub(b, a);
    let ac = sub(c, a);
    let ap = scale(a, -1.0);
    let d1 = dot(ab, ap);
    let d2 = dot(ac, ap);
    if d1 <= 0.0 && d2 <= 0.0 {
        return [1.0, 0.0, 0.0];
    }
    let bp = scale(b, -1.0);
    let d3 = dot(ab, bp);
    let d4 = dot(ac, bp);
    if d3 >= 0.0 && d4 <= d3 {
        return [0.0, 1.0, 0.0];
    }
    let vc = d1 * d4 - d3 * d2;
    if vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0 {
        let v = d1 / (d1 - d3);
        return [1.0 - v, v, 0.0];
    }
    let cp = scale(c, -1.0);
    let d5 = dot(ab, cp);
    let d6 = dot(ac, cp);
    if d6 >= 0.0 && d5 <= d6 {
        return [0.0, 0.0, 1.0];
    }
    let vb = d5 * d2 - d1 * d6;
    if vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0 {
        let w = d2 / (d2 - d6);
        return [1.0 - w, 0.0, w];
    }
    let va = d3 * d6 - d5 * d4;
    if va <= 0.0 && d4 - d3 >= 0.0 && d5 - d6 >= 0.0 {
        let w = (d4 - d3) / ((d4 - d3) + (d5 - d6));
        return [0.0, 1.0 - w, w];
    }
    let inverse = (va + vb + vc).recip();
    let v = vb * inverse;
    let w = vc * inverse;
    [1.0 - v - w, v, w]
}
