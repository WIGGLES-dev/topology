use crate::coord::{Coordinate, Orientation};

#[derive(Default)]
pub struct ShoeString {
    area_sum: f32,
}

impl ShoeString {
    pub fn add(&mut self, v1: &impl Coordinate, v2: &impl Coordinate) {
        let [x, y] = v1.xy();
        let [x1, y1] = v2.xy();
        self.area_sum += x * y1 - y * x1;
    }

    pub fn area(&self) -> f32 {
        self.area_sum / 2.
    }

    pub fn orientation(&self) -> Orientation {
        if self.area_sum > 0. {
            Orientation::Counterclockwise(0.)
        } else if self.area_sum < 0. {
            Orientation::Clockwise(0.)
        } else {
            Orientation::Neutral
        }
    }
}
