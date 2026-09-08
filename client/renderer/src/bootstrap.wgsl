struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    let positions = array(
        vec2<f32>(0.0, 0.62),
        vec2<f32>(-0.58, -0.48),
        vec2<f32>(0.58, -0.48),
    );
    let colors = array(
        vec3<f32>(0.24, 0.91, 0.82),
        vec3<f32>(0.18, 0.43, 0.88),
        vec3<f32>(0.96, 0.58, 0.20),
    );

    var output: VertexOutput;
    output.clip_position = vec4<f32>(positions[vertex_index], 0.0, 1.0);
    output.color = colors[vertex_index];
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(input.color, 1.0);
}
