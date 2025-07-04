use crate::interactive::InteractiveEngine;
use crate::moves::Move;
use crate::position::Position;
use crate::search::SearchResult;
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

#[derive(Debug, Clone, PartialEq)]
pub enum LayoutMode {
    TwoPanelClassic,
    ThreePanelAnalysis,
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
    search_result: Option<SearchResult>,
    layout_mode: LayoutMode,
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
            search_result: None,
            layout_mode: LayoutMode::TwoPanelClassic,
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
        match self.layout_mode {
            LayoutMode::TwoPanelClassic => {
                self.create_layout_with_mode(LayoutMode::TwoPanelClassic, area)
            }
            LayoutMode::ThreePanelAnalysis => {
                // Use three-panel mode if we have search results, otherwise fall back to two-panel
                if self.search_result.is_some() {
                    self.create_layout_with_mode(LayoutMode::ThreePanelAnalysis, area)
                } else {
                    self.create_layout_with_mode(LayoutMode::TwoPanelClassic, area)
                }
            }
        }
    }

    pub fn create_layout_with_mode(&self, mode: LayoutMode, area: Rect) -> Vec<Rect> {
        match mode {
            LayoutMode::TwoPanelClassic => {
                Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Percentage(65), // Board area
                        Constraint::Percentage(35), // Command area
                    ])
                    .split(area)
                    .to_vec()
            }
            LayoutMode::ThreePanelAnalysis => {
                // Split horizontally: Board (50%) | Right side (50%)
                let horizontal_layout = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Percentage(50), // Board area
                        Constraint::Percentage(50), // Right side (commands + analysis)
                    ])
                    .split(area);

                // Split right side vertically: Commands (50%) | Analysis (50%)
                let right_side = horizontal_layout[1];
                let vertical_layout = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Percentage(50), // Command area
                        Constraint::Percentage(50), // Analysis area
                    ])
                    .split(right_side);

                vec![
                    horizontal_layout[0], // Board
                    vertical_layout[0],   // Commands
                    vertical_layout[1],   // Analysis
                ]
            }
        }
    }

    pub fn create_board_widget<'a>(&self, position: &'a Position) -> BoardWidget<'a> {
        BoardWidget::new(position)
    }

    pub fn create_command_widget(&self) -> CommandWidget {
        CommandWidget::new(&self.command_buffer, self.last_response.as_deref())
    }

    pub fn create_evaluation_widget<'a>(
        &self,
        search_result: &'a SearchResult,
    ) -> EvaluationWidget<'a> {
        EvaluationWidget::new(search_result)
    }

    pub fn create_principal_variation_widget<'a>(
        &self,
        search_result: &'a SearchResult,
    ) -> PrincipalVariationWidget<'a> {
        PrincipalVariationWidget::new(search_result)
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

        // Render analysis area if in three-panel mode and have search results
        if layout.len() > 2 {
            if let Some(ref search_result) = self.search_result {
                let analysis_area = layout[2];
                let pv_widget = self.create_principal_variation_widget(search_result);
                frame.render_widget(pv_widget, analysis_area);
            }
        }
    }

    pub fn set_search_result(&mut self, search_result: Option<SearchResult>) {
        self.search_result = search_result;
    }

    pub fn search_result(&self) -> &Option<SearchResult> {
        &self.search_result
    }

    pub fn set_layout_mode(&mut self, mode: LayoutMode) {
        self.layout_mode = mode;
    }

    pub fn layout_mode(&self) -> &LayoutMode {
        &self.layout_mode
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

pub struct EvaluationWidget<'a> {
    search_result: &'a SearchResult,
}

impl<'a> EvaluationWidget<'a> {
    pub fn new(search_result: &'a SearchResult) -> Self {
        Self { search_result }
    }

    pub fn title(&self) -> Option<&str> {
        Some("Evaluation")
    }

    pub fn has_borders(&self) -> bool {
        true
    }

    pub fn content(&self) -> String {
        let score = self.search_result.evaluation as f64 / 100.0;
        let score_str = if score > 0.0 {
            format!("+{:.2}", score)
        } else if score < 0.0 {
            format!("{:.2}", score)
        } else {
            "0.00".to_string()
        };

        let best_move_str = self.search_result.best_move.to_algebraic();

        // Phase 3: Advantage indicators
        let advantage_indicator = self.get_advantage_indicator(score);

        // Phase 3: Search performance metrics
        let nps = if self.search_result.time_ms > 0 {
            (self.search_result.nodes_searched * 1000) / self.search_result.time_ms
        } else {
            0
        };

        // Phase 3: Time management indicator
        let time_indicator = if self.search_result.time_limited {
            " (time limited)"
        } else {
            ""
        };

        // Phase 3: TT hit rate as percentage
        let tt_hit_rate_pct = (self.search_result.tt_hit_rate * 100.0) as u32;

        // Phase 3: Detailed evaluation breakdown (simplified for now)
        let material_eval = (self.search_result.evaluation as f64 * 0.6) / 100.0;
        let positional_eval = (self.search_result.evaluation as f64 * 0.3) / 100.0;
        let pawn_eval = (self.search_result.evaluation as f64 * 0.1) / 100.0;

        format!(
            "Score: {} ({})\nDepth: {}\nBest: {}\n\nNodes: {}\nTime: {}ms{}\nNPS: {}\n\nTT Hit: {}%\nTT Hits: {}\nTT Stores: {}\n\nAsp Fails: {}\nAsp Research: {}\nAsp Window: {}\n\nIterations: {}\nTarget Depth: {}\nCompleted: {}\n\nMaterial: {:+.2}\nPosition: {:+.2}\nPawns: {:+.2}",
            score_str,
            advantage_indicator,
            self.search_result.completed_depth,
            best_move_str,
            self.search_result.nodes_searched,
            self.search_result.time_ms,
            time_indicator,
            nps,
            tt_hit_rate_pct,
            self.search_result.tt_hits,
            self.search_result.tt_stores,
            self.search_result.aspiration_fails,
            self.search_result.aspiration_researches,
            self.search_result.aspiration_window_size,
            self.search_result.iterations_completed,
            self.search_result.depth,
            self.search_result.completed_depth,
            material_eval,
            positional_eval,
            pawn_eval
        )
    }

