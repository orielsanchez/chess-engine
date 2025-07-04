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
                    KeyCode::Esc => {
                        // Toggle menu - open if not in menu, close if in menu
                        match app.state() {
                            TuiState::Menu => app.set_state(TuiState::Command),
                            _ => app.set_state(TuiState::Menu),
                        }
                    }
                    KeyCode::Char('1') => {
                        if matches!(app.state(), TuiState::Menu) {
                            app.handle_menu_quick_game();
                        } else if matches!(app.state(), TuiState::Command | TuiState::GamePlay) {
                            app.add_char('1');
                        }
                    }
                    KeyCode::Char('2') => {
                        if matches!(app.state(), TuiState::Menu) {
                            app.handle_menu_puzzle();
                        } else if matches!(app.state(), TuiState::Command | TuiState::GamePlay) {
                            app.add_char('2');
                        }
                    }
                    KeyCode::Char('3') => {
                        if matches!(app.state(), TuiState::Menu) {
                            app.handle_menu_analysis();
                        } else if matches!(app.state(), TuiState::Command | TuiState::GamePlay) {
                            app.add_char('3');
                        }
                    }
                    KeyCode::Char('4') => {
                        if matches!(app.state(), TuiState::Menu) {
                            app.handle_menu_help();
                        } else if matches!(app.state(), TuiState::Command | TuiState::GamePlay) {
                            app.add_char('4');
                        }
                    }
                    KeyCode::Char('5') => {
                        if matches!(app.state(), TuiState::Menu) && app.handle_menu_quit() {
                            return Ok(());
                        } else if matches!(app.state(), TuiState::Command | TuiState::GamePlay) {
                            app.add_char('5');
                        }
                    }
                    KeyCode::Enter => {
                        if matches!(app.state(), TuiState::Command | TuiState::GamePlay)
                            && app.execute_command().is_err()
                        {
                            // Command execution failed - could show error message
                        }
                    }
                    KeyCode::Backspace => {
                        if matches!(app.state(), TuiState::Command | TuiState::GamePlay) {
                            app.remove_char();
                        }
                    }
                    KeyCode::Tab => {
                        if matches!(app.state(), TuiState::Command | TuiState::GamePlay) {
                            app.handle_tab_completion();
                        }
                    }
                    KeyCode::Up => {
                        if matches!(app.state(), TuiState::Command | TuiState::GamePlay) {
                            app.handle_history_up();
                        }
                    }
                    KeyCode::Down => {
                        if matches!(app.state(), TuiState::Command | TuiState::GamePlay) {
                            app.handle_history_down();
                        }
                    }
                    KeyCode::Left => {
                        if matches!(app.state(), TuiState::Command | TuiState::GamePlay) {
                            app.move_cursor_left();
                        }
                    }
                    KeyCode::Right => {
                        if matches!(app.state(), TuiState::Command | TuiState::GamePlay) {
                            app.move_cursor_right();
                        }
                    }
                    KeyCode::Char(c) => {
                        if matches!(app.state(), TuiState::Command | TuiState::GamePlay) {
                            app.add_char(c);
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}
