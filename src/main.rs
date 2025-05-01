use crossterm::{
    event::{Event, KeyCode, poll, read},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use rand::Rng;
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use std::{io, thread, time::Duration};

struct Fish {
    x: u16,
    y: u16,
    body: String,
}

impl Fish {
    fn new(_max_x: u16, max_y: u16) -> Self {
        let mut rng = rand::rng();
        Self {
            x: 0,
            y: rng.random_range(1..max_y),
            body: String::from("><((°>"),
        }
    }

    fn update(&mut self, max_x: u16) {
        self.x += 1;
        if self.x > max_x {
            self.x = 0;
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
    let mut fish = Fish::new(cols, rows - 2); // -2 pour éviter bordures

    loop {
        terminal.draw(|f| {
            let area = f.area();

            let block = Block::default()
                .title("Aquatui")
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White));
            f.render_widget(&block, area);

            let inner = block.inner(area);

            if fish.y < inner.height {
                let mut lines: Vec<Line> = vec![];

                for i in 0..inner.height {
                    if i == fish.y {
                        let mut line = String::new();
                        line.push_str(&" ".repeat(fish.x as usize));
                        line.push_str(&fish.body);
                        lines.push(Line::from(Span::styled(
                            line,
                            Style::default().fg(Color::Cyan),
                        )));
                    } else {
                        lines.push(Line::from(""));
                    }
                }

                let paragraph = Paragraph::new(lines);
                f.render_widget(paragraph, inner);
            }
        })?;

        fish.update(cols - 2);

        if poll(Duration::from_millis(10))? {
            if let Event::Key(key) = read()? {
                if key.code == KeyCode::Char('q') {
                    break;
                }
            }
        }

        thread::sleep(Duration::from_millis(100));
    }

    Ok(())
}
