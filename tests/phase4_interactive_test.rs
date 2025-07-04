use chess_engine::tui::{GameMode, TuiApp, TuiState};
use chess_engine::{Color, Move, Square};

#[cfg(test)]
mod phase4_interactive_tests {
    use super::*;

    #[test]
    fn test_game_mode_management() {
        // RED: This test will fail because GameMode methods don't exist yet
        let mut app = TuiApp::new().unwrap();

        // Should start in Analysis mode
        assert_eq!(app.get_game_mode(), GameMode::Analysis);

        // Should be able to switch to PlayVsEngine mode
        app.set_game_mode(GameMode::PlayVsEngine {
            difficulty: 5,
            player_color: Color::White,
        });

        match app.get_game_mode() {
            GameMode::PlayVsEngine {
                difficulty,
                player_color,
            } => {
                assert_eq!(difficulty, 5);
                assert_eq!(player_color, Color::White);
            }
            _ => panic!("Should be in PlayVsEngine mode"),
        }

        // Should be able to switch to PuzzleSolving mode
        app.set_game_mode(GameMode::PuzzleSolving {
            puzzle_id: "mate_in_2_001".to_string(),
        });

        match app.get_game_mode() {
            GameMode::PuzzleSolving { puzzle_id } => {
                assert_eq!(puzzle_id, "mate_in_2_001");
            }
            _ => panic!("Should be in PuzzleSolving mode"),
        }
    }

    #[test]
    fn test_play_vs_engine_workflow() {
        // RED: This test will fail because PlayVsEngine functionality doesn't exist
        let mut app = TuiApp::new().unwrap();

        // Start a game vs engine
        app.start_engine_game(Color::White, 3);

        // Should be in PlayVsEngine mode
        match app.get_game_mode() {
            GameMode::PlayVsEngine {
                difficulty,
                player_color,
            } => {
                assert_eq!(difficulty, 3);
                assert_eq!(player_color, Color::White);
            }
            _ => panic!("Should be in PlayVsEngine mode"),
        }

        // Should be player's turn (White)
        assert_eq!(app.get_current_player_turn(), Color::White);

        // Make a move as white
        let from = Square::from_algebraic("e2").unwrap();
        let to = Square::from_algebraic("e4").unwrap();
        let player_move = Move::quiet(from, to);
        let result = app.make_player_move(player_move);
        assert!(result.is_ok());

        // Should now be engine's turn (Black)
        assert_eq!(app.get_current_player_turn(), Color::Black);

        // Engine should make a move automatically
        let engine_move = app.get_last_engine_move();
        assert!(engine_move.is_some());

        // Should be back to player's turn
        assert_eq!(app.get_current_player_turn(), Color::White);
    }

    #[test]
    fn test_puzzle_solving_workflow() {
        // RED: This test will fail because puzzle functionality doesn't exist
        let mut app = TuiApp::new().unwrap();

        // Load a tactical puzzle
        let result = app.load_puzzle("mate_in_2_001");
        assert!(result.is_ok());

        // Should be in PuzzleSolving mode
        match app.get_game_mode() {
            GameMode::PuzzleSolving { puzzle_id } => {
                assert_eq!(puzzle_id, "mate_in_2_001");
            }
            _ => panic!("Should be in PuzzleSolving mode"),
        }

        // Should have puzzle information
        let puzzle_info = app.get_puzzle_info();
        assert!(puzzle_info.is_some());
        assert_eq!(
            puzzle_info.unwrap().objective,
            "White to play and mate in 2"
        );

        // Should be able to attempt solution
        let from = Square::from_algebraic("d1").unwrap();
        let to = Square::from_algebraic("h5").unwrap();
        let attempt_move = Move::quiet(from, to);
        let solution_result = app.attempt_puzzle_solution(attempt_move);

        // Should validate the move and provide feedback
        // For this basic implementation, we expect it to be incorrect and provide feedback
        assert!(!solution_result.is_correct);
        assert!(!solution_result.feedback.is_empty());

        // Should be able to get hints
        let hint = app.get_puzzle_hint();
        assert!(hint.is_some());
        assert!(!hint.unwrap().is_empty());
    }

