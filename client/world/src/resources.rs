//! Renderer-independent contracts for raw planetary resources. Quantities use SI units.

use crate::{CelestialBodyId, WorldPosition};

/// Material identity is independent of catalog order and display names.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ResourceId {
    IronOre,
    SilicateRock,
    WaterIce,
}

impl ResourceId {
    /// Stable external key; do not serialize enum discriminants or catalog indices.
    pub const fn key(self) -> &'static str {
        match self {
            Self::IronOre => "iron-ore",
            Self::SilicateRock => "silicate-rock",
            Self::WaterIce => "water-ice",
        }
    }

    pub fn from_key(key: &str) -> Option<Self> {
        RESOURCE_CATALOG
            .iter()
            .find(|entry| entry.id.key() == key)
            .map(|entry| entry.id)
    }

    pub fn definition(self) -> &'static ResourceDefinition {
        RESOURCE_CATALOG
            .iter()
            .find(|entry| entry.id == self)
            .expect("every resource has a catalog entry")
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ResourceDefinition {
    pub id: ResourceId,
    pub name: &'static str,
    /// Effective raw-material density, a gameplay approximation rather than ore purity.
    pub density_kg_per_m3: f64,
}

/// Small immutable starting catalog. No colors, meshes, shaders, or texture handles.
pub static RESOURCE_CATALOG: &[ResourceDefinition] = &[
    ResourceDefinition {
        id: ResourceId::IronOre,
        name: "Iron ore",
        density_kg_per_m3: 3_500.0,
    },
    ResourceDefinition {
        id: ResourceId::SilicateRock,
        name: "Silicate rock",
        density_kg_per_m3: 2_700.0,
    },
    ResourceDefinition {
        id: ResourceId::WaterIce,
        name: "Water ice",
        density_kg_per_m3: 920.0,
    },
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResourceError {
    InvalidMass,
    InvalidVolume,
    InvalidTransform,
    RemainingMassExceedsInitial,
    RemainingMassIncreased,
}

impl std::fmt::Display for ResourceError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for ResourceError {}

/// Positive finite raw mass and derived solid volume; zero-mass fragments do not exist.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RawMaterial {
    resource: ResourceId,
    mass_kg: f64,
    volume_m3: f64,
}

impl RawMaterial {
    pub fn new(resource: ResourceId, mass_kg: f64) -> Result<Self, ResourceError> {
        if !mass_kg.is_finite() || mass_kg <= 0.0 {
            return Err(ResourceError::InvalidMass);
        }
        let volume_m3 = mass_kg / resource.definition().density_kg_per_m3;
        if !volume_m3.is_finite() || volume_m3 <= 0.0 {
            return Err(ResourceError::InvalidVolume);
        }
        Ok(Self {
            resource,
            mass_kg,
            volume_m3,
        })
    }

    pub const fn resource(self) -> ResourceId {
        self.resource
    }
    pub const fn mass_kg(self) -> f64 {
        self.mass_kg
    }
    pub const fn volume_m3(self) -> f64 {
        self.volume_m3
    }
}

/// Generator-assigned local identity, scoped to a celestial body. Streaming must reuse it.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DepositId {
    pub body: CelestialBodyId,
    pub local: u64,
}

/// Fragment IDs are allocated uniquely within a world/session, separately from deposit IDs.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct FragmentId(pub u64);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DepositState {
    Untouched,
    PartiallyMined,
    Depleted,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ResourceDeposit {
    id: DepositId,
    material: RawMaterial,
    position: WorldPosition,
    remaining_mass_kg: f64,
}

impl ResourceDeposit {
    /// Also accepts restored partial/depleted state without changing original material identity.
    pub fn new(
        id: DepositId,
        material: RawMaterial,
        position: WorldPosition,
        remaining_mass_kg: f64,
    ) -> Result<Self, ResourceError> {
        if !position.is_finite() {
            return Err(ResourceError::InvalidTransform);
        }
        validate_remaining(material.mass_kg(), remaining_mass_kg)?;
        Ok(Self {
            id,
            material,
            position,
            remaining_mass_kg,
        })
    }

    pub const fn id(self) -> DepositId {
        self.id
    }
    pub const fn material(self) -> RawMaterial {
        self.material
    }
    pub const fn position(self) -> WorldPosition {
        self.position
    }
    pub const fn remaining_mass_kg(self) -> f64 {
        self.remaining_mass_kg
    }
    pub fn remaining_volume_m3(self) -> f64 {
        self.remaining_mass_kg / self.material.resource().definition().density_kg_per_m3
    }
    pub fn state(self) -> DepositState {
        if self.remaining_mass_kg == 0.0 {
            DepositState::Depleted
        } else if self.remaining_mass_kg == self.material.mass_kg() {
            DepositState::Untouched
        } else {
            DepositState::PartiallyMined
        }
    }

    /// Local extraction state may only decrease; failed updates leave the deposit intact.
    pub fn set_remaining_mass_kg(&mut self, mass_kg: f64) -> Result<(), ResourceError> {
        validate_remaining(self.material.mass_kg(), mass_kg)?;
        if mass_kg > self.remaining_mass_kg {
            return Err(ResourceError::RemainingMassIncreased);
        }
        self.remaining_mass_kg = mass_kg;
        Ok(())
    }
}

fn validate_remaining(initial: f64, remaining: f64) -> Result<(), ResourceError> {
    if !remaining.is_finite() || remaining < 0.0 {
        return Err(ResourceError::InvalidMass);
    }
    if remaining > initial {
        return Err(ResourceError::RemainingMassExceedsInitial);
    }
    Ok(())
}

/// Absolute meters and a unit quaternion in [x, y, z, w] order, with no visual scale.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ResourceTransform {
    position: WorldPosition,
    orientation_xyzw: [f64; 4],
}

impl ResourceTransform {
    pub fn new(position: WorldPosition, orientation_xyzw: [f64; 4]) -> Result<Self, ResourceError> {
        let norm_squared: f64 = orientation_xyzw.iter().map(|value| value * value).sum();
        if !position.is_finite() || !norm_squared.is_finite() || (norm_squared - 1.0).abs() > 1e-9 {
            return Err(ResourceError::InvalidTransform);
        }
        // Canonicalize accepted rounding error so consumers receive a unit quaternion.
        let norm = norm_squared.sqrt();
        Ok(Self {
            position,
            orientation_xyzw: orientation_xyzw.map(|value| value / norm),
        })
    }
    pub const fn position(self) -> WorldPosition {
        self.position
    }
    pub const fn orientation_xyzw(self) -> [f64; 4] {
        self.orientation_xyzw
    }
}

/// A physical world object, not an inventory quantity. Pose can change; identity cannot.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ResourceFragment {
    id: FragmentId,
    source: DepositId,
    material: RawMaterial,
    transform: ResourceTransform,
}

impl ResourceFragment {
    pub const fn new(
        id: FragmentId,
        source: DepositId,
        material: RawMaterial,
        transform: ResourceTransform,
    ) -> Self {
        Self {
            id,
            source,
            material,
            transform,
        }
    }
    pub const fn id(self) -> FragmentId {
        self.id
    }
    pub const fn source(self) -> DepositId {
        self.source
    }
    pub const fn material(self) -> RawMaterial {
        self.material
    }
    pub const fn transform(self) -> ResourceTransform {
        self.transform
    }
    pub fn set_transform(&mut self, transform: ResourceTransform) {
        self.transform = transform;
    }
}
