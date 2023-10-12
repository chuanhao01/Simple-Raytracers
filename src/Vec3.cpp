#include "Vec3.h"

#include <cmath>
#include <iostream>

namespace vec {
Vec3 &Vec3::operator+=(const Vec3 &v) {
  e[0] += v[0];
  e[1] += v[1];
  e[2] += v[2];
  return *this;
}
Vec3 &Vec3::operator-=(const Vec3 &v) {
  e[0] -= v[0];
  e[1] -= v[1];
  e[2] -= v[2];
  return *this;
}
Vec3 &Vec3::operator*=(double t) {
  e[0] *= t;
  e[1] *= t;
  e[2] *= t;
  return *this;
}
Vec3 &Vec3::operator/=(double t) {
  *this *= 1 / t;
  return *this;
}

double Vec3::length() const { return std::sqrt(length_squared()); }
double Vec3::length_squared() const {
  return e[0] * e[0] + e[1] * e[1] + e[2] * e[2];
}

std::ostream &operator<<(std::ostream &cout, const Vec3 &v) {
  return cout << "Vec3(" << v[0] << ", " << v[1] << ", " << v[2] << ")";
}

Vec3 operator+(const Vec3 &v1, const Vec3 &v2) {
  Vec3 v = Vec3(v1);
  v += v2;
  return v;
}
Vec3 operator-(const Vec3 &v1, const Vec3 &v2) {
  Vec3 v = Vec3(v1);
  v -= v2;
  return v;
}
Vec3 operator*(const Vec3 &v, double t) {
  Vec3 nv = Vec3(v);
  nv *= t;
  return nv;
}
Vec3 operator*(double t, const Vec3 &v) {
  Vec3 nv = Vec3(v);
  nv *= t;
  return nv;
}
Vec3 operator/(Vec3 &v, double t) {
  Vec3 nv = Vec3(v);
  nv /= t;
  return nv;
}

// Since we are only dealing with 3D vecs, this is [3x1] * [1x3] always
double dot(const Vec3 &v1, const Vec3 &v2) {
  return v1[0] * v2[0] + v1[1] * v2[1] + v1[2] * v2[2];
}
Vec3 cross(const Vec3 &v1, const Vec3 &v2) {
  return Vec3(v1[1] * v2[2] - v1[2] * v2[1], v1[2] * v2[0] - v1[0] * v2[2],
              v1[0] * v2[1] - v1[1] * v2[0]);
}

Vec3 unit_vector(const Vec3 &v) {
  Vec3 nv = Vec3(v);
  return nv / nv.length();
}
} // namespace vec
