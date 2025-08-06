use rand::Rng;
use rand::rngs::ThreadRng;
use rand::{
    distr::{Distribution, weighted::WeightedIndex},
    prelude::IndexedRandom,
};
use ratatui::style::Color;

#[derive(Clone)]
pub enum Direction {
    Left,
    Right,
}

#[derive(Clone)]
pub struct Fish {
    pub x: u16,
    pub y: u16,
    pub body: Vec<String>,
    pub color: Color,
    pub speed: u16,
    pub direction: Direction,
}

impl PartialEq for Fish {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}

pub struct Bubble {
    pub x: u16,
    pub y: u16,
    pub frame: usize,
    pub z_index: u8,
    pub color: Color,
}

pub struct Seaweed {
    pub x: u16,
    pub y: u16,
    pub frame: usize,
    pub z_index: u8,
    pub height: usize,
}

pub struct Turtle {
    pub x: u16,
    pub y: u16,
    pub body: Vec<String>,
    pub color: Color,
    pub speed: u16,
    pub direction: Direction,
    pub frame: usize,
}

pub struct Shark {
    pub x: u16,
    pub y: u16,
    pub body: Vec<String>,
    pub color: Color,
    pub speed: u16,
    pub direction: Direction,
}

impl Fish {
    pub fn new(max_x: u16, max_y: u16, rng: &mut ThreadRng) -> Self {
        let direction = if rng.random_bool(0.5) {
            Direction::Right
        } else {
            Direction::Left
        };
        let x = match direction {
            Direction::Right => 0,
            Direction::Left => max_x,
        };
        let raw_bodies = match direction {
            Direction::Right => vec![
                ("><((°>".to_string(), 3),
                ("><>".to_string(), 3),
                (">º)))>".to_string(), 3),
                (
                    r#"
  \\
 / \\
>=_('>
 \\_/
   /"#
                    .to_string(),
                    1,
                ),
                (
                    r#"
        ,--,_
__    _\\.---'-.
\\ '.-"     // o\\
/_.'-.-_   \\\\  /
        `"--(/"#
                        .to_string(),
                    1,
                ),
            ],
            Direction::Left => vec![
                ("<°))><".to_string(), 3),
                ("<><".to_string(), 3),
                ("<(((º<".to_string(), 3),
                (
                    r#"
  /
 / \\
<')_=<
 \\_/
  \\"#
                    .to_string(),
                    2,
                ),
                (
                    r#"
    _,--,
 .-'---./___    __
/o \\\\     "-.' /
\\  //    _.-'._\\
  `"\\)--"#
                        .to_string(),
                    1,
                ),
            ],
        };

        let speed = rng.random_range(1..3);
        let weights: Vec<u32> = raw_bodies.iter().map(|(_, weight)| *weight).collect();
        let dist = WeightedIndex::new(&weights).unwrap();
        let choice = dist.sample(rng);
        let (raw_body, _) = &raw_bodies[choice];
        let body = raw_body.lines().map(|l| l.trim_end().to_string()).collect();
        let color = *[
            Color::Cyan,
            Color::LightMagenta,
            Color::Yellow,
            Color::LightBlue,
        ]
        .choose(rng)
        .unwrap();

        Self {
            x,
            y: rng.random_range(2..max_y.saturating_sub(4)),
            body,
            color,
            speed,
            direction,
        }
    }

    pub fn update(&mut self, max_x: u16) -> bool {
        match self.direction {
            Direction::Right => {
                self.x += self.speed;
                self.x < max_x + 10
            }
            Direction::Left => {
                if self.x <= self.speed {
                    false
                } else {
                    self.x -= self.speed;
                    true
                }
            }
        }
    }
}

impl Bubble {
    const FRAMES: [&'static str; 6] = [".", "·", "o", "°", "O", "0"];

    pub fn new(x: u16, y: u16, z_index: u8, rng: &mut ThreadRng) -> Self {
        let color = *[
            Color::Cyan,
            Color::LightBlue,
            Color::White,
            Color::LightCyan,
        ]
        .choose(rng)
        .unwrap();
        Self {
            x,
            y,
            frame: 0,
            z_index,
            color,
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

impl Seaweed {
    const FRAMES: [&'static str; 2] = [
        "
        (
        )
        (
        )
        (",
        "
        )
        (
        )
        (
        )",
    ];

    pub fn new(x: u16, y: u16, z_index: u8, rng: &mut ThreadRng) -> Self {
        Self {
            x,
            y,
            frame: 0,
            z_index,
            height: rng.random_range(2..=6),
        }
    }

    pub fn update(&mut self) {
        self.frame = (self.frame + 1) % Self::FRAMES.len();
    }

    pub fn current_symbol(&self) -> Vec<&str> {
        Self::FRAMES[self.frame].lines().map(str::trim).collect()
    }
}
