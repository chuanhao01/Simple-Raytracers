#ifndef CAMERA_H
#define CAMERA_H

#include "Color.h"
#include "Hittable_List.h"
#include "Ray.h"
#include "Vec3.h"

namespace camera {
/**
 * @brief
 * @param image_width Rendered image width in pixel count
 * @param samples_per_pixel Count of random rendered rays per pixel
 * @param max_depth Number of simulated bounces per ray
 * @param fov Degree of FOV
 * @param look_from Center point of camera
 * @param look_at Point where camera points at (Where the image plane is at)
 * @param v_up Vector to specify the v plane in relation to w (direction of
 * camera) Tilting the plane, makes the camera view tilt
 */
class Camera {
public:
  // Variables user can set
  double aspect_ratio = 16.0 / 9.0;
  int image_width = 400;

  int samples_per_pixel = 10;
  int max_depth = 10;

  double fov = 90;

  vec::Point3 look_from = vec::Point3(0, 0, 0);
  vec::Point3 look_at = vec::Point3(0, 0, -1);
  vec::Vec3 v_up = vec::Vec3(0, 1, 0);

  double focus_angle = 0;
  double focus_distance = 10;

  void render(const hittable_list::Hittable_List &world);

private:
  int image_height;
  double viewport_height;
  double viewport_width;

  vec::Vec3 u, v, w;
  vec::Vec3 center;

  vec::Vec3 pixel_delta_u;
  vec::Vec3 pixel_delta_v;

  vec::Vec3 defocus_disk_u;
  vec::Vec3 defocus_disk_v;

  vec::Point3 pixel00_loc;

  void init();
  ray::Ray get_ray(double j, double i);

  vec::Point3 pixel_sample_square();
  vec::Point3 defocus_disk_sample();

  color::Color color_ray(const ray::Ray &r, int max_depth,
                         const hittable_list::Hittable_List &world);
};
} // namespace camera

#endif
