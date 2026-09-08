use super::*;

pub(super) fn command(key: Key) -> Option<Command> {
    if let code @ (Key::LightPreset
    | Key::LightMaster
    | Key::LightFixture
    | Key::LightBrightness
    | Key::LightWarmth
    | Key::LightGeneral) = key
    {
        use crate::session::lighting::LightAction::*;
        Some(Command::Light(match code {
            Key::LightPreset => Preset,
            Key::LightMaster => Master,
            Key::LightFixture => Fixture,
            Key::LightBrightness => Brightness,
            Key::LightGeneral => General,
            _ => Warmth,
        }))
    } else {
        None
    }
}
