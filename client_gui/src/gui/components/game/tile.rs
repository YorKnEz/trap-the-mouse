use sfml::system::Vector2f;

#[derive(Clone, Copy, Default)]
pub struct Tile {
    pub blocked: bool,
    pub origin: Vector2f,
    pub points: [Vector2f; 6],
}

impl Tile {
    pub fn new(origin: Vector2f, off: &[Vector2f; 6]) -> Tile {
        let mut points = [Vector2f::new(0.0, 0.0); 6];

        for i in 0..6 {
            points[i] = Vector2f::new(origin.x + off[i].x, origin.y + off[i].y);
        }

        Tile {
            blocked: false,
            origin,
            points,
        }
    }

    pub fn inside(&self, x: u32, y: u32) -> bool {
        let p = Vector2f::new(x as f32, y as f32);

        !self.under_slope(p, self.points[5], self.points[0])
            && !self.under_slope(p, self.points[0], self.points[1])
            && self.under_slope(p, self.points[2], self.points[3])
            && self.under_slope(p, self.points[3], self.points[4])
            && self.points[5].x <= p.x
            && p.x <= self.points[1].x
    }

    fn under_slope(&self, p: Vector2f, a: Vector2f, b: Vector2f) -> bool {
        // ignore verticals
        if a.x == b.x {
            return false;
        }

        let g = (a.y - b.y) / (a.x - b.x);
        let y = g * (p.x - a.x) + a.y;

        p.y < y
    }

    // pub fn blocked(&self) -> bool { self.blocked }

    pub fn set_blocked(&mut self, value: bool) {
        self.blocked = value;
    }
}
