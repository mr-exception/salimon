struct Camera {
    right_tangent: vec4<f32>,
    up_tangent: vec4<f32>,
    forward_near: vec4<f32>,
    viewport: vec4<f32>,
};
@group(0) @binding(0) var<uniform> camera: Camera;
@group(0) @binding(1) var albedo: texture_2d_array<f32>;
@group(0) @binding(2) var surface_sampler: sampler;
@group(0) @binding(3) var detail_sampler: sampler;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) @interpolate(flat) direction_radius: vec4<f32>,
    @location(1) @interpolate(flat) distance_altitude_material: vec4<f32>,
    @location(2) @interpolate(flat) light_position: vec4<f32>,
    @location(3) @interpolate(flat) light_color: vec4<f32>,
    @location(4) @interpolate(flat) detail_origin: vec4<f32>,
};

@vertex
fn vs_main(
    @builtin(vertex_index) vertex: u32,
    @location(0) bounds: vec4<f32>,
    @location(1) direction_radius: vec4<f32>,
    @location(2) distance_altitude_material: vec4<f32>,
    @location(3) light_position: vec4<f32>,
    @location(4) light_color: vec4<f32>,
    @location(5) detail_origin: vec4<f32>,
) -> VertexOutput {
    let corners = array<vec2<f32>,6>(vec2(0.0,0.0),vec2(1.0,0.0),vec2(1.0,1.0),vec2(0.0,0.0),vec2(1.0,1.0),vec2(0.0,1.0));
    var out: VertexOutput;
    out.position = vec4(mix(bounds.xy,bounds.zw,corners[vertex]),0.0,1.0);
    out.direction_radius = direction_radius;
    out.distance_altitude_material = distance_altitude_material;
    out.light_position = light_position;
    out.light_color = light_color;
    out.detail_origin = detail_origin;
    return out;
}

struct FragmentOutput {
    @location(0) color: vec4<f32>,
    @builtin(frag_depth) depth: f32,
};

@fragment
fn fs_main(in: VertexOutput) -> FragmentOutput {
    let ndc = vec2(in.position.x / camera.viewport.x * 2.0 - 1.0, 1.0 - in.position.y / camera.viewport.y * 2.0);
    let ray = normalize(camera.forward_near.xyz + ndc.x * camera.right_tangent.w * camera.right_tangent.xyz + ndc.y * camera.up_tangent.w * camera.up_tangent.xyz);
    let direction = in.direction_radius.xyz;
    let radius = in.direction_radius.w;
    let distance = in.distance_altitude_material.x;
    let altitude = in.distance_altitude_material.y;
    // Cross-product form avoids subtracting nearly equal b*b and d*d at distance.
    let perpendicular = cross(ray,direction);
    let ratio = distance / radius;
    let discriminant = 1.0 - dot(perpendicular,perpendicular) * ratio * ratio;
    if discriminant < 0.0 { discard; }
    let b = dot(ray,direction) * distance;
    let root = radius * sqrt(max(discriminant,0.0));
    var hit: f32;
    if altitude >= 0.0 {
        if b <= 0.0 { discard; }
        // Rationalized near root: altitude was subtracted in CPU f64. Never
        // reconstruct surface depth by subtracting two million-metre f32 values.
        hit = altitude * ((distance + radius) / (b + root));
    } else {
        // Visual volumes such as the Sun can be viewed from inside.
        hit = b + root;
    }
    let view_depth = hit * dot(ray,camera.forward_near.xyz);
    if view_depth < camera.forward_near.w { discard; }
    let normal = normalize(ray * (hit / radius) - direction * ratio);
    let uv = vec2(atan2(normal.z,normal.x) / 6.28318530718 + 0.5, 0.5 - asin(clamp(normal.y,-1.0,1.0)) / 3.14159265359);
    // Explicit footprint LOD avoids derivative discontinuities at the longitude
    // wrap, at poles, and where fragments miss the silhouette. Trilinear mips
    // filter continuously through approach; geometry itself has no LOD switch.
    let grazing = max(abs(dot(normal,ray)),0.02);
    let longitude_scale = max(length(normal.xz),0.02);
    let footprint = 512.0 * hit * 2.0 * camera.up_tangent.w / (radius * camera.viewport.y * grazing * longitude_scale * 3.14159265359);
    let level = clamp(log2(max(footprint,1.0)),0.0,9.0);
    let base = textureSampleLevel(albedo,surface_sampler,uv,i32(in.distance_altitude_material.z),level).rgb;
    let tile = (ray * hit + in.detail_origin.xyz) / in.detail_origin.w;
    let detail_level = clamp(log2(max(512.0 * hit * 2.0 * camera.up_tangent.w / (in.detail_origin.w * camera.viewport.y * grazing),1.0)),0.0,9.0);
    let weights = abs(normal) / (abs(normal.x) + abs(normal.y) + abs(normal.z));
    let detail = weights.x * textureSampleLevel(albedo,detail_sampler,fract(tile.yz),6,detail_level).r
        + weights.y * textureSampleLevel(albedo,detail_sampler,fract(tile.xz),6,detail_level).r
        + weights.z * textureSampleLevel(albedo,detail_sampler,fract(tile.xy),6,detail_level).r;
    let surface = base * (0.65 + 0.7 * detail);
    let to_light = in.light_position.xyz - normal;
    let light_direction = to_light / max(length(to_light),0.00001);
    let diffuse = max(dot(normal,light_direction),0.0);
    let illumination = vec3(0.22) + 0.78 * diffuse * in.light_color.rgb;
    var out: FragmentOutput;
    out.color = vec4(mix(surface * illumination,surface,in.distance_altitude_material.w),1.0);
    out.depth = camera.forward_near.w / view_depth;
    return out;
}
