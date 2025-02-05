struct VertexInput {
    @location(0) position: vec3<f32>,
}

struct BoidInstance {
    @location(1) pos: vec2<f32>,
    @location(2) rot: f32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
}

@group(0) @binding(0)
var<uniform> camera_mat: mat3x3<f32>;

@vertex
fn vs_main(
    vertex: VertexInput,
    instance: BoidInstance,
) -> VertexOutput {
    var out: VertexOutput;


    let rot = instance.rot;
    let rot_sin = sin(rot);
    let rot_cos = cos(rot);
    let instance_mat = mat3x3<f32>(
        vec3<f32>( rot_cos, rot_sin, 0),
        vec3<f32>(-rot_sin, rot_cos, 0),
        vec3<f32>( instance.pos    , 1),
    );


    var clip_position = camera_mat * instance_mat * vertex.position;
    out.clip_position = vec4<f32>(clip_position, 1.0);

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(0.11658, 0.05112, 0.38891, 1.0);
}

