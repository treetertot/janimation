use std::ops::{Add, Mul};

#[derive(Debug, Clone)]
pub struct Line<T> {
    length: f32,
    slope: f32,
    start: T,
    end: T,
}
impl<T: Clone + Add<T, Output = T> + Mul<f32, Output = T>> Line<T> {
    pub fn new(length: f32, start: T, end: T) -> Self {
        Line {
            slope: 1. / length,
            length,
            start,
            end,
        }
    }
    pub fn sample(&self, x: f32) -> Result<T, f32> {
        if self.length < x {
            return Err(x - self.length);
        }
        let y = self.slope * x;
        Ok(self.start.clone() * (1. - y) + self.end.clone() * y)
    }
}
impl<T> Line<T> {
    pub fn len(&self) -> f32 {
        self.length
    }
}
