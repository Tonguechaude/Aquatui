use crossterm::{
    event::{Event, KeyCode, poll, read},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use rand::{Rng, prelude::IndexedRandom, rngs::ThreadRng};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use std::{io, thread, time::Duration, time::Instant};

#[derive(Clone, PartialEq)]
enum Direction {
    Left,
    Right,
}

#[derive(Clone)]
struct Fish {
    x: u16,
    y: u16,
    body: Vec<String>,
    color: Color,
    speed: u16,
    direction: Direction,
}

impl PartialEq for Fish {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}

struct Bubble {
    x: u16,
    y: u16,
    frame: usize, // pour alterner entre ".", "o", "0"
}

impl Fish {
    fn new(max_x: u16, max_y: u16, rng: &mut ThreadRng) -> Self {
        let direction = if rng.random_bool(0.5) {
            Direction::Right
        } else {
            Direction::Left
        };

        let raw_bodies = match direction {
            Direction::Right => vec![
                "><((°>".to_string(),
                "><>".to_string(),
                ">º)))>".to_string(),
                "⩿⩾⩽⩾".to_string(),
                r#"
                    \\
                   / \\
                  >=_('>
                   \\_/
                     /"#
                .to_string(),
            ],
            Direction::Left => vec![
                "<°))><".to_string(),
                "<><".to_string(),
                "<(((º<".to_string(),
                "⩾⩾⩾⩿".to_string(),
                r#"
                    /
                   / \\
                  <')_=<
                   \\_/
                    \\"#
                .to_string(),
            ],
        };

        let raw_body = raw_bodies.choose(rng).unwrap();
        let body = raw_body.lines().map(|l| l.trim_end().to_string()).collect();

        let x = match direction {
            Direction::Right => 0,
            Direction::Left => max_x,
        };

        let color = *[
            Color::Cyan,
            Color::LightMagenta,
            Color::Yellow,
            Color::LightBlue,
            Color::Green,
        ]
        .choose(rng)
        .unwrap();

        Self {
            x,
            y: rng.random_range(1..max_y),
            body,
            color,
            speed: rng.random_range(1..3),
            direction,
        }
    }

    fn update(&mut self, max_x: u16, fishes: &mut [&mut Fish]) {
        match self.direction {
            Direction::Right => {
                self.x += self.speed;
                if self.x > max_x {
                    self.x = 0;
                }
            }
            Direction::Left => {
                if self.x < self.speed {
                    self.x = max_x;
                } else {
                    self.x -= self.speed;
                }
            }
        }

        // Collisions
        for other_fish in fishes.iter_mut() {
            if self != *other_fish {
                if self.direction == Direction::Right && other_fish.direction == Direction::Left {
                    if self.x == other_fish.x && self.y == other_fish.y {
                        other_fish.x = max_x;
                    }
                }
            }
        }
    }
}

impl Bubble {
    const FRAMES: [&'static str; 3] = [".", "o", "0"];

    fn new(x: u16, y: u16) -> Self {
        Self { x, y, frame: 0 }
    }

    fn update(&mut self, max_y: u16, max_x: u16, rng: &mut ThreadRng) {
        if self.y > 0 {
            self.y -= 1;
        } else {
            self.y = max_y;
            self.x = rng.random_range(1..max_x);
        }

        self.frame = (self.frame + 1) % Self::FRAMES.len();
    }

    fn current_symbol(&self) -> &'static str {
        Self::FRAMES[self.frame]
    }
}

fn run_app<B: ratatui::backend::Backend>(terminal: &mut Terminal<B>) -> io::Result<()> {
    let size = terminal.size()?;
    let (cols, rows) = (size.width, size.height);
    let mut rng = rand::rng();
    let frame_duration = Duration::from_millis(60);
    let mut last_frame = Instant::now();

    let num_fish = 10;
    let mut fishes: Vec<Fish> = (0..num_fish)
        .map(|_| Fish::new(cols - 2, rows - 2, &mut rng))
        .collect();
    let mut bubbles: Vec<Bubble> = (0..20)
        .map(|_| Bubble::new(rng.random_range(1..cols - 1), rng.random_range(1..rows - 1)))
        .collect();

    loop {
        terminal.draw(|f| {
            let area = f.area();
            let block = Block::default()
                .title("Aquatui")
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White));
            f.render_widget(&block, area);

            let inner = block.inner(area);
            let mut lines = vec![Line::from(""); inner.height as usize];

            // Affichage des poissons (multi-lignes)
            for fish in &fishes {
                let max_body_width = fish.body.iter().map(|line| line.len()).max().unwrap_or(0);
                let display_x = fish
                    .x
                    .min(inner.width.saturating_sub(max_body_width as u16));
                for (i, body_line) in fish.body.iter().enumerate() {
                    if (fish.y as usize + i) < lines.len() {
                        let mut content = lines[fish.y as usize + i].clone();
                        while content.spans.len() < display_x as usize {
                            content.spans.push(Span::raw(" "));
                        }
                        content.spans.push(Span::styled(
                            body_line.clone(),
                            Style::default().fg(fish.color),
                        ));
                        lines[fish.y as usize + i] = content;
                    }
                }
            }

            // Bulles
            for bubble in &bubbles {
                if (bubble.y as usize) < lines.len() {
                    let x = bubble.x.min(inner.width - 1) as usize;
                    let symbol = bubble.current_symbol();
                    let bubble_span = Span::styled(symbol, Style::default().fg(Color::White));

                    let mut content = lines[bubble.y as usize].clone();
                    while content.spans.len() <= x {
                        content.spans.push(Span::raw(" "));
                    }
                    content.spans[x] = bubble_span;
                    lines[bubble.y as usize] = content;
                }
            }

            let paragraph = Paragraph::new(lines);
            f.render_widget(paragraph, inner);
        })?;

        if last_frame.elapsed() >= frame_duration {
            // Mise à jour des poissons
            for i in 0..fishes.len() {
                let (left, right) = fishes.split_at_mut(i);
                let (fish, right) = right.split_at_mut(1);
                let fish = &mut fish[0];
                let mut others: Vec<&mut Fish> = left.iter_mut().chain(right.iter_mut()).collect();
                fish.update(cols - 2, &mut others);
            }
            // Mise à jour des bulles
            for bubble in &mut bubbles {
                bubble.update(rows - 2, cols - 1, &mut rng);
            }
            last_frame = Instant::now();
        }

        // Gestion de la touche 'q'
        if poll(Duration::from_millis(1))? {
            if let Event::Key(key) = read()? {
                if key.code == KeyCode::Char('q') {
                    break;
                }
            }
        }

        // Pause très courte pour éviter 100% CPU
        thread::sleep(Duration::from_millis(20));
    }

    Ok(())
}

fn main() -> Result<(), io::Error> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let res = run_app(&mut terminal);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err);
    }

    Ok(())
}
