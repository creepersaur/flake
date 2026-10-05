struct InstanceInput {
    @location(5) model_matrix_0: vec4<f32>,
    @location(6) model_matrix_1: vec4<f32>,
    @location(7) model_matrix_2: vec4<f32>,
    @location(8) model_matrix_3: vec4<f32>,
    @location(9) model_color: vec4<f32>,
    @location(10) shape: u32,
    @location(11) tri_point_1: vec2<f32>,
    @location(12) tri_point_2: vec2<f32>,
    @location(13) tri_point_3: vec2<f32>,
    @location(14) uv_rect: vec4<f32>,
};


// Vertex shader
struct CameraUniform {
    view_proj: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: CameraUniform;
@group(1) @binding(0) var t_diffuse: texture_2d<f32>;
@group(1) @binding(1) var s_diffuse: sampler;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) @interpolate(flat) shape: u32,
    @location(2) uv: vec2<f32>,
    @location(3) tex_uv: vec2<f32>,
}

@vertex
fn vs_main(
    @builtin(vertex_index) vertex_index: u32,
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );

    var out: VertexOutput;
    out.color = instance.model_color;
    out.shape = instance.shape;
    out.uv = model.tex_coords;
    out.tex_uv = instance.uv_rect.xy + model.tex_coords * instance.uv_rect.zw;

    if (instance.shape == 2) {
        var tri_points = array<vec2<f32>, 3>(
            instance.tri_point_1,
            instance.tri_point_2,
            instance.tri_point_3,
        );
        out.clip_position = camera.view_proj * model_matrix * vec4<f32>(tri_points[vertex_index], model.position.z, 1.0);
    } else {
        out.clip_position = camera.view_proj * model_matrix * vec4<f32>(model.position, 1.0);
    }

    return out;
}

// Fragment shader

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let tex = textureSample(t_diffuse, s_diffuse, in.tex_uv);
    let color = in.color * tex;

    if color.a <= 0.0 { discard; }

    if in.shape == 1 {
        let alpha = 1.0 - smoothstep(0.48, 0.5, length(in.uv - vec2(0.5, 0.5)));
        if alpha <= 0.0 { discard; }
        return vec4(color.rgb, color.a * alpha);
    }

    return color;
}