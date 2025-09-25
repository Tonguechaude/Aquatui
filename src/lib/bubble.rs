use rand::{Rng, rngs::ThreadRng};

pub struct Bubble {
    pub x: u16,
    pub y: u16,
    pub frame: usize, // pour alterner entre ".", "o", "0"
    pub z_index: u8,
}

impl Bubble {
    const FRAMES: [&'static str; 3] = [".", "o", "0"];

    pub fn new(x: u16, y: u16, z_index: u8) -> Self {
        Self {
            x,
            y,
            frame: 0,
            z_index,
        }
    }

    pub fn update(&mut self, max_y: u16, max_x: u16, rng: &mut ThreadRng) {
        if self.y > 0 {
            self.y -= 1;
        } else {
            self.y = max_y;
            self.x = rng.random_range(1..max_x);
        }

        self.frame = (self.frame + 1) % Self::FRAMES.len();
    }

    pub fn current_symbol(&self) -> &'static str {
        Self::FRAMES[self.frame]
    }
}
