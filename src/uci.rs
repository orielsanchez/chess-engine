use crate::moves::Move;
use crate::position::{Position, PositionError};
use crate::search::SearchEngine;

#[derive(Debug, Clone)]
pub enum UciCommand {
    Uci,
    IsReady,
    Position {
        fen: Option<String>,
        moves: Vec<String>,
    },
    Go {
        depth: Option<u32>,
        movetime: Option<u64>,
        infinite: bool,
    },
    Stop,
    Quit,
}

#[derive(Debug, Clone)]
pub enum UciResponse {
    Id {
        name: String,
        author: String,
    },
    UciOk,
    ReadyOk,
    BestMove {
        best_move: String,
        ponder: Option<String>,
    },
    Info {
        depth: u32,
        nodes: u64,
        time: u64,
        pv: Vec<String>,
    },
}

pub struct UciEngine {
    position: Position,
}

impl UciEngine {
    pub fn new() -> Result<Self, PositionError> {
        Ok(Self {
            position: Position::starting_position()?,
        })
    }

    pub fn parse_command(input: &str) -> Result<UciCommand, String> {
        let parts: Vec<&str> = input.split_whitespace().collect();
        if parts.is_empty() {
            return Err("Empty command".to_string());
        }

        match parts[0] {
            "uci" => Ok(UciCommand::Uci),
            "isready" => Ok(UciCommand::IsReady),
            "quit" => Ok(UciCommand::Quit),
            "stop" => Ok(UciCommand::Stop),
            "position" => {
                if parts.len() < 2 {
                    return Err("Position command requires parameters".to_string());
                }

                let mut fen = None;
                let mut moves = Vec::new();
                let i = if parts[1] == "fen" {
                    // Extract FEN (parts 2-7)
                    if parts.len() < 8 {
                        return Err("FEN requires 6 components".to_string());
                    }
                    fen = Some(parts[2..8].join(" "));
                    8
                } else if parts[1] == "startpos" {
                    2
                } else {
                    return Err("Position must be 'startpos' or 'fen'".to_string());
                };

                // Parse moves if present
                if i < parts.len() && parts[i] == "moves" {
                    moves = parts[i + 1..].iter().map(|s| s.to_string()).collect();
                }

                Ok(UciCommand::Position { fen, moves })
            }
            "go" => {
                let mut depth = None;
                let mut movetime = None;
                let mut infinite = false;

                let mut i = 1;
                while i < parts.len() {
                    match parts[i] {
                        "depth" => {
                            if i + 1 < parts.len() {
                                depth = parts[i + 1].parse().ok();
                                i += 2;
                            } else {
                                return Err("Depth parameter missing value".to_string());
                            }
                        }
                        "movetime" => {
                            if i + 1 < parts.len() {
                                movetime = parts[i + 1].parse().ok();
                                i += 2;
                            } else {
                                return Err("Movetime parameter missing value".to_string());
                            }
                        }
                        "infinite" => {
                            infinite = true;
                            i += 1;
                        }
                        _ => i += 1,
                    }
                }

                Ok(UciCommand::Go {
                    depth,
                    movetime,
                    infinite,
                })
            }
            _ => Err(format!("Unknown command: {}", parts[0])),
        }
    }

