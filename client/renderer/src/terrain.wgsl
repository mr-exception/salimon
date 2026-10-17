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
    @location(3) facet_weight: f32,
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
    // Distance fades shading only. Vertex morph, shared borders and depth stay
    // unchanged; outer transition and startup converge to the analytic surface.
    out.facet_weight = (1.0 - blend) * (1.0 - smoothstep(80.0, 240.0, length(local + terrain_uniform.offset.xyz)));
    return out;
}
@fragment fn fs_main(in: Output) -> @location(0) vec4<f32> {
    let radial = normalize(in.radial);
    // Geometric derivatives expose existing triangles without expanding vertices
    // or transporting extra normals. Orient outward regardless of screen winding.
    let face = cross(dpdx(in.local), dpdy(in.local));
    let outward = select(-face, face, dot(face, radial) >= 0.0);
    let face_normal = select(normalize(in.normal), outward * inverseSqrt(max(dot(face, face), 1e-20)), dot(face, face) > 1e-20);
    let n = normalize(mix(normalize(in.normal), face_normal, in.facet_weight));
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
    return vec4(base*(0.94+0.12*detail)*(vec3(0.22)+0.78*diffuse*terrain_uniform.material_light_color.yzw),1.0);
}
