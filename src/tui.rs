use crate::interactive::InteractiveEngine;
use crate::position::Position;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, Widget},
};

#[derive(Debug, Clone, PartialEq)]
pub enum TuiState {
    Command,
    Board,
}

pub struct TuiApp {
    engine: InteractiveEngine,
    state: TuiState,
    command_buffer: String,
    last_response: Option<String>,
}

impl TuiApp {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            engine: InteractiveEngine::new()?,
            state: TuiState::Command,
            command_buffer: String::new(),
            last_response: None,
        })
    }

    pub fn position(&self) -> &Position {
        self.engine.current_position()
    }

    pub fn state(&self) -> &TuiState {
        &self.state
    }

    pub fn set_state(&mut self, state: TuiState) {
        self.state = state;
    }

    pub fn command_buffer(&self) -> &str {
        &self.command_buffer
    }

    pub fn add_char(&mut self, c: char) {
        self.command_buffer.push(c);
    }

    pub fn remove_char(&mut self) {
        self.command_buffer.pop();
    }

    pub fn clear_command_buffer(&mut self) {
        self.command_buffer.clear();
    }

    pub fn update_position(&mut self, position: Position) {
        // This is a simplified version - in a real implementation
        // we'd need to update the engine's internal state
        // For now, we'll implement this through FEN
        let fen = position.to_fen();
        if let Ok(cmd) = InteractiveEngine::parse_command(&format!("position {}", fen)) {
            if self.engine.handle_command(cmd).is_ok() {
                // Position updated successfully
            }
        }
    }

    pub fn execute_command(&mut self) -> Result<(), String> {
        if self.command_buffer.is_empty() {
            return Ok(());
        }

        let command = InteractiveEngine::parse_command(&self.command_buffer)?;
        let response = self.engine.handle_command(command)?;
        let formatted_response = InteractiveEngine::format_response(&response);

        self.last_response = Some(formatted_response);
        self.clear_command_buffer();

        Ok(())
    }

    pub fn create_layout(&self, area: Rect) -> Vec<Rect> {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(65), // Board area
                Constraint::Percentage(35), // Command area
            ])
            .split(area)
            .to_vec()
    }

    pub fn create_board_widget<'a>(&self, position: &'a Position) -> BoardWidget<'a> {
        BoardWidget::new(position)
    }

    pub fn create_command_widget(&self) -> CommandWidget {
        CommandWidget::new(&self.command_buffer, self.last_response.as_deref())
    }

    pub fn render(&self, frame: &mut Frame) {
        let layout = self.create_layout(frame.size());
        let board_area = layout[0];
        let command_area = layout[1];

        // Render board
        let board_widget = self.create_board_widget(self.position());
        frame.render_widget(board_widget, board_area);

        // Render command area
        let command_widget = self.create_command_widget();
        frame.render_widget(command_widget, command_area);
    }
}

pub struct BoardWidget<'a> {
    position: &'a Position,
}

impl<'a> BoardWidget<'a> {
    pub fn new(position: &'a Position) -> Self {
        Self { position }
    }

    pub fn title(&self) -> Option<&str> {
        Some("Chess Board")
    }

    pub fn has_borders(&self) -> bool {
        true
    }
}

impl<'a> Widget for BoardWidget<'a> {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let board_display = self.position.to_ascii_board();

        let paragraph = Paragraph::new(board_display)
            .block(
                Block::default()
                    .title("Chess Board")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::White)),
            )
            .style(Style::default().fg(Color::White));

        paragraph.render(area, buf);
    }
}

pub struct CommandWidget<'a> {
    command_buffer: &'a str,
    last_response: Option<&'a str>,
}

impl<'a> CommandWidget<'a> {
    pub fn new(command_buffer: &'a str, last_response: Option<&'a str>) -> Self {
        Self {
            command_buffer,
            last_response,
        }
    }

    pub fn title(&self) -> Option<&str> {
        Some("Commands")
    }

    pub fn has_borders(&self) -> bool {
        true
    }

    pub fn content(&self) -> String {
        let mut content = String::new();

        if let Some(response) = self.last_response {
            content.push_str("Last response:\n");
            content.push_str(response);
            content.push_str("\n\n");
        }

        content.push_str("Command: ");
        content.push_str(self.command_buffer);
        content.push('_'); // Cursor indicator

        content
    }
}

impl<'a> Widget for CommandWidget<'a> {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let content = self.content();

        let paragraph = Paragraph::new(content)
            .block(
                Block::default()
                    .title("Commands")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::White)),
            )
            .style(Style::default().fg(Color::White));

        paragraph.render(area, buf);
    }
}
