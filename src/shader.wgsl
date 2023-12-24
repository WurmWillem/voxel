// Vertex shader

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

struct InstanceInput {
    @location(5) model_matrix_0: vec4<f32>,
    @location(6) model_matrix_1: vec4<f32>,
    @location(7) model_matrix_2: vec4<f32>,
    @location(8) model_matrix_3: vec4<f32>,
};

struct CameraUniform {
    view_proj: mat4x4<f32>,
};
@group(1) @binding(0)
var<uniform> camera: CameraUniform; 

@group(2) @binding(0)
var<uniform> count: f32; 

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    var model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );
    if count != 0. {
        let s = 0.01;
        let new_y = (sin((model_matrix.w.x + count) * s) + sin((model_matrix.w.z + count) * s * 0.543));
        model_matrix.w.y += new_y * 3.;
    }
    
    // model_matrix.w.y *= 50.;

    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    var clip_pos = camera.view_proj * model_matrix  * vec4<f32>(model.position, 1.0);
    // clip_pos.y = sin(clip_pos.x * 0.1);
    out.clip_position = clip_pos;
    return out;
}

// Fragment shader

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0)@binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    //return vec4<f32>(in.color, 1.0);
    return textureSample(t_diffuse, s_diffuse, in.tex_coords);
}