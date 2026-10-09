struct Placement {
    model_to_clip: mat4x4<f32>,
    status: vec4<f32>,
};
@group(0) @binding(0) var<uniform> placement: Placement;
@group(0) @binding(1) var surface: texture_2d<f32>;
@group(0) @binding(2) var surface_sampler: sampler;
struct Output {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) @interpolate(flat) material: vec3<f32>,
    @location(4) view_point: vec3<f32>,
};
@vertex
fn vs_main(@location(0) point: vec3<f32>, @location(1) normal: vec3<f32>,
           @location(2) base: vec4<f32>, @location(3) indicator: f32,
           @location(4) uv: vec2<f32>, @location(5) metallic_roughness: vec2<f32>) -> Output {
    var out: Output;
    out.position = placement.model_to_clip * vec4<f32>(point, 1.0);
    out.color = base;
    // Asset +X forward, +Y up, +Z camera-right, matching GRIP_TO_VIEW.
    out.normal = vec3<f32>(normal.z, normal.y, -normal.x);
    out.view_point = vec3<f32>(point.z + 0.20, point.y - 0.20, -point.x - 0.55);
    out.uv = uv;
    out.material = vec3<f32>(indicator, metallic_roughness);
    return out;
}
@fragment fn fs_main(input: Output) -> @location(0) vec4<f32> {
    let albedo = textureSample(surface, surface_sampler, input.uv).rgb * input.color.rgb;
    let normal = normalize(input.normal);
    let light = normalize(vec3<f32>(-0.4, 0.8, 0.5));
    let view = normalize(-input.view_point);
    let half_vector = normalize(light + view);
    let metallic = input.material.y;
    let roughness = input.material.z;
    let diffuse = 0.42 + 0.58 * max(dot(normal, light), 0.0);
    // Compact camera-local manufactured-material shading, not world PBR/IBL.
    // Authored metallic/roughness separate steel highlights from matte rubber.
    let specular = pow(max(dot(normal, half_vector), 0.0), mix(80.0, 8.0, roughness));
    let fresnel = mix(vec3<f32>(0.04), albedo, metallic);
    let shaded = albedo * diffuse + fresnel * specular * (1.0 - roughness * 0.5);
    let status_color = mix(vec3<f32>(0.9, 0.6, 0.15), vec3<f32>(0.2, 1.0, 0.8), placement.status.x);
    return vec4<f32>(mix(shaded, status_color, input.material.x), 1.0);
}
