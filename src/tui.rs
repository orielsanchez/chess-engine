use crate::interactive::InteractiveEngine;
use crate::moves::Move;
use crate::position::Position;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, Widget},
};
use std::collections::VecDeque;

#[derive(Debug, Clone, PartialEq)]
pub enum TuiState {
    Command,
    Board,
}

pub struct CommandCompletion {
    commands: Vec<String>,
    aliases: Vec<(String, String)>,
}

impl Default for CommandCompletion {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandCompletion {
    pub fn new() -> Self {
        Self {
            commands: vec![
                "analyze".to_string(),
                "legal".to_string(),
                "move".to_string(),
                "position".to_string(),
                "undo".to_string(),
                "help".to_string(),
            ],
            aliases: vec![
                ("a".to_string(), "analyze".to_string()),
                ("l".to_string(), "legal".to_string()),
                ("m".to_string(), "move".to_string()),
                ("p".to_string(), "position".to_string()),
                ("u".to_string(), "undo".to_string()),
                ("h".to_string(), "help".to_string()),
            ],
        }
    }

    pub fn complete_command(&self, input: &str) -> Vec<String> {
        if input.is_empty() {
            return self.commands.clone();
        }

        let mut completions = Vec::new();

        // Check direct command matches
        for command in &self.commands {
            if command.starts_with(input) {
                completions.push(command.clone());
            }
        }

        // Check alias matches
        for (alias, command) in &self.aliases {
            if alias.starts_with(input) && !completions.contains(command) {
                completions.push(command.clone());
            }
        }

        completions
    }

    pub fn complete_move(&self, app: &TuiApp, input: &str) -> Vec<String> {
        // Get legal moves from current position
        if let Ok(legal_moves) = app.position().generate_legal_moves() {
            let move_strings: Vec<String> =
                legal_moves.into_iter().map(|m| m.to_algebraic()).collect();

            if input.is_empty() {
                return move_strings;
            }

            return move_strings
                .into_iter()
                .filter(|m| m.starts_with(input))
                .collect();
        }

        Vec::new()
    }

    pub fn expand_alias(&self, input: &str) -> String {
        let parts: Vec<&str> = input.split_whitespace().collect();
        if parts.is_empty() {
            return input.to_string();
        }

        // Check if first part is an alias
        for (alias, command) in &self.aliases {
            if parts[0] == alias {
                let mut result = vec![command.clone()];
                result.extend(parts[1..].iter().map(|s| s.to_string()));
                return result.join(" ");
            }
        }

        input.to_string()
    }
}

pub struct CommandHistory {
    commands: VecDeque<String>,
    current_index: Option<usize>,
    max_size: usize,
}

impl Default for CommandHistory {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandHistory {
    pub fn new() -> Self {
        Self {
            commands: VecDeque::new(),
            current_index: None,
            max_size: 50,
        }
    }

    pub fn add_command(&mut self, command: String) {
        if command.trim().is_empty() {
            return;
        }

        // Don't add duplicates
        if self.commands.back() == Some(&command) {
            return;
        }

        self.commands.push_back(command);

        // Maintain max size
        if self.commands.len() > self.max_size {
            self.commands.pop_front();
        }

        // Reset index to end
        self.current_index = None;
    }

    pub fn get_previous(&mut self) -> Option<String> {
        if self.commands.is_empty() {
            return None;
        }

        match self.current_index {
            None => {
                // Start from the end
                self.current_index = Some(self.commands.len() - 1);
                self.commands.back().cloned()
            }
            Some(index) => {
                if index > 0 {
                    self.current_index = Some(index - 1);
                    self.commands.get(index - 1).cloned()
                } else {
                    None
                }
            }
        }
    }

    pub fn get_next(&mut self) -> Option<String> {
        match self.current_index {
            None => None,
            Some(index) => {
                if index < self.commands.len() - 1 {
                    self.current_index = Some(index + 1);
                    self.commands.get(index + 1).cloned()
                } else {
                    self.current_index = None;
                    None
                }
            }
        }
    }

