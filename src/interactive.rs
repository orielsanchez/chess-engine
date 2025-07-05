use crate::moves::Move;
use crate::position::{Position, PositionError};
use crate::search::SearchEngine;
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub enum InteractiveCommand {
    Analyze,
    Legal,
    Move {
        algebraic_move: String,
    },
    Position {
        fen: String,
    },
    Undo,
    Help,
    // Phase 4: Interactive Features
    Play {
        player_color: String,
        difficulty: u8,
    },
    Puzzle {
        puzzle_id: String,
    },
    Threats,
    Hint,
    Clock,
}

#[derive(Debug, Clone)]
pub enum InteractiveResponse {
    Analysis {
        evaluation: i32,
        best_move: String,
        depth: u32,
    },
    LegalMoves {
        moves: Vec<String>,
    },
    MoveResult {
        success: bool,
        resulting_fen: String,
    },
    PositionSet {
        success: bool,
        fen: String,
    },
    UndoResult {
        success: bool,
        resulting_fen: String,
    },
    Help {
        commands: String,
    },
    // Phase 4: Interactive Features
    GameStarted {
        mode: String,
        player_color: String,
    },
    PuzzleLoaded {
        objective: String,
        puzzle_id: String,
    },
    ThreatsFound {
        threat_count: usize,
        threats: Vec<String>,
    },
    PuzzleHint {
        hint: String,
    },
    GameClock {
        white_time_ms: u64,
        black_time_ms: u64,
    },
}

pub struct InteractiveEngine {
    position: Position,
    position_history: VecDeque<Position>,
}

impl InteractiveEngine {
    /// Create a new interactive engine
    ///
    /// # Errors
    ///
    /// Returns `PositionError` if the starting position cannot be created
    pub fn new() -> Result<Self, PositionError> {
        Ok(Self {
            position: Position::starting_position()?,
            position_history: VecDeque::new(),
        })
    }

    #[must_use]
    pub const fn current_position(&self) -> &Position {
        &self.position
    }

    pub fn parse_command(input: &str) -> Result<InteractiveCommand, String> {
        let input = input.trim();
        let parts: Vec<&str> = input.split_whitespace().collect();

        if parts.is_empty() {
            return Err("Empty command".to_string());
        }

        match parts[0] {
            "analyze" => Ok(InteractiveCommand::Analyze),
            "legal" => Ok(InteractiveCommand::Legal),
            "move" => {
                if parts.len() != 2 {
                    return Err("Move command requires exactly one argument".to_string());
                }
                Ok(InteractiveCommand::Move {
                    algebraic_move: parts[1].to_string(),
                })
            }
            "position" => {
                if parts.len() < 2 {
                    return Err("Position command requires FEN string".to_string());
                }
                let fen = parts[1..].join(" ");
                Ok(InteractiveCommand::Position { fen })
            }
            "undo" => Ok(InteractiveCommand::Undo),
            "help" => Ok(InteractiveCommand::Help),
            // Phase 4: Interactive Features
            "play" => {
                if parts.len() != 3 {
                    return Err(
                        "Play command requires color and difficulty: play <white|black> <1-10>"
                            .to_string(),
                    );
                }
                let player_color = parts[1].to_string();
                let difficulty = parts[2]
                    .parse::<u8>()
                    .map_err(|_| "Difficulty must be a number between 1-10".to_string())?;
                if !(1..=10).contains(&difficulty) {
                    return Err("Difficulty must be between 1 and 10".to_string());
                }
                if player_color != "white" && player_color != "black" {
                    return Err("Color must be 'white' or 'black'".to_string());
                }
                Ok(InteractiveCommand::Play {
                    player_color,
                    difficulty,
                })
            }
            "puzzle" => {
                if parts.len() != 2 {
                    return Err("Puzzle command requires puzzle ID: puzzle <id>".to_string());
                }
                Ok(InteractiveCommand::Puzzle {
                    puzzle_id: parts[1].to_string(),
                })
            }
            "threats" => Ok(InteractiveCommand::Threats),
            "hint" => Ok(InteractiveCommand::Hint),
            "clock" => Ok(InteractiveCommand::Clock),
            _ => Err(format!("Unknown command: {}", parts[0])),
        }
    }

