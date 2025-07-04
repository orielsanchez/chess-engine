use chess_engine::tui::{TuiApp, TuiState};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let mut app = TuiApp::new()?;
    let res = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut TuiApp,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| app.render(f))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Char('t') => {
                        // Toggle between command and board mode
                        match app.state() {
                            TuiState::Command => app.set_state(TuiState::Board),
                            TuiState::Board => app.set_state(TuiState::Command),
                        }
                    }
                    KeyCode::Enter => {
                        if matches!(app.state(), TuiState::Command)
                            && app.execute_command().is_err()
                        {
                            // Command execution failed - could show error message
                        }
                    }
                    KeyCode::Backspace => {
                        if matches!(app.state(), TuiState::Command) {
                            app.remove_char();
                        }
                    }
                    KeyCode::Char(c) => {
                        if matches!(app.state(), TuiState::Command) {
                            app.add_char(c);
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}
