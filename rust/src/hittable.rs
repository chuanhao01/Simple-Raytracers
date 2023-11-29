use std::{cmp::Ordering, fmt::Display, sync::Arc};

use rand::{thread_rng, Rng};

use crate::{objects::AABB, ray::Ray, Hittables, Interval, Materials, Vec3};

pub struct HitRecord {
    /// Point at which the hit occurs
    pub p: Vec3,
    /// t value used to generate the hit Point from the ray
    pub t: f64,
    /// Material of the Hittable Object hit
    pub material: Arc<Materials>,
    /// Unit Vector of the normal generated by the hit ray, going against the direction of the hit ray
    pub against_normal_unit: Vec3,
    /// If the ray hits from the outside
    pub front_face: bool,
}
impl HitRecord {
    /// Used to create the HitRecord object
    /// Takes in the ray that Hits the object as well as the
    pub fn new(_ray: &Ray, outward_normal_unit: &Vec3, t: f64, material: Arc<Materials>) -> Self {
        let p = _ray.at(t);
        let front_face = Vec3::dot(&_ray.direction, outward_normal_unit) < 0_f64;
        let against_normal_unit = if front_face {
            outward_normal_unit.clone()
        } else {
            -outward_normal_unit.clone()
        };
        HitRecord {
            p,
            t,
            material,
            against_normal_unit,
            front_face,
        }
    }
}

pub trait Hittable<T> {
    fn hit(&self, _ray: &Ray, valid_t_interval: Interval) -> Option<T>;
}

/// A Hittable List to hold all the Hittables in the world
/// Do create the object with a default and use the [HittableList::add] method to add Hittables to the list
#[derive(Default)]
pub struct HittablesList {
    pub v: Vec<Arc<Hittables>>,
    bbox: AABB,
}
impl HittablesList {
    /// Create an empty HittableList
    pub fn new() -> Self {
        Self {
            v: Vec::new(),
            bbox: AABB::default(),
        }
    }

    /// Adds a new Hittable into the list
    pub fn add(&mut self, hittable: Arc<Hittables>) {
        // Dont think this is needed anywhere?
        self.bbox = AABB::from_aabb(&self.bbox, hittable.bbox());
        self.v.push(hittable);
    }
    /// Appends the hittables to the current list
    pub fn append(&mut self, hittables: &mut Vec<Arc<Hittables>>) {
        self.bbox = hittables
            .iter()
            .fold(self.bbox, |acc: AABB, hittable: &Arc<Hittables>| {
                AABB::from_aabb(&acc, hittable.bbox())
            });

        self.v.append(hittables);
    }
    /// Method to get the length of Hittables
    pub fn len(&self) -> usize {
        self.v.len()
    }

    pub fn is_empty(&self) -> bool {
        self.v.is_empty()
    }
}