    pub fn handle_command(
        &mut self,
        command: InteractiveCommand,
    ) -> Result<InteractiveResponse, String> {
        match command {
            InteractiveCommand::Analyze => self.handle_analyze(),
            InteractiveCommand::Legal => self.handle_legal(),
            InteractiveCommand::Move { algebraic_move } => self.handle_move(&algebraic_move),
            InteractiveCommand::Position { fen } => self.handle_position(&fen),
            InteractiveCommand::Undo => self.handle_undo(),
            InteractiveCommand::Help => self.handle_help(),
            // Phase 4: Interactive Features
            InteractiveCommand::Play {
                player_color,
                difficulty,
            } => self.handle_play(&player_color, difficulty),
            InteractiveCommand::Puzzle { puzzle_id } => self.handle_puzzle(&puzzle_id),
            InteractiveCommand::Threats => self.handle_threats(),
            InteractiveCommand::Hint => self.handle_hint(),
            InteractiveCommand::Clock => self.handle_clock(),
        }
    }

    fn handle_analyze(&self) -> Result<InteractiveResponse, String> {
        let evaluation = self.position.evaluate();

        // Use search engine to find best move
        let mut search_engine = SearchEngine::new();
        let result = search_engine
            .find_best_move(&self.position)
            .map_err(|e| format!("Search error: {}", e))?;

        let best_move = result.best_move.to_algebraic();

        Ok(InteractiveResponse::Analysis {
            evaluation,
            best_move,
            depth: result.depth as u32,
        })
    }

    fn handle_legal(&self) -> Result<InteractiveResponse, String> {
        let legal_moves = self
            .position
            .generate_legal_moves()
            .map_err(|e| format!("Move generation error: {}", e))?;

        let moves: Vec<String> = legal_moves.into_iter().map(|m| m.to_algebraic()).collect();

        Ok(InteractiveResponse::LegalMoves { moves })
    }

    fn handle_move(&mut self, algebraic_move: &str) -> Result<InteractiveResponse, String> {
        // Parse the move
        let chess_move = match Move::from_algebraic(algebraic_move) {
            Ok(m) => m,
            Err(_) => {
                return Ok(InteractiveResponse::MoveResult {
                    success: false,
                    resulting_fen: self.position.to_fen(),
                });
            }
        };

        // Check if move is legal
        let legal_moves = self
            .position
            .generate_legal_moves()
            .map_err(|e| format!("Move generation error: {}", e))?;

        if !legal_moves.contains(&chess_move) {
            return Ok(InteractiveResponse::MoveResult {
                success: false,
                resulting_fen: self.position.to_fen(),
            });
        }

        // Save current position to history
        self.position_history.push_back(self.position.clone());

        // Apply the move (modify position in place)
        match self.position.apply_move_for_search(chess_move) {
            Ok(()) => Ok(InteractiveResponse::MoveResult {
                success: true,
                resulting_fen: self.position.to_fen(),
            }),
            Err(_) => {
                // Remove from history if move failed
                self.position_history.pop_back();
                Ok(InteractiveResponse::MoveResult {
                    success: false,
                    resulting_fen: self.position.to_fen(),
                })
            }
        }
    }

    fn handle_position(&mut self, fen: &str) -> Result<InteractiveResponse, String> {
        match Position::from_fen(fen) {
            Ok(position) => {
                // Clear history when setting new position
                self.position_history.clear();
                self.position = position;
                Ok(InteractiveResponse::PositionSet {
                    success: true,
                    fen: fen.to_string(),
                })
            }
            Err(_) => Ok(InteractiveResponse::PositionSet {
                success: false,
                fen: self.position.to_fen(),
            }),
        }
    }

    fn handle_undo(&mut self) -> Result<InteractiveResponse, String> {
        if let Some(previous_position) = self.position_history.pop_back() {
            self.position = previous_position;
            Ok(InteractiveResponse::UndoResult {
                success: true,
                resulting_fen: self.position.to_fen(),
            })
        } else {
            Ok(InteractiveResponse::UndoResult {
                success: false,
                resulting_fen: self.position.to_fen(),
            })
        }
    }

    fn handle_help(&self) -> Result<InteractiveResponse, String> {
        let commands =
            "analyze legal move position undo help play puzzle threats hint clock".to_string();
        Ok(InteractiveResponse::Help { commands })
    }

