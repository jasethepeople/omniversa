// Vertex shader
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec4<f32>,
    @location(2) stress: f32,
    @location(3) viscosity: f32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) stress: f32,
    @location(2) world_position: vec3<f32>,
};

struct Uniforms {
    view_proj: mat4x4<f32>,
    time: f32,
    fragility: f32,
    _padding: vec2<f32>,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;

    // Add subtle animation based on stress
    let wobble = sin(uniforms.time * 2.0 + input.position.x * 10.0) * input.stress * 0.1;
    let animated_pos = input.position + vec3<f32>(wobble, wobble * 0.5, 0.0);

    output.clip_position = uniforms.view_proj * vec4<f32>(animated_pos, 1.0);
    output.color = input.color;
    output.stress = input.stress;
    output.world_position = animated_pos;

    return output;
}

// Fragment shader
@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // Base color from vertex
    var color = input.color;

    // Stress glow effect
    let glow = input.stress * 0.5;
    color.r += glow;
    color.g -= glow * 0.3;

    // Distance-based fade for depth perception
    let depth = length(input.world_position);
    let fade = 1.0 / (1.0 + depth * 0.1);
    color.a *= fade;

    // Fragility pulse
    let pulse = sin(uniforms.time * 5.0) * 0.1 + 0.9;
    color *= pulse;

    return color;
}
