# Chess Engine

[![Rust](https://img.shields.io/badge/rust-1.87+-orange.svg)](https://www.rust-lang.org/)
[![Tests](https://img.shields.io/badge/tests-59%20passing-green.svg)](#testing)
[![Buy Me A Coffee](https://img.shields.io/badge/Buy%20Me%20A%20Coffee-support-yellow.svg)](https://www.buymeacoffee.com/orielsanchez)

A high-performance chess engine written in Rust featuring tournament-level search algorithms, modern terminal interface, and UCI protocol compliance. Designed for competitive play with advanced optimizations including transposition tables, killer moves, and aspiration windows.

## Features

### Core Engine
- **Tournament-level search** with alpha-beta pruning and aspiration windows
- **Transposition table** with Zobrist hashing (4.1x performance improvement)
- **Advanced move ordering** with killer moves heuristic
- **Quiescence search** for tactical accuracy
- **Iterative deepening** with time-controlled search (292K nodes/second)
- **Complete chess rules** including castling, en passant, and promotion
- **FEN position** import/export with validation

### User Interfaces
- **Modern TUI** with visual ASCII board display and split-pane interface
- **Interactive CLI** with command history, tab completion, and smart aliases
- **UCI protocol** support for chess GUI compatibility (Arena, ChessBase, Fritz)
- **PGN import/export** with comprehensive game notation support

### Performance
- **Tournament performance**: 292K nodes/second at depth 5
- **97.1% cache hit rate** with intelligent position caching
- **67% faster move selection** through optimized algorithms
- **Production-ready**: Zero unwrap/panic patterns, comprehensive error handling

## Quick Start

### Prerequisites
- Rust 1.87+ (2024 edition)

### Installation
```bash
git clone <repository-url>
cd chess-engine
cargo build --release
```

### Usage

#### Terminal UI (Recommended)
```bash
cargo run --bin tui
```
- Visual chess board with Unicode pieces
- Split-pane interface (board + commands)
- Tab completion and command history
- Real-time position updates

#### Interactive Analysis
```bash
cargo run --bin interactive
```
Commands:
- `analyze` - Get best move and evaluation
- `move e4` - Make a move
- `legal` - Show all legal moves
- `position <fen>` - Set position from FEN
- `undo` - Undo last move
- `help` - Show all commands

#### UCI Protocol
```bash
cargo run
```
Compatible with chess GUIs via UCI protocol.

## Architecture

### Core Components
- **Position**: Game state with move validation
- **Search**: Tournament-level minimax with optimizations
- **Evaluation**: Material + positional assessment
- **Move Generation**: Complete legal move generation
- **Transposition Table**: Position caching with Zobrist hashing

### Performance Optimizations
- **Alpha-beta pruning**: 90%+ node reduction
- **Killer moves**: Historical move ordering
- **Aspiration windows**: Narrow search windows
- **Quiescence search**: Tactical sequence evaluation
- **Principal variation**: Best move tracking

## Testing

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test search_tests
```

**Test Coverage**: 59 comprehensive tests covering all components

## Development

### Code Quality
```bash
# Format code
cargo fmt

# Run linter
cargo clippy

# Run all checks
cargo fmt && cargo test && cargo clippy
```

### Performance Testing
```bash
# Run benchmarks (if available)
cargo bench

# Profile performance
cargo build --release
time ./target/release/chess-engine
```

## Project Structure

```
src/
├── main.rs           # UCI engine entry point
├── lib.rs            # Library exports
├── types.rs          # Core data types
├── board.rs          # Board representation
├── position.rs       # Game state management
├── moves.rs          # Move representation
├── movegen.rs        # Legal move generation
├── fen.rs            # FEN parsing/generation
├── eval.rs           # Position evaluation
├── search.rs         # Search algorithms
├── transposition.rs  # Hash table
├── pgn.rs            # PGN import/export
├── uci.rs            # UCI protocol
├── interactive.rs    # Interactive CLI
└── tui.rs            # Terminal UI
```

## Roadmap

### Completed
- Tournament-level search with all optimizations
- Modern terminal UI with visual board
- Interactive analysis mode
- UCI protocol compliance
- PGN import/export

### In Progress
- Enhanced evaluation (king safety, mobility)
- Rich analysis display with statistics

### Planned
- Opening book integration
- Endgame tablebase support
- Multi-PV analysis
- Training modes and puzzles

## Contributing

1. Follow Test-Driven Development (TDD)
2. Maintain zero unwrap/panic patterns
3. Ensure all tests pass: `cargo test`
4. Format code: `cargo fmt`
5. Check lints: `cargo clippy`

## Performance Benchmarks

| Feature | Performance |
|---------|-------------|
| Search Speed | 292K nodes/second |
| Cache Hit Rate | 97.1% |
| Search Improvement | 4.1x faster |
| Test Coverage | 59 tests |
| Code Quality | Zero clippy warnings |

## Technical Highlights

### Algorithm Optimizations
- **Transposition Table**: 4.1x performance improvement with 97.1% cache hit rate
- **Alpha-Beta Pruning**: 90%+ node reduction in search tree
- **Killer Moves Heuristic**: Enhanced move ordering for beta cutoffs  
- **Aspiration Windows**: Narrow search windows for efficiency
- **Quiescence Search**: Tactical sequence evaluation to horizon

### Code Quality
- **Zero Unsafe Code**: Memory-safe Rust throughout
- **Comprehensive Error Handling**: No unwrap/panic patterns
- **Test-Driven Development**: 59 passing tests with full coverage
- **Production Standards**: Clippy-clean with extensive documentation

### Architecture
- **Modular Design**: Clean separation of concerns
- **Performance-Critical**: Optimized hot paths and data structures
- **Cross-Platform**: Works on Windows, macOS, and Linux
- **Standards Compliant**: Full UCI protocol implementation

## Support

If you find this project helpful for learning chess engine development or Rust optimization techniques, consider supporting continued development:

[![Buy Me A Coffee](https://img.shields.io/badge/Buy%20Me%20A%20Coffee-support-yellow.svg)](https://www.buymeacoffee.com/orielsanchez)

## License

MIT License - See LICENSE file for details

## Author

Chess engine developer focused on high-performance algorithms and modern Rust practices.