    pub fn handle_command(&mut self, command: UciCommand) -> Result<Vec<UciResponse>, String> {
        match command {
            UciCommand::Uci => Ok(vec![
                UciResponse::Id {
                    name: "Chess Engine v0.1.0".to_string(),
                    author: "Rust Chess Engine Project".to_string(),
                },
                UciResponse::UciOk,
            ]),
            UciCommand::IsReady => Ok(vec![UciResponse::ReadyOk]),
            UciCommand::Position { fen, moves } => {
                // Set position
                if let Some(fen_str) = fen {
                    self.position =
                        Position::from_fen(&fen_str).map_err(|e| format!("FEN error: {}", e))?;
                } else {
                    self.position = Position::starting_position()
                        .map_err(|e| format!("Position error: {}", e))?;
                }

                // Apply moves
                for move_str in moves {
                    let mv = Move::from_algebraic(&move_str)
                        .map_err(|e| format!("Move parse error: {}", e))?;
                    self.position
                        .apply_move_for_search(mv)
                        .map_err(|e| format!("Move application error: {}", e))?;
                }

                Ok(vec![])
            }
            UciCommand::Go {
                depth,
                movetime,
                infinite: _,
            } => {
                let mut search = SearchEngine::new();
                let result = if let Some(time_ms) = movetime {
                    // Time-based search
                    search
                        .find_best_move_timed(&self.position, time_ms)
                        .map_err(|e| format!("Search error: {}", e))?
                } else {
                    // Depth-based search
                    let search_depth = depth.unwrap_or(5) as u8;
                    search
                        .find_best_move_constrained(&self.position, search_depth, 10000)
                        .map_err(|e| format!("Search error: {}", e))?
                };

                let best_move = result.best_move.to_algebraic();

                // Generate search info
                let info = UciResponse::Info {
                    depth: result.completed_depth as u32,
                    nodes: result.nodes_searched,
                    time: result.time_ms,
                    pv: vec![best_move.clone()],
                };

                Ok(vec![
                    info,
                    UciResponse::BestMove {
                        best_move,
                        ponder: None,
                    },
                ])
            }
            UciCommand::Stop => Ok(vec![]),
            UciCommand::Quit => Ok(vec![]),
        }
    }

