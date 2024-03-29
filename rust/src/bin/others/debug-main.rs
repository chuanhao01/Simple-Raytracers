use std::sync::Arc;

use rust_simple_raytracer::{
    materials::Dielectric, Hittables, HittablesList, Lambertain, Metal, ScatterMaterials, Sphere,
    Vec3, BVH,
};

#[allow(unused_variables)]
fn test_scene() {
    let material_ground = Arc::new(ScatterMaterials::Lambertain(Lambertain {
        albedo: Vec3::new(0.8_f64, 0.8_f64, 0_f64),
    }));

    let material_red = Arc::new(ScatterMaterials::Lambertain(Lambertain {
        albedo: Vec3::new(0.8_f64, 0.0_f64, 0.0_f64),
    }));
    let material_green = Arc::new(ScatterMaterials::Lambertain(Lambertain {
        albedo: Vec3::new(0.0_f64, 0.8_f64, 0.0_f64),
    }));
    let material_blue = Arc::new(ScatterMaterials::Lambertain(Lambertain {
        albedo: Vec3::new(0.0_f64, 0.0_f64, 0.8_f64),
    }));

    let material_metal = Arc::new(ScatterMaterials::Metal(Metal::new(
        Vec3::new(0.1_f64, 0.2_f64, 0.5_f64),
        0.0_f64,
    )));
    let material_metal_fuzzy = Arc::new(ScatterMaterials::Metal(Metal::new(
        Vec3::new(0.1_f64, 0.2_f64, 0.5_f64),
        0.3_f64,
    )));
    let material_glass = Arc::new(ScatterMaterials::Dielectric(Dielectric {
        index_of_reflectance: 1.4,
    }));

    let mut hittable_list = HittablesList::new();
    hittable_list.add(Hittables::Sphere(Sphere::new(
        Vec3::new(-1.0, 1.0, -0.7),
        0.5,
        Arc::clone(&material_red),
    )));
    hittable_list.add(Hittables::Sphere(Sphere::new(
        Vec3::new(0.0, 1.0, -0.7),
        0.5,
        Arc::clone(&material_green),
    )));
    hittable_list.add(Hittables::Sphere(Sphere::new(
        Vec3::new(1.0, 1.0, -0.7),
        0.5,
        Arc::clone(&material_blue),
    )));
    let world = BVH::from_hittable_list(&hittable_list);

    eprintln!("{}", world);
}

fn main() {
    test_scene();
}
