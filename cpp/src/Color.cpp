#include "Color.h"
#include "Interval.h"

#include <cmath>
#include <iostream>

namespace color {

double linear_to_gamma(double linear_component) {
  return std::sqrt(linear_component);
}

/**
 * Take a color pixel and prints out the rgb values to cout
 */
void write_color(std::ostream &os, const Color &pixel_color,
                 int samples_per_pixel) {

  auto r = pixel_color.x();
  auto g = pixel_color.y();
  auto b = pixel_color.z();

  auto scale = 1.0 / samples_per_pixel;
  r *= scale;
  g *= scale;
  b *= scale;

  r = linear_to_gamma(r);
  g = linear_to_gamma(g);
  b = linear_to_gamma(b);

  static const interval::Interval intensity(0.0, 0.999);

  os << static_cast<int>(intensity.clamp(r) * 256) << " "
     << static_cast<int>(intensity.clamp(g) * 256) << " "
     << static_cast<int>(intensity.clamp(b) * 256);
}
} // namespace color
