const PI = 3.14159;

struct VertexInput {
    @location(0) position: vec3<f32>,
}

struct BoidInstance {
    @location(1) pos: vec2<f32>,
    @location(2) vel: vec2<f32>,
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

    let rot = atan2(instance.vel.y, instance.vel.x);

    let rot_sin = sin(rot);
    let rot_cos = cos(rot);
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
    vel: vec2<f32>,
}

@group(0) @binding(0) var<storage, read> boids_src: array<Boid>;
@group(0) @binding(1) var<storage, read_write> boids_dst: array<Boid>;

fn smoothing_kernel(r: f32, dst: f32) -> f32 {
    let v = max(0.0, r - dst);
    return (v * v * v) / (r * r * r);
}
@compute
@workgroup_size(64)
fn cs_main(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
    let total = arrayLength(&boids_src);
    let idx = global_invocation_id.x;
    if(idx >= total) { return; }

    let flock_radius = 4.0;
    let avoid_radius = 3.0;
    let wall_radius = 512.0;


    let separation_weight = 0.55;
    let alignment_weight  = 0.15;
    let cohesion_weight   = 0.05;

    var separation_force = vec2<f32>(0, 0);
    var alignment_force  = vec2<f32>(0, 0);
    var center_flock     = vec2<f32>(0, 0);


    let wall_weight = 3.0;
    var wall_force  = vec2<f32>(0, 0);

    var n_flock = 0;
    let instance = boids_src[idx];

    let dst_from_wall = wall_radius - length(instance.pos);
    wall_force = (-instance.pos) * smoothing_kernel(2.0, dst_from_wall);
    

    for(var i = u32(0); i < total; i++) {
        let other = boids_src[i];

        let d_pos = other.pos - instance.pos;
        let dt = dot(d_pos, d_pos);
        if(dt < flock_radius * flock_radius) {
            n_flock += 1;
            if(dt > 0 && dt < avoid_radius * avoid_radius) { separation_force -= d_pos / (dt + 1); }

            let d_vel = other.vel - instance.vel;
            let dt_vel = length(d_vel);
            if(dt_vel > 0) { alignment_force += d_vel; }

            center_flock += other.pos;
        }

    }
    
    let new_pos = instance.pos + instance.vel * 0.20;
    var new_vel =  instance.vel;
    if(n_flock > 0) {
        alignment_force /= f32(n_flock);
        let cohesion_force = (center_flock / f32(n_flock)) - instance.pos;
        // let cohesion_force = vec2<f32>(0, 0);

        let acceleration = separation_force * separation_weight
                         + alignment_force  * alignment_weight
                         + cohesion_force   * cohesion_weight
                         + wall_force       * wall_weight;

        new_vel += acceleration;
        new_vel /= length(new_vel);
    }
    boids_dst[idx] = Boid(new_pos, new_vel);
}