    pub fn len(&self) -> usize {
        self.commands.len()
    }

    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }
}

pub struct TuiApp {
    engine: InteractiveEngine,
    state: TuiState,
    command_buffer: String,
    cursor_position: usize,
    last_response: Option<String>,
    completion: CommandCompletion,
    history: CommandHistory,
}

impl TuiApp {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            engine: InteractiveEngine::new()?,
            state: TuiState::Command,
            command_buffer: String::new(),
            cursor_position: 0,
            last_response: None,
            completion: CommandCompletion::new(),
            history: CommandHistory::new(),
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
        self.command_buffer.insert(self.cursor_position, c);
        self.cursor_position += 1;
    }

    pub fn remove_char(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
            self.command_buffer.remove(self.cursor_position);
        }
    }

    pub fn clear_command_buffer(&mut self) {
        self.command_buffer.clear();
        self.cursor_position = 0;
    }

    pub fn set_command_buffer(&mut self, command: String) {
        self.cursor_position = command.len();
        self.command_buffer = command;
    }

    pub fn cursor_position(&self) -> usize {
        self.cursor_position
    }

    pub fn move_cursor_left(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
        }
    }

    pub fn move_cursor_right(&mut self) {
        if self.cursor_position < self.command_buffer.len() {
            self.cursor_position += 1;
        }
    }

    pub fn insert_char_at_cursor(&mut self, c: char) {
        self.command_buffer.insert(self.cursor_position, c);
        self.cursor_position += 1;
    }

    pub fn handle_tab_completion(&mut self) -> bool {
        if self.command_buffer.is_empty() {
            // Show all available commands
            let completions = self.completion.complete_command("");
            if !completions.is_empty() {
                self.set_command_buffer(completions[0].clone());
                return true;
            }
            return false;
        }

        let parts: Vec<&str> = self.command_buffer.split_whitespace().collect();

        if parts.len() == 1 {
            // Complete command name
            let completions = self.completion.complete_command(parts[0]);
            if completions.len() == 1 {
                self.set_command_buffer(completions[0].clone());
                return true;
            } else if completions.len() > 1 {
                // Find common prefix
                let common_prefix = find_common_prefix(&completions);
                if common_prefix.len() > parts[0].len() {
                    self.set_command_buffer(common_prefix);
                    return true;
                }
            }
        } else if parts.len() == 2 && parts[0] == "move" {
            // Complete move
            let completions = self.completion.complete_move(self, parts[1]);
            if completions.len() == 1 {
                self.set_command_buffer(format!("move {}", completions[0]));
                return true;
            } else if completions.len() > 1 {
                let common_prefix = find_common_prefix(&completions);
                if common_prefix.len() > parts[1].len() {
                    self.set_command_buffer(format!("move {}", common_prefix));
                    return true;
                }
            }
        }

        false
    }

    pub fn handle_history_up(&mut self) {
        if let Some(command) = self.history.get_previous() {
            self.set_command_buffer(command);
        }
    }

    pub fn handle_history_down(&mut self) {
        if let Some(command) = self.history.get_next() {
            self.set_command_buffer(command);
        } else {
            self.clear_command_buffer();
        }
    }

    pub fn parse_natural_move(&self, input: &str) -> Result<Move, String> {
        Move::from_algebraic(input).map_err(|e| e.to_string())
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

        // Add to history before processing
        self.history.add_command(self.command_buffer.clone());

        // Expand aliases
        let expanded_command = self.completion.expand_alias(&self.command_buffer);

        let command = InteractiveEngine::parse_command(&expanded_command)?;
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
        let layout = self.create_layout(frame.area());
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

fn find_common_prefix(strings: &[String]) -> String {
    if strings.is_empty() {
        return String::new();
    }

    let first = &strings[0];
    let mut prefix_len = first.len();

    for string in strings.iter().skip(1) {
        let common_len = first
            .chars()
            .zip(string.chars())
            .take_while(|(a, b)| a == b)
            .count();
        prefix_len = prefix_len.min(common_len);
    }

    first.chars().take(prefix_len).collect()
}
