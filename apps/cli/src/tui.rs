use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io;

pub fn run_debugger(prompt: &str, sql: &str, explanation: &str) -> anyhow::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints(
                    [
                        Constraint::Length(3),
                        Constraint::Length(10),
                        Constraint::Min(5),
                    ]
                    .as_ref(),
                )
                .split(f.area());

            let prompt_widget = Paragraph::new(prompt)
                .block(Block::default().borders(Borders::ALL).title(" NATURAL LANGUAGE PROMPT "));
            f.render_widget(prompt_widget, chunks[0]);

            let sql_widget = Paragraph::new(sql)
                .block(Block::default().borders(Borders::ALL).title(" GENERATED SQL (AIQL CORE) "));
            f.render_widget(sql_widget, chunks[1]);

            let exp_widget = Paragraph::new(explanation)
                .block(Block::default().borders(Borders::ALL).title(" EXPLANATION "));
            f.render_widget(exp_widget, chunks[2]);
        })?;

        if let Event::Key(key) = event::read()? {
            if let KeyCode::Char('q') = key.code {
                break;
            }
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
