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

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) normal: vec3<f32>,
};

@vertex
fn vs_main(
    @location(0) local_position: vec3<f32>,
    @location(1) local_normal: vec3<f32>,
    @location(2) color: vec4<f32>,
    @location(3) door_flag: f32,
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
    output.normal = normal;
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let key = normalize(vec3<f32>(0.35, 0.82, 0.44));
    let light = 0.32 + 0.68 * abs(dot(input.normal, key));
    return vec4<f32>(input.color.rgb * light, input.color.a);
}
