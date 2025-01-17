use core::ops::{Add, Mul, Sub};

use super::VecMethods;

#[derive(Copy, Clone, Debug)]
pub struct Vec3<T> {
    pub x: T,
    pub y: T,
    pub z: T,
}

impl<T> Vec3<T> {
    pub const fn new(x: T, y: T, z: T) -> Self {
        Self { x, y, z }
    }
}

impl<T> VecMethods<T> for Vec3<T>
where
    T: Copy + Add<Output = T> + Mul<Output = T> + Sub<Output = T>,
{
    fn sum(&self) -> T {
        self.x + self.y + self.z
    }

    fn get_attenuation_factor(&self) -> T {
        (self.x * self.x) + (self.y * self.y) + (self.z * self.z)
    }
}

impl<T: Copy> Vec3<T> {
    pub fn map<Y>(&self, f: impl Fn(T) -> Y) -> Vec3<Y> {
        Vec3 {
            x: f(self.x),
            y: f(self.y),
            z: f(self.z),
        }
    }
}

impl<T> Sub<Vec3<T>> for Vec3<T>
where
    T: Sub<Output = T>,
{
    type Output = Vec3<T>;

    fn sub(self, rhs: Self) -> Self::Output {
        Vec3 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl<T: Add<Output = T>> Add<Vec3<T>> for Vec3<T> {
    type Output = Vec3<T>;

    fn add(self, rhs: Self) -> Self::Output {
        Vec3 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl<T> Mul<T> for Vec3<T>
where
    T: Copy + Mul<Output = T>,
{
    type Output = Vec3<T>;

    fn mul(self, rhs: T) -> Self::Output {
        Vec3 {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}
