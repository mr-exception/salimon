struct Patch {
    view_projection: mat4x4<f32>,
    offset: vec4<f32>,
    material_light_color: vec4<f32>,
    light: vec4<f32>,
    mask: vec4<f32>,
    anchor: vec4<f32>,
    detail_origin: vec4<f32>,
};
@group(0) @binding(0) var<uniform> terrain_uniform: Patch;
@group(0) @binding(1) var albedo: texture_2d_array<f32>;
@group(0) @binding(2) var surface_sampler: sampler;
@group(0) @binding(3) var detail_sampler: sampler;
struct Output {
    @builtin(position) position: vec4<f32>,
    @location(0) normal: vec3<f32>,
    @location(1) local: vec3<f32>,
    @location(2) radial: vec3<f32>,
};
@vertex fn vs_main(@location(0) position: vec3<f32>, @location(1) normal: vec3<f32>, @location(2) far_position: vec3<f32>) -> Output {
    var out: Output;
    let outer_blend = smoothstep(terrain_uniform.mask.w, terrain_uniform.mask.w * 1.5, length(position - terrain_uniform.mask.xyz));
    let blend = mix(1.0,outer_blend,terrain_uniform.anchor.w);
    let local = mix(position, far_position, blend);
    out.position = terrain_uniform.view_projection * vec4(local + terrain_uniform.offset.xyz, 1.0);
    let radial = normalize(local + terrain_uniform.anchor.xyz);
    out.normal = mix(normal,radial,blend);
    out.local = local;
    out.radial = radial;
    return out;
}
@fragment fn fs_main(in: Output) -> @location(0) vec4<f32> {
    let n = normalize(in.normal);
    let radial = normalize(in.radial);
    let uv = vec2(atan2(radial.z,radial.x)/6.28318530718+0.5,0.5-asin(clamp(radial.y,-1.0,1.0))/3.14159265359);
    let base = textureSampleLevel(albedo,surface_sampler,uv,i32(terrain_uniform.material_light_color.x),0.0).rgb;
    let tile = (in.local + terrain_uniform.detail_origin.xyz)/terrain_uniform.detail_origin.w;
    let weights = abs(n)/(abs(n.x)+abs(n.y)+abs(n.z));
    let detail = weights.x * textureSample(albedo,detail_sampler,tile.yz,6).r
        + weights.y * textureSample(albedo,detail_sampler,tile.xz,6).r
        + weights.z * textureSample(albedo,detail_sampler,tile.xy,6).r;
    let to_light=terrain_uniform.light.xyz-in.local;
    let light_direction=to_light/max(length(to_light),0.00001);
    let diffuse = max(dot(n,light_direction),0.0);
    return vec4(base*(0.65+0.7*detail)*(vec3(0.22)+0.78*diffuse*terrain_uniform.material_light_color.yzw),1.0);
}
