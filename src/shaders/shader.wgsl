struct InstanceInput {
    @location(5)  pos: vec3<f32>,
    @location(6)  rotation: f32,
    @location(7)  size: vec2<f32>,
    @location(8)  radius: vec4<f32>,
    @location(9)  model_color: vec4<f32>, // Unorm8x4 arrives as 0..1 floats
    @location(10) shape: u32,
    @location(11) tri_point_1: vec2<f32>,
    @location(12) tri_point_2: vec2<f32>,
    @location(13) tri_point_3: vec2<f32>,
    @location(14) uv_rect: vec4<f32>,
};

struct CameraUniform {
    view_proj: mat4x4<f32>,
};

@group(0) @binding(0) var<uniform> camera: CameraUniform;
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
    @location(4) local: vec2<f32>,
    @location(5) @interpolate(flat) half_size: vec2<f32>,
    @location(6) @interpolate(flat) radius: vec4<f32>,
    @location(7) @interpolate(flat) thickness: f32,
}

@vertex
fn vs_main(
    @builtin(vertex_index) vertex_index: u32,
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.color = instance.model_color;
    out.shape = instance.shape;
    out.uv = model.tex_coords;
    out.tex_uv = instance.uv_rect.xy + model.tex_coords * instance.uv_rect.zw;
    out.half_size = instance.size * 0.5;
    out.radius = instance.radius;
    out.thickness = select(0.0, instance.tri_point_1.x, instance.shape == 0u);
    // Thickness is stored inside tri_point_1.x to avoid increasing struct size

    var world: vec3<f32>;

    if (instance.shape == 2u) {
        var tri_points = array<vec2<f32>, 3>(
            instance.tri_point_1,
            instance.tri_point_2,
            instance.tri_point_3,
        );

        world = vec3<f32>(tri_points[vertex_index], instance.pos.z);
        out.local = vec2<f32>(0.0);
    } else {
        // scale, rotate about top-left, translate (same as T * R * S before)
        let p = model.position.xy * instance.size;
        let s = sin(instance.rotation);
        let c = cos(instance.rotation);
        let rotated = vec2<f32>(p.x * c - p.y * s, p.x * s + p.y * c);
        world = vec3<f32>(instance.pos.xy + rotated, instance.pos.z);
        out.local = p - instance.size * 0.5; // unrotated, centered, for the SDF
    }

    out.clip_position = camera.view_proj * vec4<f32>(world, 1.0);
    return out;
}

fn corner_radius(p: vec2<f32>, r: vec4<f32>) -> f32 {
    let top    = select(r.x, r.y, p.x > 0.0);
    let bottom = select(r.w, r.z, p.x > 0.0);
    return select(top, bottom, p.y > 0.0);
}

fn sd_round_box(p: vec2<f32>, b: vec2<f32>, r: vec4<f32>) -> f32 {
    let rad = corner_radius(p, r);
    let q = abs(p) - b + vec2<f32>(rad);
    return length(max(q, vec2<f32>(0.0))) + min(max(q.x, q.y), 0.0) - rad;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // derivatives must be computed before any branch or discard
    let d = sd_round_box(in.local, in.half_size, in.radius);
    let aa = max(fwidth(d), 1e-4);

    var rect_alpha = clamp(0.5 - d / aa, 0.0, 1.0);                   // outer shape
    if in.thickness > 0.0 {
        rect_alpha -= clamp(0.5 - (d + in.thickness) / aa, 0.0, 1.0); // minus inner shape
    }
    let is_sdf = any(in.radius > vec4<f32>(0.0)) || in.thickness > 0.0;
    rect_alpha = select(1.0, rect_alpha, is_sdf);

    let tex = textureSample(t_diffuse, s_diffuse, in.tex_uv);
    let color = in.color * tex;

    if color.a <= 0.0 { discard; }

    if in.shape == 1u {
        let alpha = 1.0 - smoothstep(0.491, 0.5, length(in.uv - vec2<f32>(0.5)));
        if alpha <= 0.0 { discard; }
        return vec4<f32>(color.rgb, color.a * alpha);
    }

    if in.shape == 0u {
        if rect_alpha <= 0.0 { discard; }
        return vec4<f32>(color.rgb, color.a * rect_alpha);
    }

    return color;
}