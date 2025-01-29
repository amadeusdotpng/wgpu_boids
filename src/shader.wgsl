const PI = 3.14159;

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


    let rot_sin = sin(instance.rot);
    let rot_cos = cos(instance.rot);
    let instance_mat = mat3x3<f32>(
        vec3<f32>( rot_cos, rot_sin, 0),
        vec3<f32>(-rot_sin, rot_cos, 0),
        vec3<f32>( instance.pos    , 1),
    );


    let clip_position = camera_mat * instance_mat * vec3<f32>(vertex.position);
    out.clip_position = vec4<f32>(clip_position, 1.0);

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(0.11658, 0.05112, 0.38891, 1.0);
}

struct Boid {
    pos: vec2<f32>,
    rot: f32,
}

@group(0) @binding(0) var<storage, read> boids_src: array<Boid>;
@group(0) @binding(1) var<storage, read_write> boids_dst: array<Boid>;

@compute
@workgroup_size(64)
fn cs_main(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
    let total = arrayLength(&boids_src);
    let idx = global_invocation_id.x;
    if(idx >= total) {
        return;
    }
    let instance = boids_src[idx];

    let rot_cos = cos(instance.rot + PI/2);
    let rot_sin = sin(instance.rot + PI/2);
    let pos = vec2<f32>(
        instance.pos.x + 0.05 * rot_cos,
        instance.pos.y + 0.05 * rot_sin,
    );

    boids_dst[idx] = Boid(pos, instance.rot);
}