    fn get_advantage_indicator(&self, score: f64) -> &'static str {
        let abs_score = score.abs();
        match abs_score {
            x if x >= 5.0 => {
                if score > 0.0 {
                    "winning"
                } else {
                    "losing"
                }
            }
            x if x >= 3.0 => {
                if score > 0.0 {
                    "winning"
                } else {
                    "losing"
                }
            }
            x if x >= 2.0 => {
                if score > 0.0 {
                    "advantage"
                } else {
                    "disadvantage"
                }
            }
            x if x >= 1.0 => {
                if score > 0.0 {
                    "advantage"
                } else {
                    "disadvantage"
                }
            }
            x if x >= 0.50 => {
                if score > 0.0 {
                    "slight advantage"
                } else {
                    "slight disadvantage"
                }
            }
            x if x >= 0.25 => {
                if score > 0.0 {
                    "slight advantage"
                } else {
                    "slight disadvantage"
                }
            }
            _ => "equal",
        }
    }
}

impl<'a> Widget for EvaluationWidget<'a> {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let content = self.content();

        let paragraph = Paragraph::new(content)
            .block(
                Block::default()
                    .title("Evaluation")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::White)),
            )
            .style(Style::default().fg(Color::White));

        paragraph.render(area, buf);
    }
}

pub struct PrincipalVariationWidget<'a> {
    search_result: &'a SearchResult,
}

impl<'a> PrincipalVariationWidget<'a> {
    pub fn new(search_result: &'a SearchResult) -> Self {
        Self { search_result }
    }

    pub fn title(&self) -> Option<&str> {
        Some("Principal Variation")
    }

    pub fn has_borders(&self) -> bool {
        true
    }

    pub fn content(&self) -> String {
        let depth = self.search_result.completed_depth;
        let pv = &self.search_result.principal_variation;

        if pv.is_empty() {
            return format!("Depth: {}\n\nNo variation", depth);
        }

        let mut content = format!("Depth: {}\n\n", depth);

        // Format moves in pairs: "1. e4    e5"
        let mut move_number = 1;
        let mut i = 0;

        while i < pv.len() {
            let white_move = self.format_move_to_algebraic(&pv[i]);

            if i + 1 < pv.len() {
                // Both white and black moves
                let black_move = self.format_move_to_algebraic(&pv[i + 1]);
                content.push_str(&format!(
                    "{}. {}    {}\n",
                    move_number, white_move, black_move
                ));
                i += 2;
            } else {
                // Only white move
                content.push_str(&format!("{}. {}\n", move_number, white_move));
                i += 1;
            }

            move_number += 1;
        }

        content
    }

    fn format_move_to_algebraic(&self, move_obj: &Move) -> String {
        // For now, use simple algebraic notation - we'll enhance this later
        let algebraic = move_obj.to_algebraic();

        // Convert coordinate notation (e2e4) to simple algebraic (e4)
        // This is a basic conversion - we'll implement proper SAN later
        if algebraic.len() >= 4 {
            let from_square = &algebraic[0..2];
            let to_square = &algebraic[2..4];

            // Basic piece detection based on from square
            match from_square {
                // Pawns - just show destination
                "a2" | "b2" | "c2" | "d2" | "e2" | "f2" | "g2" | "h2" | "a7" | "b7" | "c7"
                | "d7" | "e7" | "f7" | "g7" | "h7" => {
                    to_square.to_string() // e.g., "e4" from "e2e4"
                }
                // Knights from starting positions
                "b1" | "g1" | "b8" | "g8" => {
                    format!("N{}", to_square)
                }
                // Bishops from starting positions
                "c1" | "f1" | "c8" | "f8" => {
                    format!("B{}", to_square)
                }
                // Other pieces - use coordinate notation for now
                _ => algebraic,
            }
        } else {
            algebraic
        }
    }
}

impl<'a> Widget for PrincipalVariationWidget<'a> {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let content = self.content();

        let paragraph = Paragraph::new(content)
            .block(
                Block::default()
                    .title("Principal Variation")
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
