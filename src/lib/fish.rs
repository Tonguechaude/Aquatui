use rand::{
    Rng,
    distr::{Distribution, weighted::WeightedIndex},
    prelude::IndexedRandom,
    rngs::ThreadRng,
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
                (
                    r#"
>() >()   >()   >()
 >() >()   >()    >()
    >()  >() >() >() >()
>()   >()     >()   >()"#
                        .to_string(),
                    1,
                ),
            ],
            Direction::Left => vec![
                ("<°))><".to_string(), 10),
                ("<><".to_string(), 10),
                ("<(((º<".to_string(), 10),
                (
                    r#"
  /
 / \\
<')_=<
 \\_/
  \\"#
                    .to_string(),
                    10,
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
                (
                    r#"
()< ()<   ()<   ()<
 ()< ()<   ()<    ()<
   ()<  ()< ()< ()< ()<
()<   ()<     ()<   ()<"#
                        .to_string(),
                    10,
                ),
                (
                    r#"
                          A       ;
                |   ,--,-/ \---,-/|  ,
               _|\,'. /|      /|   `/|-.
           \`.'    /|      ,            `;.
          ,'\   A     A         A   A _ /| `.;
        ,/  _              A       _  / _   /|  ;
       /\  / \   ,  ,           A  /    /     `/|
      /_| | _ \         ,     ,             ,/  \
     // | |/ `.\  ,-      ,       ,   ,/ ,/      \/
     / @| |@  / /'   \  \      ,              >  /|    ,--.
    |\_/   \_/ /      |  |           ,  ,/        \  ./' __:..
    |  __ __  |       |  | .--.  ,         >  >   |-'   /     `
  ,/| /  '  \ |       |  |     \      ,           |    /
 /  |<--.__,->|       |  | .    `.        >  >    /   (
/_,' \\  ^  /  \     /  /   `.    >--            /^\   |
      \\___/    \   /  /      \__'     \   \   \/   \  |
       `.   |/          ,  ,                  /`\    \  )
         \  '  |/    ,       V    \          /        `-\
          `|/  '  V      V           \    \.'            \_
           '`-.       V       V        \./'\
               `|/-.      \ /   \ /,---`\
                /   `._____V_____V'
                           '     '"#
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
                match self.direction {
                    Direction::Right => self.x < max_x + 10,
                    Direction::Left => self.x > 0u16.saturating_sub(10),
                }
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