    pub fn format_response(response: &UciResponse) -> String {
        match response {
            UciResponse::Id { name, author } => {
                format!("id name {}\nid author {}", name, author)
            }
            UciResponse::UciOk => "uciok".to_string(),
            UciResponse::ReadyOk => "readyok".to_string(),
            UciResponse::BestMove { best_move, ponder } => {
                if let Some(ponder_move) = ponder {
                    format!("bestmove {} ponder {}", best_move, ponder_move)
                } else {
                    format!("bestmove {}", best_move)
                }
            }
            UciResponse::Info {
                depth,
                nodes,
                time,
                pv,
            } => {
                format!(
                    "info depth {} nodes {} time {} pv {}",
                    depth,
                    nodes,
                    time,
                    pv.join(" ")
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_uci_command() {
        let result = UciEngine::parse_command("uci");
        assert!(matches!(result, Ok(UciCommand::Uci)));
    }

    #[test]
    fn test_parse_isready_command() {
        let result = UciEngine::parse_command("isready");
        assert!(matches!(result, Ok(UciCommand::IsReady)));
    }

    #[test]
    fn test_parse_position_startpos() {
        let result = UciEngine::parse_command("position startpos");
        assert!(
            matches!(result, Ok(UciCommand::Position { fen: None, moves }) if moves.is_empty())
        );
    }

    #[test]
    fn test_parse_position_startpos_with_moves() {
        let result = UciEngine::parse_command("position startpos moves e2e4 e7e5");
        assert!(
            matches!(result, Ok(UciCommand::Position { fen: None, moves }) if moves == vec!["e2e4", "e7e5"])
        );
    }

    #[test]
    fn test_parse_position_fen() {
        let fen = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1";
        let input = format!("position fen {}", fen);
        let result = UciEngine::parse_command(&input);
        assert!(
            matches!(result, Ok(UciCommand::Position { fen: Some(f), moves }) if f == fen && moves.is_empty())
        );
    }

    #[test]
    fn test_parse_go_depth() {
        let result = UciEngine::parse_command("go depth 10");
        assert!(matches!(
            result,
            Ok(UciCommand::Go {
                depth: Some(10),
                movetime: None,
                infinite: false
            })
        ));
    }

    #[test]
    fn test_parse_go_movetime() {
        let result = UciEngine::parse_command("go movetime 5000");
        assert!(matches!(
            result,
            Ok(UciCommand::Go {
                depth: None,
                movetime: Some(5000),
                infinite: false
            })
        ));
    }

    #[test]
    fn test_parse_go_infinite() {
        let result = UciEngine::parse_command("go infinite");
        assert!(matches!(
            result,
            Ok(UciCommand::Go {
                depth: None,
                movetime: None,
                infinite: true
            })
        ));
    }

    #[test]
    fn test_parse_stop_command() {
        let result = UciEngine::parse_command("stop");
        assert!(matches!(result, Ok(UciCommand::Stop)));
    }

    #[test]
    fn test_parse_quit_command() {
        let result = UciEngine::parse_command("quit");
        assert!(matches!(result, Ok(UciCommand::Quit)));
    }

    #[test]
    fn test_handle_uci_command() {
        let mut engine = UciEngine::new().unwrap();
        let responses = engine.handle_command(UciCommand::Uci).unwrap();

        assert_eq!(responses.len(), 2);
        assert!(matches!(responses[0], UciResponse::Id { .. }));
        assert!(matches!(responses[1], UciResponse::UciOk));
    }

    #[test]
    fn test_handle_isready_command() {
        let mut engine = UciEngine::new().unwrap();
        let responses = engine.handle_command(UciCommand::IsReady).unwrap();

        assert_eq!(responses.len(), 1);
        assert!(matches!(responses[0], UciResponse::ReadyOk));
    }

    #[test]
    fn test_handle_position_startpos() {
        let mut engine = UciEngine::new().unwrap();
        let command = UciCommand::Position {
            fen: None,
            moves: vec![],
        };
        let responses = engine.handle_command(command).unwrap();

        // Should return empty response for position commands
        assert!(responses.is_empty());
        // Position should be starting position
        assert_eq!(
            engine.position.to_fen(),
            Position::starting_position().unwrap().to_fen()
        );
    }

    #[test]
    fn test_handle_position_with_moves() {
        let mut engine = UciEngine::new().unwrap();
        let command = UciCommand::Position {
            fen: None,
            moves: vec!["e2e4".to_string(), "e7e5".to_string()],
        };
        let responses = engine.handle_command(command).unwrap();

        assert!(responses.is_empty());
        // Position should have moves applied
        assert_ne!(
            engine.position.to_fen(),
            Position::starting_position().unwrap().to_fen()
        );
    }

    #[test]
    fn test_handle_go_depth() {
        let mut engine = UciEngine::new().unwrap();
        let command = UciCommand::Go {
            depth: Some(5),
            movetime: None,
            infinite: false,
        };
        let responses = engine.handle_command(command).unwrap();

        // Should return bestmove
        assert!(!responses.is_empty());
        assert!(
            responses
                .iter()
                .any(|r| matches!(r, UciResponse::BestMove { .. }))
        );
    }

    #[test]
    fn test_format_id_response() {
        let response = UciResponse::Id {
            name: "Chess Engine".to_string(),
            author: "Test Author".to_string(),
        };
        let formatted = UciEngine::format_response(&response);
        assert_eq!(formatted, "id name Chess Engine\nid author Test Author");
    }

    #[test]
    fn test_format_uciok_response() {
        let response = UciResponse::UciOk;
        let formatted = UciEngine::format_response(&response);
        assert_eq!(formatted, "uciok");
    }

    #[test]
    fn test_format_readyok_response() {
        let response = UciResponse::ReadyOk;
        let formatted = UciEngine::format_response(&response);
        assert_eq!(formatted, "readyok");
    }

    #[test]
    fn test_format_bestmove_response() {
        let response = UciResponse::BestMove {
            best_move: "e2e4".to_string(),
            ponder: Some("e7e5".to_string()),
        };
        let formatted = UciEngine::format_response(&response);
        assert_eq!(formatted, "bestmove e2e4 ponder e7e5");
    }

    #[test]
    fn test_format_bestmove_no_ponder() {
        let response = UciResponse::BestMove {
            best_move: "e2e4".to_string(),
            ponder: None,
        };
        let formatted = UciEngine::format_response(&response);
        assert_eq!(formatted, "bestmove e2e4");
    }

    #[test]
    fn test_format_info_response() {
        let response = UciResponse::Info {
            depth: 5,
            nodes: 12345,
            time: 1000,
            pv: vec!["e2e4".to_string(), "e7e5".to_string()],
        };
        let formatted = UciEngine::format_response(&response);
        assert_eq!(formatted, "info depth 5 nodes 12345 time 1000 pv e2e4 e7e5");
    }
}
