/// Trait for rooms
pub trait Room {
    /// Returns true if this overlaps with other
    fn intersect(&self, other: &dyn Room) -> bool {
        let _own_spaces = self.spaces();
        let _other_spaces = other.spaces();

        _own_spaces.iter()
            .filter(|x| _other_spaces.contains(x))
            .count() == 0
    }

    /// Returns a coordinate pair (x, y) of the center of the room
    fn center(&self) -> (i32, i32);

    /// Returns vector of coordinate pairs (x, y) of spaces that the room
    /// occupies
    fn spaces(&self) -> Vec<(i32, i32)>;
}
pub struct Rect {
    pub x1: i32,
    pub x2: i32,
    pub y1: i32,
    pub y2: i32,
}

impl Rect {
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Rect {
        Rect {
            x1: x,
            y1: y,
            x2: x + w,
            y2: y + h,
        }
    }
}

impl Room for Rect {
    fn center(&self) -> (i32, i32) {
        ((self.x1 + self.x2) / 2, (self.y1 + self.y2) / 2)
    }

    fn spaces(&self) -> Vec<(i32, i32)> {
        let mut _spaces = Vec::new();
        for y in self.y1 + 1..=self.y2 {
            for x in self.x1 + 1..=self.x2 {
                _spaces.push((x, y));
            }
        }
        _spaces
    }
}
