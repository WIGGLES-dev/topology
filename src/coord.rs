use std::cmp::Ordering;

pub type Precision = f32;

pub trait Coordinate {
    fn xyz(&self) -> [Precision; 3];
    fn xy(&self) -> [Precision; 2] {
        [self.x(), self.y()]
    }
    fn x(&self) -> Precision {
        self.xyz()[0]
    }
    fn y(&self) -> Precision {
        self.xyz()[1]
    }
    fn z(&self) -> Precision {
        self.xyz()[2]
    }
}

impl Coordinate for [Precision; 3] {
    fn xyz(&self) -> [Precision; 3] {
        *self
    }
}

impl Coordinate for (Precision, Precision, Precision) {
    fn xyz(&self) -> [Precision; 3] {
        [self.0, self.1, self.2]
    }
}

impl Coordinate for (Precision, Precision) {
    fn xyz(&self) -> [Precision; 3] {
        [self.0, self.1, 0.]
    }
}

impl Coordinate for [Precision; 2] {
    fn xyz(&self) -> [Precision; 3] {
        [self[0], self[1], 0.]
    }
}

pub trait FromCoordinate {
    fn from_xyz(xyz: [Precision; 3]) -> Self
    where
        Self: Sized;
    fn from_xy([x, y]: [Precision; 2]) -> Self
    where
        Self: Sized,
    {
        Self::from_xyz([x, y, 0.])
    }
}

pub trait UpdateCoordinate {
    fn set_xyz(&mut self, xyz: [Precision; 3]);
    fn set_xy(&mut self, [x, y]: [Precision; 2]) {
        Self::set_xyz(self, [x, y, 0.]);
    }
}

pub enum Winding {
    Clockwise,
    CounterClockwise,
}

impl Winding {
    pub fn flip(&self) -> Winding {
        match self {
            Self::Clockwise => Self::CounterClockwise,
            Self::CounterClockwise => Self::Clockwise,
        }
    }
}

#[derive(Debug)]
pub enum Orientation {
    Clockwise(Precision),
    Counterclockwise(Precision),
    Neutral,
}

fn orientation([x1, y1]: [Precision; 2], [x2, y2]: [Precision; 2]) -> Orientation {
    let [x, y] = [x2 - x1, y2 - y1];
    let theta = y.atan2(x);
    match theta {
        theta if theta > 0. => Orientation::Counterclockwise(theta),
        theta if theta < 0. => Orientation::Clockwise(theta),
        _ => Orientation::Neutral,
    }
}

pub fn polar_angle([x, y]: [Precision; 2], [cx, cy]: [Precision; 2]) -> Precision {
    (y - cy).atan2(x - cx)
}

impl Orientation {
    pub fn from_points(from: [Precision; 2], to: [Precision; 2]) -> Self {
        orientation(from, to)
    }

    pub fn is_cw(&self) -> bool {
        match self {
            Self::Clockwise(_) => true,
            _ => false,
        }
    }

    pub fn is_ccw(&self) -> bool {
        match self {
            Self::Counterclockwise(_) => true,
            _ => false,
        }
    }

    pub fn is_neutral(&self) -> bool {
        match self {
            Self::Neutral => true,
            _ => false,
        }
    }
}

pub fn sort_clockwise(
    [cx, cy]: [Precision; 2],
    [ax, ay]: [Precision; 2],
    [bx, by]: [Precision; 2],
) -> Ordering {
    if ax - cx >= 0. && bx - cx < 0. {
        return Ordering::Less;
    }

    if ax - cx < 0. && bx - cx >= 0. {
        return Ordering::Greater;
    }

    if ax - cx == 0. && bx - cx == 0. {
        if ay - cy >= 0. || by - cy >= 0. {
            return if ay > by {
                Ordering::Less
            } else {
                Ordering::Greater
            };
        }
        return if by > ay {
            Ordering::Less
        } else {
            Ordering::Greater
        };
    }

    let det = (ax - cx) * (by - cy) - (bx - cx) * (ay - cy);
    if det < 0. {
        return Ordering::Less;
    }

    if det > 0. {
        return Ordering::Greater;
    }

    let d1 = (ax - cx) * (ax - cx) + (ay - cy) * (ay - cy);
    let d2 = (bx - cx) * (bx - cx) + (by - cy) * (by - cy);

    if d1 > d2 {
        Ordering::Less
    } else {
        Ordering::Greater
    }
}

#[test]
fn test_orientation() {
    assert!(
        orientation((0., 0.).xy(), (1., 1.).xy()).is_ccw(),
        "(0,0) (1,1)"
    );

    assert!(
        orientation((0., 0.).xy(), (-1., -1.).xy()).is_cw(),
        "(0,0) (-1,-1)"
    );

    assert!(
        orientation((1., 1.).xy(), (-1., 1.).xy()).is_ccw(),
        "(1,1) (-1,1)"
    );

    assert!(
        orientation((1., 1.).xy(), (1., 5.).xy()).is_ccw(),
        "(1,1) (1,5)"
    );

    assert!(
        orientation((0., 0.).xy(), (1., 0.).xy()).is_neutral(),
        "(0,0) (1,0)"
    );
}

#[test]
fn test_sort_clockwise() {
    let center = [0., 0.];
    let mut points = vec![
        [0., 1.],    // 12 o clock
        [1., 0.5],   // 2 o clock
        [1., -0.5],  // 4 o clock
        [0., -1.],   // 6 o clock
        [-1., -0.5], // 8 o clock
        [-1., 0.5],  // 10 o clock
    ];

    points.reverse();

    let mut sorted_points = points.clone();
    sorted_points.sort_by(|a, b| sort_clockwise(center, *a, *b));

    assert_eq!(sorted_points[0], points[5]);
    assert_eq!(sorted_points[1], points[4]);
    assert_eq!(sorted_points[2], points[3]);
    assert_eq!(sorted_points[3], points[2]);
    assert_eq!(sorted_points[4], points[1]);
    assert_eq!(sorted_points[5], points[0]);

    print!("{:?}", sorted_points);
}
