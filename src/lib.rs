//! Chess engine library providing core functionality for chess game analysis.
//!
//! This library includes modules for board representation, move generation, position
//! evaluation, search algorithms, and various chess-specific utilities.

pub mod benchmark;
pub mod board;
pub mod distance_to_mate;
pub mod eval;
pub mod fen;
pub mod interactive;
pub mod movegen;
pub mod moves;
pub mod pgn;
pub mod position;
pub mod search;
pub mod tablebase;
pub mod transposition;
pub mod tui;
pub mod types;
pub mod uci;

pub use board::*;
pub use fen::*;
pub use moves::*;
pub use position::*;
pub use transposition::*;
pub use types::*;
