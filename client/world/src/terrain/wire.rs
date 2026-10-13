use super::*;

impl TerrainDefinition {
    /// Canonical little-endian encoding doubles as the complete cache identity.
    /// No world center is included; translating a planet leaves terrain unchanged.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = b"SLMTERR1".to_vec();
        bytes.extend(self.version.to_le_bytes());
        bytes.push(body_tag(self.body));
        bytes.extend(self.seed.to_le_bytes());
        bytes.extend(self.radius_meters.to_le_bytes());
        bytes.push(self.layers.len() as u8);
        bytes.push(self.features.len() as u8);
        bytes.extend([0; 3]);
        for layer in &self.layers {
            bytes.push(match layer.kind {
                LayerKind::Hills => 0,
                LayerKind::Ridges => 1,
            });
            bytes.extend(layer.amplitude_meters.to_le_bytes());
            bytes.extend(layer.wavelength_meters.to_le_bytes());
        }
        for feature in &self.features {
            for value in feature
                .direction
                .into_iter()
                .chain([feature.radius_meters, feature.elevation_meters])
            {
                bytes.extend(value.to_le_bytes());
            }
        }
        bytes
    }
    /// Decode only against a known body. Reject mismatched identity/radius,
    /// malformed lengths, trailing data and versions rather than falling back.
    pub fn from_bytes(body: CelestialBody, bytes: &[u8]) -> Result<Self, TerrainError> {
        if bytes.len() < 32 || bytes.len() > MAX_DEFINITION_BYTES || &bytes[..8] != b"SLMTERR1" {
            return Err(TerrainError::InvalidEncoding);
        }
        let version = u16::from_le_bytes([bytes[8], bytes[9]]);
        if version != GENERATOR_VERSION {
            return Err(TerrainError::UnsupportedVersion);
        }
        let layers_count = usize::from(bytes[27]);
        let features_count = usize::from(bytes[28]);
        if bytes[10] != body_tag(body.id)
            || bytes[29..32] != [0; 3]
            || layers_count > MAX_LAYERS
            || features_count > MAX_FEATURES
            || bytes.len() != 32 + layers_count * 17 + features_count * 40
        {
            return Err(TerrainError::InvalidEncoding);
        }
        let mut cursor = Cursor { bytes, offset: 11 };
        let seed = u64::from_le_bytes(cursor.word());
        let radius = cursor.float();
        if radius.to_bits() != body.radius_meters.to_bits() {
            return Err(TerrainError::InvalidDefinition);
        }
        cursor.offset = 32;
        let mut layers = Vec::with_capacity(layers_count);
        for _ in 0..layers_count {
            let kind = match bytes[cursor.offset] {
                0 => LayerKind::Hills,
                1 => LayerKind::Ridges,
                _ => return Err(TerrainError::InvalidEncoding),
            };
            cursor.offset += 1;
            layers.push(Layer {
                kind,
                amplitude_meters: cursor.float(),
                wavelength_meters: cursor.float(),
            });
        }
        let mut features = Vec::with_capacity(features_count);
        for _ in 0..features_count {
            features.push(Feature {
                direction: [cursor.float(), cursor.float(), cursor.float()],
                radius_meters: cursor.float(),
                elevation_meters: cursor.float(),
            });
        }
        Self::new(body, version, seed, layers, features)
    }
}
// Whole-record size and bounded counts are validated before any cursor read.
struct Cursor<'a> {
    bytes: &'a [u8],
    offset: usize,
}
impl Cursor<'_> {
    fn word(&mut self) -> [u8; 8] {
        let mut word = [0; 8];
        word.copy_from_slice(&self.bytes[self.offset..self.offset + 8]);
        self.offset += 8;
        word
    }
    fn float(&mut self) -> f64 {
        f64::from_le_bytes(self.word())
    }
}
fn body_tag(id: CelestialBodyId) -> u8 {
    match id {
        CelestialBodyId::Sun => 0,
        CelestialBodyId::Mercury => 1,
        CelestialBodyId::Venus => 2,
        CelestialBodyId::Earth => 3,
        CelestialBodyId::Moon => 4,
        CelestialBodyId::Mars => 5,
    }
}
