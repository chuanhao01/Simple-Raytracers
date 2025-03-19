struct Ray {
    origin: vec3f,
    direction: vec3f
}
fn no_ray() -> Ray{
    return Ray(vec3f(0f), vec3f(0f));
}
fn at(ray: Ray, t: f32) -> vec3f {
    return ray.origin + t * ray.direction;
}

struct LightRay{
    ray: Ray,
    attenuation: vec3f,
    done: bool,
}

fn color_ray(ray: Ray) -> LightRay {
    var t_max_so_far = FLT_MAX;
    var closest_hit = no_hit_record();
    for (var i = 0u; i < arrayLength(&spheres); i += 1u) {
        let sphere = spheres[i];
        let hit = intersect_sphere(sphere, ray, t_max_so_far);
        if within(T_MIN, hit.t, t_max_so_far) && hit.hit {
            closest_hit = hit;
            t_max_so_far = closest_hit.t;
        }
    }

    if closest_hit.hit{
        if closest_hit.material.t == 0u {
            let scatter_material = scatter_materials[closest_hit.material.sactter_idx];
            if scatter_material.t == 0u{
                return scatter_lambertain(closest_hit, scatter_material);
            } else if scatter_material.t == 1u {
                return scatter_metal(ray, closest_hit, scatter_material);
            } else if scatter_material.t == 2u {
                return scatter_dielectric(ray, closest_hit, scatter_material);
            }
        }
    }
    // return LightRay(no_ray(), vec3f(0f), true);
    return sky_color(ray);
}


fn sky_color(ray: Ray) -> LightRay {
    let a = 0.5 * (normalize(ray.direction).y + 1.0);
    return LightRay(Ray(vec3f(0f), vec3f(0f)), (1.0 - a) * vec3f(1.0) + a * vec3f(0.5, 0.7, 1.0), true);
}
@fragment
fn display_fs(@builtin(position) pos: vec4f) -> @location(0) vec4f {
    init_rng(vec2u(pos.xy));
    let camera_origin = camera_uniforms.look_from;
    let aspect_raio = f32(uniforms.vp_width) / f32(uniforms.vp_height);

    // let offset = vec2f(0f, 0f);
    let offset = vec2f(
        rand_f32() - 0.5,
        rand_f32() - 0.5
    );

    let _w = camera_uniforms.look_at - camera_uniforms.look_from;
    // let _w = uniforms.look_from - uniforms.look_at;
    let focal_distance = length(_w);

    let w = normalize(_w);
    let u = normalize(cross(camera_uniforms.v_up, w));
    let v = cross(w, u);


    let height = 2f * focal_distance * tan(radians(camera_uniforms.theta / 2f));
    let width = aspect_raio * height;
    let delta_x = vec3f(width / f32(uniforms.vp_width)) * u;
    let delta_y = vec3f(-height / f32(uniforms.vp_height)) * v;
    let top_left = camera_uniforms.look_at + vec3f(-width / 2f) * u + vec3f(height / 2f) * v + vec3f(0.5) * delta_x + vec3f(0.5) * delta_y;
    // Add a offset from unit_square to the x, y * delta
    var uv = top_left + vec3f((f32(pos.x) + offset.x)) * delta_x + vec3f((f32(pos.y) + offset.y)) * delta_y;

    let camera_pixel_direction = uv - camera_origin;
    let camera_pixel_ray = Ray(camera_origin, camera_pixel_direction);

    var throughput = vec3f(1f);
    var ray = camera_pixel_ray;
    var current_depth = 0u;
    while current_depth < DEPTH{
        let radiance_sample = color_ray(ray);
        throughput *= radiance_sample.attenuation;
        ray = radiance_sample.ray;
        current_depth += 1u;
        if radiance_sample.done{
            break;
        }
    }

    var old_sum: vec3f;
    if uniforms.frame_count > 1u {
        old_sum = textureLoad(radiance_samples_old, vec2u(pos.xy), 0).xyz;
    } else {
        old_sum = vec3f(0f);
    }

    let new_sum = old_sum + throughput;
    textureStore(radiance_samples_new, vec2u(pos.xy), vec4f(new_sum, 0f));

    // return vec4f(throughput, 1f);

    // Adding gamma correction
    let color = new_sum / f32(uniforms.frame_count);
    return vec4f(pow(color, vec3(1f/ 2.2)), 1f);
    // return vec4f(sqrt(new_sum / f32(uniforms.frame_count)), 1f);
    // return vec4f(spheres[1].center, 1f);
}