    #[test]
    fn test_threat_visualization() {
        // RED: This test will fail because threat detection doesn't exist
        let mut app = TuiApp::new().unwrap();

        // Set up a position with tactical threats
        let position_fen = "rnbqkb1r/pppp1ppp/5n2/4p3/2B1P3/8/PPPP1PPP/RNBQK1NR w KQkq - 2 3";
        app.set_position_from_fen(position_fen).unwrap();

        // Should detect threats in the position
        let threats = app.get_threats_for_position();
        assert!(!threats.is_empty());

        // Should identify the bishop attacking f7
        let bishop_threat = threats.iter().find(|t| {
            t.attacking_piece_square == "c4" && t.target_square == "f7" && t.threat_type == "Attack"
        });
        assert!(bishop_threat.is_some());

        // Should identify knight fork potential
        let _fork_threat = threats
            .iter()
            .find(|t| t.threat_type == "Fork" || t.threat_type == "Potential Fork");
        // This position may or may not have fork threats, just checking the system works

        // Should be able to visualize threats on board
        let threat_overlay = app.get_threat_overlay();
        assert!(!threat_overlay.is_empty());
    }

    #[test]
    fn test_game_state_persistence() {
        // RED: This test will fail because GameState doesn't exist
        let mut app = TuiApp::new().unwrap();

        // Start a game and make some moves
        app.start_engine_game(Color::White, 5);
        let from = Square::from_algebraic("e2").unwrap();
        let to = Square::from_algebraic("e4").unwrap();
        let move1 = Move::quiet(from, to);
        app.make_player_move(move1).unwrap();

        // Game state should persist across TUI state changes
        app.set_tui_state(TuiState::Board);
        assert_eq!(app.get_current_player_turn(), Color::Black);

        app.set_tui_state(TuiState::Command);
        assert_eq!(app.get_current_player_turn(), Color::Black);

        app.set_tui_state(TuiState::GamePlay);
        assert_eq!(app.get_current_player_turn(), Color::Black);

        // Move history should be maintained
        let history = app.get_move_history();
        assert_eq!(history.len(), 2); // Player move + engine response
        assert_eq!(history[0], move1);

        // Game clock should be tracking time
        let clock_times = app.get_game_clock();
        assert!(clock_times.is_some());
        let (white_time, black_time) = clock_times.unwrap();
        assert!(white_time > 0);
        assert!(black_time > 0);
    }

    #[test]
    fn test_interactive_move_input() {
        // RED: This test will fail because interactive move input doesn't exist
        let mut app = TuiApp::new().unwrap();
        app.start_engine_game(Color::White, 3);

        // Should validate legal moves
        let from = Square::from_algebraic("e2").unwrap();
        let to = Square::from_algebraic("e4").unwrap();
        let legal_move = Move::quiet(from, to);
        let result = app.validate_and_execute_move("e2e4");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), legal_move);

        // Should reject illegal moves
        let illegal_result = app.validate_and_execute_move("e2e5"); // Illegal pawn move
        assert!(illegal_result.is_err());
        assert!(illegal_result.unwrap_err().contains("Illegal move"));

        // Should handle ambiguous notation
        let ambiguous_result = app.validate_and_execute_move("Nf3");
        if ambiguous_result.is_ok() {
            // Should resolve ambiguity correctly
            let resolved_move = ambiguous_result.unwrap();
            assert!(
                resolved_move.to_string().contains("g1f3")
                    || resolved_move.to_string().contains("b1f3")
            );
        }

        // Should provide helpful error messages
        let invalid_result = app.validate_and_execute_move("invalid");
        assert!(invalid_result.is_err());
        assert!(invalid_result.unwrap_err().contains("Invalid move format"));
    }

    #[test]
    fn test_game_clock_management() {
        // RED: This test will fail because game clock doesn't exist
        let mut app = TuiApp::new().unwrap();

        // Should start without a clock in analysis mode
        assert!(app.get_game_clock().is_none());

        // Should create clock when starting a game
        app.start_engine_game(Color::White, 5);
        let initial_clock = app.get_game_clock();
        assert!(initial_clock.is_some());

        let (white_time, black_time) = initial_clock.unwrap();
        // Should start with reasonable default time (e.g., 5 minutes each)
        assert!(white_time >= 300000); // 5 minutes in ms
        assert!(black_time >= 300000);

        // Should tick down white's time when it's white's turn
        assert_eq!(app.get_current_player_turn(), Color::White);

        // Simulate time passing
        std::thread::sleep(std::time::Duration::from_millis(10));
        app.update_game_clock();

        let updated_clock = app.get_game_clock().unwrap();
        assert!(updated_clock.0 < white_time); // White time should decrease
        assert_eq!(updated_clock.1, black_time); // Black time unchanged

        // Should switch to black's clock after white moves
        let from = Square::from_algebraic("e2").unwrap();
        let to = Square::from_algebraic("e4").unwrap();
        let move1 = Move::quiet(from, to);
        app.make_player_move(move1).unwrap();

        // Now black's time should tick down
        assert_eq!(app.get_current_player_turn(), Color::Black);
    }
}
