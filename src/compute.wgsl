const PI = 3.14159;

struct Boid {
    pos: vec2<f32>,
    rot: f32,
}

@group(0) @binding(0) var<storage, read> boids_src: array<Boid>;
@group(0) @binding(1) var<storage, read_write> boids_dst: array<Boid>;

@group(1) @binding(0)
var<uniform> camera_mat: mat3x3<f32>;

@group(1) @binding(1)
var<uniform> proj_matrix: mat3x3<f32>;

@group(2) @binding(0)
var<uniform> cursor: vec2<f32>;
@compute
@workgroup_size(64)
fn cs_main(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
    let total = arrayLength(&boids_src);
    let idx = global_invocation_id.x;
    if(idx >= total) {
        return;
    }
    let instance = boids_src[idx];

    let d = cursor - (proj_matrix * vec3<f32>(instance.pos, 1.0)).xy;
    let rot = atan2(d.y, d.x) + 0.49 * PI;

    let rot_cos = cos(rot);
    let rot_sin = sin(rot);
    let pos = vec2<f32>(
        instance.pos.x + 0.05 * rot_cos,
        instance.pos.y + 0.05 * rot_sin,
    );

    boids_dst[idx] = Boid(pos, rot);
}
