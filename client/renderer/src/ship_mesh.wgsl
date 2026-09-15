struct ShipUniform {
    view_projection: mat4x4<f32>,
    center: vec4<f32>,
    forward: vec4<f32>,
    up: vec4<f32>,
    port: vec4<f32>,
    door_offset: vec4<f32>,
    padding: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> ship: ShipUniform;
@group(0) @binding(1)
var instruments: texture_2d<f32>;
@group(0) @binding(2)
var instrument_sampler: sampler;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) light: vec3<f32>,
    @location(2) emissive: vec3<f32>,
    @location(3) screen_uv: vec2<f32>,
    @location(4) @interpolate(flat) display_panel: u32,
};

@vertex
fn vs_main(
    @location(0) local_position: vec3<f32>,
    @location(1) local_normal: vec3<f32>,
    @location(2) color: vec4<f32>,
    @location(3) door_flag: f32,
    @location(4) emissive: vec3<f32>,
    @location(5) interior_flag: f32,
    @location(6) display: vec3<f32>,
) -> VertexOutput {
    let local = local_position + ship.door_offset.xyz * door_flag;
    let relative = ship.center.xyz
        + ship.forward.xyz * local.x
        + ship.up.xyz * local.y
        + ship.port.xyz * local.z;
    let normal = normalize(
        ship.forward.xyz * local_normal.x
        + ship.up.xyz * local_normal.y
        + ship.port.xyz * local_normal.z
    );
    var output: VertexOutput;
    output.clip_position = ship.view_projection * vec4<f32>(relative, 1.0);
    output.color = color;
    let key = normalize(vec3<f32>(0.35, 0.82, 0.44));
    let exterior_light = vec3<f32>(0.32 + 0.68 * abs(dot(normal, key)));
    // A restrained, ship-local cabin fill stays warm as the ship rotates. This
    // is evaluated per vertex; it needs no light loops, textures, or shadows.
    let cabin_light = vec3<f32>(0.76, 0.69, 0.59)
        + vec3<f32>(0.16) * abs(local_normal.y);
    output.light = mix(exterior_light, cabin_light, interior_flag);
    output.emissive = emissive;
    output.screen_uv = display.xy;
    output.display_panel = u32(display.z);
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    if input.display_panel > 0u {
        // Clamp inside each atlas tile to prevent sampling its neighbor at an
        // edge. Explicit LOD permits sampling in this per-surface branch.
        let size = vec2<f32>(textureDimensions(instruments));
        let tile_size = vec2<f32>(size.x, size.y / 3.0);
        let pixel = vec2<f32>(0.5) + clamp(input.screen_uv, vec2<f32>(0.0), vec2<f32>(1.0))
            * (tile_size - vec2<f32>(1.0));
        let uv = (pixel + vec2<f32>(0.0, f32(input.display_panel - 1u) * tile_size.y)) / size;
        return textureSampleLevel(instruments, instrument_sampler, uv, 0.0);
    }
    // Keep lamps, displays, and engine/core accents self-lit instead of
    // clamping their emission into the base color and then darkening it.
    return vec4<f32>(input.color.rgb * input.light + input.emissive, input.color.a);
}
