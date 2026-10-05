struct Placement {
    projection: mat4x4<f32>,
    status: vec4<f32>,
};
@group(0) @binding(0) var<uniform> placement: Placement;
struct Output {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
};
@vertex
fn vs_main(@location(0) point: vec3<f32>, @location(1) normal: vec3<f32>,
           @location(2) base: vec4<f32>, @location(3) indicator: f32) -> Output {
    // Rotate asset +X forward, +Y up into the right-handed camera frame. The grip is camera-local.
    let p = vec3<f32>(0.20 + point.z, -0.20 + point.y, -0.55 - point.x);
    let fill = 0.65 + 0.35 * max(dot(normal, normalize(vec3<f32>(-0.4, 0.8, -0.5))), 0.0);
    var out: Output;
    out.position = placement.projection * vec4<f32>(p, 1.0);
    let status_color = mix(vec3<f32>(0.9, 0.6, 0.15), vec3<f32>(0.2, 1.0, 0.8), placement.status.x);
    out.color = vec4<f32>(mix(base.rgb * fill, status_color, indicator), 1.0);
    return out;
}
@fragment fn fs_main(input: Output) -> @location(0) vec4<f32> { return input.color; }