/// Bounding volume hierarchy
pub struct BVH {
    left: Option<Box<BVH>>,
    right: Option<Box<BVH>>,
    bbox: AABB,
    hittable: Option<Arc<Hittables>>,
}
impl BVH {
    pub fn from_hittable_list(hittable_list: &HittablesList) -> Self {
        Self::new(&hittable_list.v[..], 0, hittable_list.len())
    }
    #[allow(clippy::clone_on_copy)]
    fn new(hittables: &[Arc<Hittables>], start: usize, end: usize) -> Self {
        let mut rng = thread_rng();
        let axis = rng.gen_range(0_i64..3_i64);

        let hittable_comparer = |a: &Arc<Hittables>, b: &Arc<Hittables>| -> Ordering {
            if a.bbox().axis(axis).min < b.bbox().axis(axis).min {
                Ordering::Less
            } else if a.bbox().axis(axis).min > b.bbox().axis(axis).min {
                Ordering::Greater
            } else {
                Ordering::Equal
            }
        };
        let list_size = end - start;
        if list_size == 0 {
            BVH {
                left: None,
                right: None,
                hittable: Some(Arc::new(Hittables::None)),
                bbox: Hittables::None.bbox().clone(),
            }
        } else if list_size == 1 {
            BVH {
                left: None,
                right: None,
                hittable: Some(hittables[start].clone()),
                bbox: hittables[start].bbox().clone(),
            }
        } else if list_size == 2 {
            BVH {
                left: Some(Box::new(BVH {
                    left: None,
                    right: None,
                    hittable: Some(hittables[start].clone()),
                    bbox: hittables[start].bbox().clone(),
                })),
                right: Some(Box::new(BVH {
                    left: None,
                    right: None,
                    hittable: Some(hittables[end - 1].clone()),
                    bbox: hittables[end - 1].bbox().clone(),
                })),
                hittable: None,
                bbox: AABB::from_aabb(hittables[start].bbox(), hittables[end - 1].bbox()),
            }
        } else {
            let mut hittables = hittables.to_vec();
            // Small bug, I used to sort the whole vector
            // hittables.sort_by(hittable_comparer);
            // Since the vector is sorted everytime by a random axis
            // Example with 4
            // The left side places 0, 1 from 0, 1, 2, 3
            // Then the right side sorts 2, 3, 0, 1 With the chosen
            // hittables.sort_by(hittable_comparer);
            hittables[start..end].sort_by(hittable_comparer);
            let mid = start + list_size / 2;
            let left = Self::new(&hittables, start, mid);
            let right = Self::new(&hittables, mid, end);
            let bbox = AABB::from_aabb(&left.bbox, &right.bbox);

            BVH {
                left: Some(Box::new(left)),
                right: Some(Box::new(right)),
                hittable: None,
                bbox,
            }
        }
    }
    /// Does a pre order traversal of the BVH, taking the to_string of each per line
    fn pre_order_debug(root: &Self) -> String {
        let mut s = String::new();
        let mut ss = root.bbox.to_string();
        if let Some(hittable) = &root.hittable {
            ss.push(' ');
            ss.push_str(&hittable.to_string());
        }
        s.push_str(&ss);
        s.push('\n');
        if let Some(left) = &root.left {
            s.push_str(&Self::pre_order_debug(left))
        }
        if let Some(right) = &root.right {
            s.push_str(&Self::pre_order_debug(right))
        }
        s
    }
}
impl Hittable<HitRecord> for BVH {
    fn hit(&self, _ray: &Ray, valid_t_interval: Interval) -> Option<HitRecord> {
        // Deal with base case of edge leaf nodes (Just the hittables)
        // Exit early to prevent computation
        if let (None, None) = (&self.left, &self.right) {
            match &self.hittable {
                Some(hittable) => return hittable.hit(_ray, valid_t_interval),
                None => panic!("Hittable should exist for outer BVH node"),
            };
        }

        // Check if we don't hit the bbox of the BVH (Nicer rust code)
        self.bbox.hit(_ray, valid_t_interval)?;

        match (&self.left, &self.right) {
            // BVH nodes which are not edges
            (Some(left), Some(right)) => {
                let mut valid_t_interval = valid_t_interval;
                if let Some(left_hit) = left.hit(_ray, valid_t_interval) {
                    valid_t_interval.max = left_hit.t;
                    if let Some(right_hit) = right.hit(_ray, valid_t_interval) {
                        Some(right_hit)
                    } else {
                        Some(left_hit)
                    }
                } else {
                    right.hit(_ray, valid_t_interval)
                }
            }
            // If there is a single Node or we reach (None, None) again
            // Should never reach here
            _ => {
                panic!("Trying to match left or right, but either no left or right or both")
            }
        }
    }
}
impl Display for BVH {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = BVH::pre_order_debug(self);
        write!(f, "{}", s)
    }
}

