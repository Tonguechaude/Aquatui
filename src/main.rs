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

enum Direction {
    Left,
    Right,
}

struct Fish {
    x: u16,
    y: u16,
    body: Vec<String>, // Chaque poisson est maintenant composé de plusieurs lignes
    color: Color,
    speed: u16,
    direction: Direction,
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

        let bodies = match direction {
            Direction::Right => vec![
                "><((°>".to_string(),
                "><>".to_string(),
                ">º)))>".to_string(),
                "⩿⩾⩽⩾".to_string(),
                r#"\\
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
                r#" /
                   / \\
                  <')_=<
                   \\_/
                    \\"#
                .to_string(),
            ],
        };

        let body = bodies;

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

    fn update(&mut self, max_x: u16) {
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

    let num_fish = 15;
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

            // Fond océan dégradé
            for i in 0..lines.len() {
                let shade = match i {
                    0..=3 => Color::Blue,
                    4..=6 => Color::Rgb(0, 0, 139),
                    _ => Color::Black,
                };
                lines[i] = Line::from(Span::styled(
                    " ".repeat(inner.width as usize),
                    Style::default().bg(shade),
                ));
            }

            // Affichage des poissons (multi-lignes)
            for fish in &fishes {
                for (i, body_line) in fish.body.iter().enumerate() {
                    if (fish.y as usize + i) < lines.len() {
                        let mut line = String::new();
                        let display_x = fish
                            .x
                            .min(inner.width.saturating_sub(body_line.len() as u16));
                        line.push_str(&" ".repeat(display_x as usize));
                        line.push_str(body_line);
                        lines[fish.y as usize + i] =
                            Line::from(Span::styled(line, Style::default().fg(fish.color)));
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
                    if x < content.spans.len() {
                        content.spans[x] = bubble_span;
                    } else {
                        content.spans.push(bubble_span);
                    }
                    lines[bubble.y as usize] = content;
                }
            }

            let paragraph = Paragraph::new(lines);
            f.render_widget(paragraph, inner);
        })?;

        if last_frame.elapsed() >= frame_duration {
            // Mise à jour des poissons
            for fish in &mut fishes {
                fish.update(cols - 2);
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
