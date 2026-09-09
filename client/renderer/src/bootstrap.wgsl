struct Camera {
    view_projection: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: Camera;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@vertex
fn vs_main(
    @location(0) local_position: vec3<f32>,
    @location(1) relative_center: vec3<f32>,
    @location(2) half_extents: vec3<f32>,
    @location(3) color: vec4<f32>,
) -> VertexOutput {
    let camera_relative_position = relative_center + local_position * half_extents;
    var output: VertexOutput;
    output.clip_position = camera.view_projection * vec4<f32>(camera_relative_position, 1.0);
    output.color = color;
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return input.color;
}
