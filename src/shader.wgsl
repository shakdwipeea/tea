struct CameraUniform {
    view_proj: mat4x4<f32>,
}

@group(1) @binding(0)
var<uniform> camera: CameraUniform;

struct VertextInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(5) model_matrix_0: vec4<f32>,
    @location(6) model_matrix_1: vec4<f32>,
    @location(7) model_matrix_2: vec4<f32>,
    @location(8) model_matrix_3: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

@vertex
fn vs_main(model: VertextInput) -> VertexOutput {
    let model_matrix = mat4x4<f32>(
        model.model_matrix_0,
        model.model_matrix_1,
        model.model_matrix_2,
        model.model_matrix_3,
    );
    
    var out: VertexOutput;
    out.clip_position = camera.view_proj * model_matrix * vec4<f32>(model.position, 1.0);
    out.tex_coords = model.tex_coords;
    return out;
}

@group(0) @binding(0)
var t_diffuse : texture_2d<f32>;

@group(0) @binding(1)
var s_diffuse_sampler : sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t_diffuse, s_diffuse_sampler, in.tex_coords);
}