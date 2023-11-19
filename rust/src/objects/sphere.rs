use std::{fmt::Display, sync::Arc};

use crate::{HitRecord, Hittable, Interval, Materials, Ray, Vec3, AABB};

use super::HittableObject;

/// Simple Sphere object
/// To initialize use [Sphere::new]
pub struct Sphere {
    pub center: Vec3,
    pub radius: f64,
    pub material: Arc<Materials>,
    bbox: AABB,
}
impl Sphere {
    pub fn new(center: Vec3, radius: f64, material: Arc<Materials>) -> Self {
        let radius_v = Vec3::new(radius, radius, radius);
        let bbox = AABB::from_points(
            &(center.clone() - radius_v.clone()),
            &(center.clone() + radius_v.clone()),
        );
        Self {
            center,
            radius,
            material,
            bbox,
        }
    }
}
impl HittableObject for Sphere {
    fn bbox(&self) -> &AABB {
        &self.bbox
    }
}
impl Hittable<HitRecord> for Sphere {
    fn hit(&self, _ray: &Ray, valid_t_interval: Interval) -> Option<HitRecord> {
        let a_minus_c = _ray.origin.clone() - self.center.clone();

        let a = _ray.direction.length_squared();
        let b = Vec3::dot(&a_minus_c, &_ray.direction);
        let c = a_minus_c.length_squared() - self.radius * self.radius;

        // Ray does not hit the sphere
        let discriminant = b * b - a * c;
        if discriminant < 0_f64 {
            return None;
        }

        // Find the closer root, since the ray is from the camera, smaller t is closer to the camera
        let sqrt_discriminant = discriminant.sqrt();
        let root = (-b - sqrt_discriminant) / a;
        let root = if !valid_t_interval.surrounds(root) {
            (-b + sqrt_discriminant) / a
        } else {
            root
        };
        if !valid_t_interval.surrounds(root) {
            return None;
        }
        // let outward_normal_unit = (_ray.at(root) - self.center.clone()).unit_vector();
        let outward_normal_unit = (_ray.at(root) - self.center.clone()) / self.radius;
        Some(HitRecord::new(
            _ray,
            &outward_normal_unit,
            root,
            Arc::clone(&self.material),
        ))
    }
}
impl Display for Sphere {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Sphere(center: {}, radius: {})",
            self.center, self.radius
        )
    }
}

#[cfg(test)]
mod test {
    use crate::ScatterMaterials;

    use super::*;
    use std::f64::INFINITY;

    #[test]
    fn test_sphere_new() {
        let mat = Arc::new(Materials::ScatterMaterial(ScatterMaterials::None));
        let s = Sphere::new(Vec3::new_int(0, 0, 0), 1.0, Arc::clone(&mat));
        assert_eq!(s.bbox.x.min, -1.0);
        assert_eq!(s.bbox.y.min, -1.0);
        assert_eq!(s.bbox.z.min, -1.0);

        assert_eq!(s.bbox.x.max, 1.0);
        assert_eq!(s.bbox.y.max, 1.0);
        assert_eq!(s.bbox.z.max, 1.0);
    }

    #[test]
    fn test_sphere_hit() {
        // Ensure the ray hits the sphere
        let mat = Arc::new(Materials::ScatterMaterial(ScatterMaterials::None));
        let s = Sphere::new(Vec3::new_int(0, 0, 0), 1.0, Arc::clone(&mat));
        let r = Ray {
            direction: Vec3::new_int(0, 0, 1),
            origin: Vec3::new_int(0, 0, -2),
        };
        let hr = s
            .hit(
                &r,
                Interval {
                    min: 0.001,
                    max: INFINITY,
                },
            )
            .unwrap();
        assert_eq!(hr.p, Vec3::new_int(0, 0, -1));
        assert_eq!(hr.t, 1.0);
        assert_eq!(hr.against_normal_unit, Vec3::new_int(0, 0, -1));
        assert!(Arc::ptr_eq(&mat, &hr.material));
        assert!(hr.front_face);

        // Ensure you get the second t value
        let hr = s
            .hit(
                &r,
                Interval {
                    min: 1.0,
                    max: INFINITY,
                },
            )
            .unwrap();
        assert_eq!(hr.t, 3.0);

        // Ensure interval out of range
        assert!(s.hit(&r, Interval { min: 4.0, max: 5.0 },).is_none());

        // Ensure that the ray does not hit the sphere
        let r = Ray {
            direction: Vec3::new_int(2, 0, 1),
            ..r
        };
        assert!(s
            .hit(
                &r,
                Interval {
                    min: 0.001,
                    max: INFINITY,
                },
            )
            .is_none());
    }
}