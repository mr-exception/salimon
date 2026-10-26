struct Camera { view_projection: mat4x4<f32>, };
@group(0) @binding(0) var<uniform> camera: Camera;
struct Output {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) light: vec3<f32>,
    @location(2) local: vec3<f32>,
    @location(3) @interpolate(flat) age: vec4<f32>,
};
@vertex fn vs_main(@location(0) position: vec3<f32>, @location(1) color: vec4<f32>, @location(2) warm: vec3<f32>,
    @location(3) center_scale: vec4<f32>, @location(4) orientation: vec4<f32>, @location(5) tint_power: vec4<f32>, @location(6) illumination: vec4<f32>, @location(7) age: vec4<f32>) -> Output {
    var output: Output;
    let t = 2.0 * cross(orientation.xyz, position);
    let rotated = position + orientation.w * t + cross(orientation.xyz, t);
    output.position = camera.view_projection * vec4<f32>(center_scale.xyz + rotated * center_scale.w, 1.0);
    output.color = min(color.rgb * tint_power.rgb, vec3<f32>(1.0));
    output.light=vec3<f32>(0.10,0.14,0.20)+tint_power.w*warm;
    if illumination.w > 0.5 { output.light=illumination.rgb; }
    output.local=position;
    output.age=age;
    return output;
}
@fragment fn fs_main(input: Output) -> @location(0) vec4<f32> {
    return vec4<f32>(surface_wear(input.color,input.local,input.age)*input.light,1.0);
}
