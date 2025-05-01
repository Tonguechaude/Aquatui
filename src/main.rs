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
use std::{
    io,
    thread::sleep,
    time::{Duration, Instant},
};

enum Direction {
    Left,
    Right,
}

struct Fish {
    x: u16,
    y: u16,
    body: String,
    speed: u16,
    direction: Direction,
}

impl Fish {
    fn new(max_x: u16, max_y: u16, rng: &mut ThreadRng) -> Self {
        let direction = if rng.random_bool(0.5) {
            Direction::Right
        } else {
            Direction::Left
        };

        let bodies = match direction {
            Direction::Right => vec!["><((°>", "><>", ">º)))>", "⩿⩾⩽⩾"],
            Direction::Left => vec!["<°))><", "<><", "<(((º<", "⩾⩽⩾⩿"],
        };

        let body = bodies.choose(rng).unwrap().to_string();

        let x = match direction {
            Direction::Right => 0,
            Direction::Left => max_x,
        };

        Self {
            x,
            y: rng.random_range(1..max_y),
            body,
            speed: rng.random_range(1..4),
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

fn run_app<B: ratatui::backend::Backend>(terminal: &mut Terminal<B>) -> io::Result<()> {
    let size = terminal.size()?;
    let (cols, rows) = (size.width, size.height);

    let mut rng = rand::rng();
    let num_fish = 15;
    let mut fishes: Vec<Fish> = (0..num_fish)
        .map(|_| Fish::new(cols - 2, rows - 2, &mut rng))
        .collect();

    let frame_duration = Duration::from_millis(60);
    let mut last_frame = Instant::now();

    loop {
        // Rendu
        terminal.draw(|f| {
            let area = f.area();

            let block = Block::default()
                .title("Aquatui")
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White));
            f.render_widget(&block, area);

            let inner = block.inner(area);

            let mut lines: Vec<Line> = vec![Line::from(""); inner.height as usize];

            for fish in &fishes {
                if (fish.y as usize) < lines.len() {
                    let mut line = String::new();
                    let display_x = fish
                        .x
                        .min(inner.width.saturating_sub(fish.body.len() as u16));
                    line.push_str(&" ".repeat(display_x as usize));
                    line.push_str(&fish.body);
                    lines[fish.y as usize] =
                        Line::from(Span::styled(line, Style::default().fg(Color::Cyan)));
                }
            }

            let paragraph = Paragraph::new(lines);
            f.render_widget(paragraph, inner);
        })?;

        // Mouvement selon le timer
        if last_frame.elapsed() >= frame_duration {
            for fish in &mut fishes {
                fish.update(cols - 2);
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
        sleep(Duration::from_millis(5));
    }

    Ok(())
}
