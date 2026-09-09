struct OverlayDimensions {
    surface_and_image: vec4<f32>,
}

@group(0) @binding(0)
var overlay_image: texture_2d<f32>;

@group(0) @binding(1)
var<uniform> dimensions: OverlayDimensions;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    let corners = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 0.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(0.0, 1.0),
        vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(1.0, 1.0),
    );
    let corner = corners[vertex_index];
    let surface_size = dimensions.surface_and_image.xy;
    let image_size = dimensions.surface_and_image.zw;
    let margin = vec2<f32>(16.0, 16.0);
    let pixel_position = margin + corner * image_size;
    let ndc = vec2<f32>(
        pixel_position.x / surface_size.x * 2.0 - 1.0,
        1.0 - pixel_position.y / surface_size.y * 2.0,
    );

    var output: VertexOutput;
    output.position = vec4<f32>(ndc, 0.0, 1.0);
    output.uv = corner;
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let texture_size = vec2<i32>(textureDimensions(overlay_image));
    let unclamped = vec2<i32>(input.uv * vec2<f32>(texture_size));
    let texel = clamp(unclamped, vec2<i32>(0), texture_size - vec2<i32>(1));
    return textureLoad(overlay_image, texel, 0);
}
