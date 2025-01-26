struct VertexInput {
    @location(0) position: vec3<f32>,
}

struct InstanceInput {
    @location(1) model_matrix_0: vec3<f32>,
    @location(2) model_matrix_1: vec3<f32>,
    @location(3) model_matrix_2: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
}

@group(0) @binding(0)
var<uniform> camera_mat: mat3x3<f32>;

@vertex
fn vs_main(
    vertex: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    var out: VertexOutput;

    let instance_mat = mat3x3<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
    );
    let clip_position = camera_mat * instance_mat * vertex.position;
    out.clip_position = vec4<f32>(clip_position, 1.0);

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(0.11658, 0.05112, 0.38891, 1.0);
}
