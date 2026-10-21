struct ShipUniform {
    view_projection: mat4x4<f32>,
    center: vec4<f32>,
    forward: vec4<f32>,
    up: vec4<f32>,
    port: vec4<f32>,
    door_pose: vec4<f32>,
    padding: vec4<f32>,
    activity: vec4<f32>, // conversion, signed thrust, Fabricator mode, active phase
    detail: vec4<f32>, // Core charge, recipe progress, cockpit authority, ramped rudder yaw
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
    @location(4) @interpolate(flat) display_flag: u32,
    @location(5) local: vec3<f32>,
    @location(6) @interpolate(flat) machinery_role: u32,
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
    @location(7) machinery_role: f32,
) -> VertexOutput {
    let angle = ship.door_pose.x * min(door_flag, 1.0);
    let cosine = cos(angle);
    let sine = sin(angle);
    let hinge = ship.door_pose.yz;
    let tip = select(0.0, 1.0, door_flag > 1.5);
    // Decorative pedal travel only; authority/colliders and structural supports
    // remain unchanged. Power-gated ramped yaw moves the two pedals in opposition.
    let pedal = select(0.0, -0.015 * ship.detail.w * sign(local_position.z), machinery_role == 5.0);
    let offset = local_position.xy + vec2<f32>(pedal, 0.0) + ship.padding.yx * tip - hinge;
    let local = vec3<f32>(
        hinge.x + cosine * offset.x - sine * offset.y,
        hinge.y + sine * offset.x + cosine * offset.y,
        local_position.z,
    );
    let relative = ship.center.xyz
        + ship.forward.xyz * local.x
        + ship.up.xyz * local.y
        + ship.port.xyz * local.z;
    let hinged_normal = vec3<f32>(
        cosine * local_normal.x - sine * local_normal.y,
        sine * local_normal.x + cosine * local_normal.y,
        local_normal.z,
    );
    let normal = normalize(
        ship.forward.xyz * hinged_normal.x
        + ship.up.xyz * hinged_normal.y
        + ship.port.xyz * hinged_normal.z
    );
    var output: VertexOutput;
    output.clip_position = ship.view_projection * vec4<f32>(relative, 1.0);
    output.color = color;
    let key = normalize(vec3<f32>(0.35, 0.82, 0.44));
    let exterior_light = vec3<f32>(0.82, 0.90, 1.0)
        * (0.28 + 0.62 * abs(dot(normal, key)));
    // Ship-local work zones, evaluated per vertex. Deliberately bounded fill,
    // not PBR, shadowing or physically simulated light transport. center.w is
    // the explicit bus state shared with the screen atlas.
    let powered = ship.center.w;
    let cockpit = 1.0 - smoothstep(1.0, 5.0, abs(local.x - 4.0));
    let workshop = 1.0 - smoothstep(1.0, 4.0, abs(local.x + 3.0));
    let task_fill = max(cockpit, workshop);
    let ceiling = 0.12 * abs(local_normal.y);
    let cold_spill = vec3<f32>(0.10, 0.14, 0.20);
    let cabin_light = cold_spill + powered
        * (vec3<f32>(0.46, 0.34, 0.22)
        + vec3<f32>(0.18, 0.14, 0.09) * task_fill + vec3<f32>(ceiling));
    output.light = mix(exterior_light, cabin_light, interior_flag);
    // Authored electrical lamps/accents extinguish on bus loss. Unpowered
    // surfaces retain a cool readability floor; this is not emergency power.
    var intensity = 1.0;
    if machinery_role == 1.0 {
        // Running lamps are separate surfaces. No drive glow when coasting.
        intensity = abs(ship.activity.y);
    } else if machinery_role == 2.0 {
        let breath = 0.9 + 0.1 * cos(ship.activity.w * 6.2831853);
        intensity = 0.25 + 0.5 * ship.detail.x + 0.25 * ship.activity.x * breath;
    } else if machinery_role == 3.0 {
        intensity = ship.detail.x;
    } else if machinery_role == 6.0 {
        intensity = 0.12 + 0.88 * ship.detail.z;
    }
    output.emissive = emissive * powered * intensity;
    output.screen_uv = display.xy;
    output.display_flag = u32(display.z);
    output.local = local;
    output.machinery_role = u32(machinery_role);
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    if input.display_flag == 3u && ship.padding.z < 0.5 {
        discard;
    }
    if input.display_flag == 3u {
        // One anchored package, styled by output category. Independent of bus power.
        let uv = input.screen_uv;
        let cross = (abs(uv.x - 0.5) < 0.09 && abs(uv.y - 0.5) < 0.3)
            || (abs(uv.y - 0.5) < 0.09 && abs(uv.x - 0.5) < 0.3);
        let medical = select(vec3<f32>(0.85, 0.88, 0.85), vec3<f32>(0.85, 0.08, 0.06), cross);
        let ration = select(vec3<f32>(0.64, 0.42, 0.18), vec3<f32>(0.18, 0.28, 0.08), uv.y > 0.35 && uv.y < 0.65);
        return vec4<f32>(select(ration, medical, ship.padding.w > 0.5), 1.0);
    }
    if input.display_flag == 1u || input.display_flag == 2u || input.display_flag == 4u || input.display_flag == 5u {
        // Clamp inside the single display texture at an
        // edge. Explicit LOD permits sampling in this per-surface branch.
        let size = vec2<f32>(textureDimensions(instruments));
        let core = input.display_flag == 2u;
        let machine = input.display_flag == 4u;
        let archive = input.display_flag == 5u;
        let section = size.x / 6.0;
        let region = vec2<f32>(select(section * 3.0, section, core || machine || archive), size.y);
        let origin = vec2<f32>(select(select(select(0.0, section * 3.0, core), section * 4.0, machine), section * 5.0, archive), 0.0);
        let pixel = origin + vec2<f32>(0.5) + clamp(input.screen_uv, vec2<f32>(0.0), vec2<f32>(1.0))
            * (region - vec2<f32>(1.0));
        let uv = pixel / size;
        return textureSampleLevel(instruments, instrument_sampler, uv, 0.0);
    }
    if input.machinery_role == 4u {
        // Progress-driven work sweep, steady amber for blocked output, quiet idle.
        // Actual recipe progress controls strength; no movement during no-power.
        let working = ship.activity.z == 1.0;
        let blocked = ship.activity.z == 2.0;
        let band = 1.0 - smoothstep(0.02, 0.10, abs(input.local.x - (-4.075 + 1.85 * ship.detail.y)));
        let idle = vec3<f32>(0.015, 0.04, 0.04);
        let work = vec3<f32>(0.06, 0.28, 0.30) * (0.3 + band * (0.4 + 0.3 * ship.detail.y));
        let status = select(select(idle, work, working), vec3<f32>(0.30, 0.14, 0.025), blocked);
        return vec4<f32>(input.color.rgb * input.light + status * ship.center.w, input.color.a);
    }
    // Keep lamps, displays, and engine/core accents self-lit instead of
    // clamping their emission into the base color and then darkening it.
    return vec4<f32>(input.color.rgb * input.light + input.emissive, input.color.a);
}