    // Phase 4: Interactive Feature Handlers
    fn handle_play(
        &self,
        player_color: &str,
        difficulty: u8,
    ) -> Result<InteractiveResponse, String> {
        // Note: This will be bridged to TuiApp later
        Ok(InteractiveResponse::GameStarted {
            mode: format!("Playing vs Engine (difficulty {})", difficulty),
            player_color: player_color.to_string(),
        })
    }

    fn handle_puzzle(&self, puzzle_id: &str) -> Result<InteractiveResponse, String> {
        // Note: This will be bridged to TuiApp later
        Ok(InteractiveResponse::PuzzleLoaded {
            objective: "White to play and mate in 2".to_string(),
            puzzle_id: puzzle_id.to_string(),
        })
    }

    fn handle_threats(&self) -> Result<InteractiveResponse, String> {
        // Note: This will be bridged to TuiApp later
        // For now, return placeholder threat data
        let threats = vec!["e4:d5".to_string(), "d1:h5".to_string()];
        Ok(InteractiveResponse::ThreatsFound {
            threat_count: threats.len(),
            threats,
        })
    }

    fn handle_hint(&self) -> Result<InteractiveResponse, String> {
        // Note: This will be bridged to TuiApp later
        Ok(InteractiveResponse::PuzzleHint {
            hint: "Look for a forcing move that leads to mate".to_string(),
        })
    }

    const fn handle_clock(&self) -> Result<InteractiveResponse, String> {
        // Note: This will be bridged to TuiApp later
        Ok(InteractiveResponse::GameClock {
            white_time_ms: 300_000, // 5 minutes
            black_time_ms: 300_000, // 5 minutes
        })
    }

    #[allow(clippy::cast_precision_loss)]
    pub fn format_response(response: &InteractiveResponse) -> String {
        match response {
            InteractiveResponse::Analysis {
                evaluation,
                best_move,
                depth,
            } => {
                let eval_str = if *evaluation >= 0 {
                    format!("+{:.2}", *evaluation as f32 / 100.0)
                } else {
                    format!("{:.2}", *evaluation as f32 / 100.0)
                };
                format!(
                    "Evaluation: {}\nBest move: {}\nDepth: {}",
                    eval_str, best_move, depth
                )
            }
            InteractiveResponse::LegalMoves { moves } => {
                format!("Legal moves: {}", moves.join(" "))
            }
            InteractiveResponse::MoveResult {
                success,
                resulting_fen,
            } => {
                if *success {
                    format!("Move successful\nPosition: {}", resulting_fen)
                } else {
                    format!("Invalid move\nPosition: {}", resulting_fen)
                }
            }
            InteractiveResponse::PositionSet { success, fen } => {
                if *success {
                    format!("Position set: {}", fen)
                } else {
                    format!("Invalid FEN: {}", fen)
                }
            }
            InteractiveResponse::UndoResult {
                success,
                resulting_fen,
            } => {
                if *success {
                    format!("Move undone\nPosition: {}", resulting_fen)
                } else {
                    format!("No moves to undo\nPosition: {}", resulting_fen)
                }
            }
            InteractiveResponse::Help { commands } => {
                format!("Available commands: {}", commands)
            }
            // Phase 4: Interactive Feature Responses
            InteractiveResponse::GameStarted { mode, player_color } => {
                format!("Game started: {} as {}", mode, player_color)
            }
            InteractiveResponse::PuzzleLoaded {
                objective,
                puzzle_id,
            } => {
                format!("Puzzle loaded: {}\nObjective: {}", puzzle_id, objective)
            }
            InteractiveResponse::ThreatsFound {
                threat_count,
                threats,
            } => {
                if *threat_count == 0 {
                    "No threats found in current position".to_string()
                } else {
                    format!("Threats found ({}): {}", threat_count, threats.join(", "))
                }
            }
            InteractiveResponse::PuzzleHint { hint } => {
                format!("Hint: {}", hint)
            }
            InteractiveResponse::GameClock {
                white_time_ms,
                black_time_ms,
            } => {
                let white_minutes = white_time_ms / 60000;
                let white_seconds = (white_time_ms % 60000) / 1000;
                let black_minutes = black_time_ms / 60000;
                let black_seconds = (black_time_ms % 60000) / 1000;
                format!(
                    "Game Clock - White: {}:{:02} | Black: {}:{:02}",
                    white_minutes, white_seconds, black_minutes, black_seconds
                )
            }
        }
    }
}