impl Hittable<HitRecord> for Vec<Hittables> {
    fn hit(&self, _ray: &Ray, valid_t_interval: Interval) -> Option<HitRecord> {
        // For each hittable in the vec, iter through the hittables and run hit, accumulate the hits and return the nearest hit
        let (_, result) = self
            .iter()
            .fold((valid_t_interval.max, None), |acc, hittable| {
                if let Some(hit_record) = hittable.hit(
                    _ray,
                    Interval {
                        min: valid_t_interval.min,
                        max: acc.0,
                    },
                ) {
                    (hit_record.t, Some(hit_record))
                } else {
                    acc
                }
            });
        result
    }
}
impl Hittable<HitRecord> for Vec<Arc<Hittables>> {
    fn hit(&self, _ray: &Ray, valid_t_interval: Interval) -> Option<HitRecord> {
        // For each hittable in the vec, iter through the hittables and run hit, accumulate the hits and return the nearest hit
        let (_, result) = self
            .iter()
            .fold((valid_t_interval.max, None), |acc, hittable| {
                if let Some(hit_record) = hittable.hit(
                    _ray,
                    Interval {
                        min: valid_t_interval.min,
                        max: acc.0,
                    },
                ) {
                    (hit_record.t, Some(hit_record))
                } else {
                    acc
                }
            });
        result
    }
}

#[cfg(test)]
mod test {
    use std::f64::INFINITY;

    use crate::{Lambertain, Metal, ScatterMaterials, Sphere};

    use super::*;

    #[test]
    fn test_bvh_from_hittable_list() {
        let mut hittable_list = HittablesList::new();
        hittable_list.add(Arc::new(Hittables::Sphere(Sphere::new(
            Vec3::new_int(0, 0, 0),
            1.0,
            Arc::new(Materials::ScatterMaterial(ScatterMaterials::None)),
        ))));
        hittable_list.add(Arc::new(Hittables::None));
        hittable_list.add(Arc::new(Hittables::None));
        hittable_list.add(Arc::new(Hittables::None));

        let bvh = BVH::from_hittable_list(&hittable_list);
        // Bad Rust code, but oh well its for a test
        if let Hittables::Sphere(ref sphere) = *bvh.left.unwrap().left.unwrap().hittable.unwrap() {
            assert_eq!(sphere.radius, 1.0);
            assert!(matches!(
                *sphere.material,
                Materials::ScatterMaterial(ScatterMaterials::None)
            ))
        }
    }
    #[test]
    fn test_bvh_hit() {
        let mut hittable_list = HittablesList::new();
        hittable_list.add(Arc::new(Hittables::Sphere(Sphere::new(
            Vec3::new_int(0, 0, -1),
            0.5,
            Arc::new(Materials::ScatterMaterial(ScatterMaterials::Metal(
                Metal::new(Vec3::new(0.0, 0.0, 1.0), 0.1),
            ))),
        ))));
        hittable_list.add(Arc::new(Hittables::Sphere(Sphere::new(
            Vec3::new_int(0, 0, -3),
            1.0,
            Arc::new(Materials::ScatterMaterial(ScatterMaterials::Lambertain(
                Lambertain {
                    albedo: Vec3::new(0.0, 1.0, 0.0),
                },
            ))),
        ))));
        hittable_list.add(Arc::new(Hittables::Sphere(Sphere::new(
            Vec3::new_int(0, 0, -5),
            1.0,
            Arc::new(Materials::ScatterMaterial(ScatterMaterials::Lambertain(
                Lambertain {
                    albedo: Vec3::new(0.0, 0.0, 1.0),
                },
            ))),
        ))));
        let bvh = BVH::from_hittable_list(&hittable_list);

        let hit = bvh
            .hit(
                &Ray {
                    origin: Vec3::new_int(0, 0, 0),
                    direction: Vec3::new(0.0, 0.0, -1.0),
                },
                Interval {
                    min: 0.001,
                    max: INFINITY,
                },
            )
            .unwrap();
        assert_eq!(hit.t, 0.5);
    }
}
