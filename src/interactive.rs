use crate::moves::Move;
use crate::position::{Position, PositionError};
use crate::search::SearchEngine;
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub enum InteractiveCommand {
    Analyze,
    Legal,
    Move { algebraic_move: String },
    Position { fen: String },
    Undo,
    Help,
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
}

pub struct InteractiveEngine {
    position: Position,
    position_history: VecDeque<Position>,
}

impl InteractiveEngine {
    pub fn new() -> Result<Self, PositionError> {
        Ok(Self {
            position: Position::starting_position()?,
            position_history: VecDeque::new(),
        })
    }

    pub fn current_position(&self) -> &Position {
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
        let commands = "analyze legal move position undo help".to_string();
        Ok(InteractiveResponse::Help { commands })
    }

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
        }
    }
}
