use chess_engine::position::Position;
use chess_engine::tablebase::{MockTablebase, Tablebase, TablebaseError, TablebaseResult};

/// Comprehensive test suite for real Syzygy tablebase support
///
/// These tests define the behavior we want for production Syzygy tablebase integration:
/// 1. Loading real .rtbw and .rtbz files from disk
/// 2. Actual endgame position lookups with perfect results  
/// 3. DTM (Distance to Mate) and DTZ (Distance to Zeroing) support
/// 4. Performance characteristics superior to mock implementation
/// 5. File management and error handling
///
/// INITIAL STATE: All tests will FAIL - this defines our implementation targets

#[cfg(test)]
mod syzygy_tests {
    use super::*;

    #[test]
    fn test_syzygy_tablebase_creation_with_path() {
        // RED: Test that we can create a Syzygy tablebase pointed at a directory
        let tablebase_path = "/tmp/syzygy_test";
        std::fs::create_dir_all(tablebase_path).unwrap();

        // Create at least one tablebase file so is_initialized() returns true
        create_uncompressed_syzygy_file(&format!("{}/KQvK.rtbw", tablebase_path));

        let syzygy = SyzygyTablebase::new(tablebase_path).unwrap();

        assert_eq!(syzygy.tablebase_path(), tablebase_path);
        assert!(syzygy.is_initialized());

        // Cleanup
        std::fs::remove_dir_all(tablebase_path).ok();
    }

    #[test]
    fn test_syzygy_file_discovery() {
        // RED: Test automatic discovery of .rtbw and .rtbz files
        let tablebase_path = "/tmp/syzygy_test";
        std::fs::create_dir_all(tablebase_path).ok();

        // Create mock tablebase files for testing
        create_uncompressed_syzygy_file(&format!("{}/KQvK.rtbw", tablebase_path));
        create_uncompressed_syzygy_file(&format!("{}/KRvK.rtbw", tablebase_path));
        create_uncompressed_syzygy_file(&format!("{}/KPvK.rtbz", tablebase_path));

        let syzygy = SyzygyTablebase::new(tablebase_path).unwrap();

        // Should discover all available tablebase files
        assert!(syzygy.is_available("KQvK"));
        assert!(syzygy.is_available("KRvK"));
        assert!(syzygy.is_available("KPvK"));
        assert!(!syzygy.is_available("KNvK")); // Not present

        // Cleanup
        std::fs::remove_dir_all(tablebase_path).ok();
    }

    #[test]
    fn test_syzygy_real_kqvk_lookup() {
        // RED: Test real Syzygy lookup for King + Queen vs King position
        let position = Position::from_fen("8/8/8/8/8/2k5/2Q5/2K5 w - - 0 1").unwrap();
        let syzygy = create_test_syzygy_tablebase();

        let result = syzygy.probe(&position).unwrap();

        match result {
            TablebaseResult::Win(dtm) => {
                // Real Syzygy should give precise DTM, not just hardcoded 10
                assert!(dtm <= 10); // Should be mate in 10 or fewer
                assert!(dtm > 0); // But not immediate mate
            }
            _ => panic!("KQvK position should be winning for side with Queen"),
        }
    }

    #[test]
    fn test_syzygy_dtm_vs_dtz_distinction() {
        // RED: Test that Syzygy can provide both DTM and DTZ results
        let position = Position::from_fen("8/8/8/8/3k4/8/3PK3/8 w - - 0 1").unwrap();
        let syzygy = create_test_syzygy_tablebase();

        // Should be able to query both distance-to-mate and distance-to-zeroing
        let dtm_result = syzygy.probe_dtm(&position).unwrap();
        let dtz_result = syzygy.probe_dtz(&position).unwrap();

        // DTM and DTZ can be different (DTZ considers 50-move rule)
        match (dtm_result, dtz_result) {
            (TablebaseResult::Win(dtm), TablebaseResult::Win(dtz)) => {
                // DTZ should be >= DTM due to 50-move rule considerations
                assert!(dtz >= dtm, "DTZ should be >= DTM for winning positions");
            }
            _ => {} // Other combinations are possible
        }
    }

    #[test]
    fn test_syzygy_performance_vs_mock() {
        // RED: Test that Syzygy lookup is faster than mock for real positions
        let position = Position::from_fen("8/8/8/8/8/2k5/2Q5/2K5 w - - 0 1").unwrap();

        let syzygy = create_test_syzygy_tablebase();
        let mock = MockTablebase::new();

        // Time Syzygy lookup
        let syzygy_start = std::time::Instant::now();
        for _ in 0..1000 {
            let _ = syzygy.probe(&position).unwrap();
        }
        let syzygy_time = syzygy_start.elapsed();

        // Time mock lookup
        let mock_start = std::time::Instant::now();
        for _ in 0..1000 {
            let _ = mock.probe(&position).unwrap();
        }
        let mock_time = mock_start.elapsed();

        // Syzygy should be comparable or faster despite disk access
        // (due to efficient binary format and caching)
        assert!(
            syzygy_time.as_nanos() < mock_time.as_nanos() * 2,
            "Syzygy performance should be competitive with mock"
        );
    }

    #[test]
    fn test_syzygy_position_normalization() {
        // RED: Test that Syzygy handles position normalization correctly
        // Same position with different orientations should give consistent results

        let pos1 = Position::from_fen("8/8/8/8/8/2k5/2Q5/2K5 w - - 0 1").unwrap();
        let pos2 = Position::from_fen("3K4/3Q4/3k4/8/8/8/8/8 b - - 0 1").unwrap(); // Equivalent but shifted

        let syzygy = create_test_syzygy_tablebase();

        let result1 = syzygy.probe(&pos1).unwrap();
        let result2 = syzygy.probe(&pos2).unwrap();

        // Results should be equivalent (both winning, similar DTM)
        match (result1, result2) {
            (TablebaseResult::Win(dtm1), TablebaseResult::Win(dtm2)) => {
                // DTM should be very close (within 1-2 moves due to normalization)
                assert!((dtm1 as i32 - dtm2 as i32).abs() <= 2);
            }
            (TablebaseResult::Loss(dtm1), TablebaseResult::Loss(dtm2)) => {
                assert!((dtm1 as i32 - dtm2 as i32).abs() <= 2);
            }
            _ => panic!("Equivalent positions should have equivalent results"),
        }
    }

    #[test]
    fn test_syzygy_error_handling() {
        // RED: Test error handling for missing files and invalid positions

        // Test missing tablebase directory
        let result = SyzygyTablebase::new("/nonexistent/path");
        assert!(result.is_err());

        // Test position not in tablebase (too many pieces)
        let opening =
            Position::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
        let syzygy = create_test_syzygy_tablebase();

        let result = syzygy.probe(&opening);
        assert_eq!(result, Err(TablebaseError::NotFound));
    }

    #[test]
    fn test_syzygy_memory_management() {
        // RED: Test that Syzygy tablebase manages memory efficiently
        let syzygy = create_test_syzygy_tablebase();

        // Should start with minimal memory usage
        let initial_loaded = syzygy.loaded_tablebase_count();
        assert_eq!(initial_loaded, 0);

        // After probing, should load specific tablebase
        let position = Position::from_fen("8/8/8/8/8/2k5/2Q5/2K5 w - - 0 1").unwrap();
        let _ = syzygy.probe(&position).unwrap();

        let after_probe = syzygy.loaded_tablebase_count();
        assert!(after_probe > initial_loaded);

        // Should be able to unload to free memory
        syzygy.unload_all();
        let after_unload = syzygy.loaded_tablebase_count();
        assert_eq!(after_unload, 0);
    }

    #[test]
    fn test_syzygy_thread_safety() {
        // RED: Test that Syzygy tablebase is thread-safe for concurrent lookups
        use std::sync::Arc;
        use std::thread;

        let syzygy = Arc::new(create_test_syzygy_tablebase());
        let position = Arc::new(Position::from_fen("8/8/8/8/8/2k5/2Q5/2K5 w - - 0 1").unwrap());

        let mut handles = vec![];

        // Spawn multiple threads doing concurrent lookups
        for _ in 0..4 {
            let syzygy_clone = Arc::clone(&syzygy);
            let position_clone = Arc::clone(&position);

            let handle = thread::spawn(move || {
                for _ in 0..100 {
                    let result = syzygy_clone.probe(&*position_clone);
                    assert!(result.is_ok());
                }
            });

            handles.push(handle);
        }

        // All threads should complete successfully
        for handle in handles {
            handle.join().unwrap();
        }
    }

    #[test]
    fn test_syzygy_integration_with_search() {
        // RED: Test that search engine can use Syzygy tablebase for optimization
        let mut position = Position::from_fen("8/8/8/8/8/2k5/1Q6/2K5 w - - 0 1").unwrap();

        // Create position with Syzygy tablebase enabled
        position.set_tablebase(Box::new(create_test_syzygy_tablebase()));

        let search_result = position.find_best_move_with_tablebase(1000);

        // Should find optimal move quickly due to perfect tablebase knowledge
        assert!(search_result.best_move.is_some());
        assert!(search_result.score > 15000); // Strong winning score
        assert!(search_result.depth >= 1);

        // Should indicate tablebase was used
        assert!(search_result.tablebase_hits > 0);
    }

    #[test]
    fn test_real_syzygy_kqk_wdl_parsing() {
        // RED: Test real Syzygy WDL file parsing for KQK endgame
        let test_path = "/tmp/syzygy_real_test";
        std::fs::create_dir_all(test_path).unwrap();

        create_minimal_kqk_wdl_file(&format!("{}/KQvK.rtbw", test_path));

        let syzygy = SyzygyTablebase::new(test_path).unwrap();

        // Test known winning KQK position
        let winning_pos = Position::from_fen("4k3/8/8/8/4Q3/8/4K3/8 w - - 0 1").unwrap();
        let result = syzygy.probe(&winning_pos).unwrap();

        // Should return Win (WDL-only, ignore DTM for now)
        assert!(
            matches!(result, TablebaseResult::Win(_)),
            "KQK position should be a win"
        );

        std::fs::remove_dir_all(test_path).ok();
    }

    // Helper function to create test Syzygy tablebase
    // This will fail initially until we implement SyzygyTablebase
    fn create_test_syzygy_tablebase() -> SyzygyTablebase {
        let test_path = "/tmp/syzygy_test_data";
        std::fs::create_dir_all(test_path).unwrap();

        // Create minimal test tablebase files
        create_test_tablebase_files(test_path);

        SyzygyTablebase::new(test_path).unwrap()
    }

    fn create_test_tablebase_files(path: &str) {
        // Create mock .rtbw and .rtbz files with minimal valid structure
        // These will be replaced with real Syzygy file format later
        create_uncompressed_syzygy_file(&format!("{}/KQvK.rtbw", path));
        create_uncompressed_syzygy_file(&format!("{}/KRvK.rtbw", path));
        create_uncompressed_syzygy_file(&format!("{}/KPvK.rtbz", path));
    }

    fn create_minimal_kqk_wdl_file(file_path: &str) {
        let mut data = Vec::new();

        // --- Header (32 bytes total, all little-endian) ---

        // Magic Number (4 bytes)
        data.extend_from_slice(&0x5d23e871u32.to_le_bytes());

        // Number of blocks (4 bytes). Must be 0 for uncompressed files.
        data.extend_from_slice(&0u32.to_le_bytes());

        // Info bitfield (4 bytes). We can use 0 for now.
        // The parser will eventually need to decode this, but for the first test, it can just skip it.
        data.extend_from_slice(&0u32.to_le_bytes());

        // Reserved field (4 bytes): placeholder
        data.extend_from_slice(&0u32.to_le_bytes());

        // Size for side 1 (e.g., White) (8 bytes). Let's define 4 positions.
        let num_positions_white: u64 = 4;
        data.extend_from_slice(&num_positions_white.to_le_bytes());

        // Size for side 2 (e.g., Black) (8 bytes). Let's also define 4 positions.
        let num_positions_black: u64 = 4;
        data.extend_from_slice(&num_positions_black.to_le_bytes());

        // --- WDL Data Payload (starts at byte 32) ---
        // Total positions = 4 (white) + 4 (black) = 8.
        // Each position is 2 bits, so we need 8 * 2 = 16 bits = 2 bytes of data.

        // Byte 1: WDL data for the 4 white-to-move positions.
        // Let's make them all 'Win' (value=2, binary=10).
        // The four results are packed: 10 10 10 10
        data.push(0b10101010); // This is 0xAA

        // Byte 2: WDL data for the 4 black-to-move positions.
        // Let's make them all 'Draw' (value=1, binary=01).
        // The four results are packed: 01 01 01 01
        data.push(0b01010101); // This is 0x55

        std::fs::write(file_path, data).unwrap();
    }

    #[test]
    fn test_compressed_syzygy_file_detection() {
        // RED: Test that we can detect compressed Syzygy files (nblocks > 0)
        let tablebase_path = "/tmp/syzygy_compressed_test";
        std::fs::create_dir_all(tablebase_path).unwrap();

        // Create a compressed tablebase file with nblocks > 0
        let file_path = format!("{}/KQvK.rtbw", tablebase_path);
        create_compressed_syzygy_file(&file_path);

        let syzygy = SyzygyTablebase::new(tablebase_path).unwrap();
        let position = Position::from_fen("8/8/8/8/8/2k5/2Q5/2K5 w - - 0 1").unwrap();

        // Should now support compressed files instead of returning error
        let result = syzygy.probe(&position);
        assert!(
            result.is_ok(),
            "Compressed file support should work: {:?}",
            result.err()
        );

        // Cleanup
        std::fs::remove_dir_all(tablebase_path).ok();
    }

    #[test]
    fn test_compressed_file_block_parsing() {
        // RED: Test that we correctly parse the block structure in compressed files
        let tablebase_path = "/tmp/syzygy_block_test";
        std::fs::create_dir_all(tablebase_path).unwrap();

        let file_path = format!("{}/KQvK.rtbw", tablebase_path);
        create_compressed_syzygy_file(&file_path);

        let syzygy = SyzygyTablebase::new(tablebase_path).unwrap();
        let position = Position::from_fen("8/8/8/8/8/2k5/2Q5/2K5 w - - 0 1").unwrap();

        // Should parse blocks and return valid result
        match syzygy.probe(&position).unwrap() {
            TablebaseResult::Win(_) | TablebaseResult::Loss(_) | TablebaseResult::Draw => {
                // Any valid result indicates successful block parsing
            }
        }

        // Cleanup
        std::fs::remove_dir_all(tablebase_path).ok();
    }

    #[test]
    fn test_block_index_navigation() {
        // RED: Test that we can navigate the block index table correctly
        let tablebase_path = "/tmp/syzygy_index_test";
        std::fs::create_dir_all(tablebase_path).unwrap();

        let file_path = format!("{}/KQvK.rtbw", tablebase_path);
        create_multi_block_syzygy_file(&file_path);

        let syzygy = SyzygyTablebase::new(tablebase_path).unwrap();

        // Test multiple positions to ensure we navigate different blocks
        let positions = [
            "8/8/8/8/8/2k5/2Q5/2K5 w - - 0 1", // First block
            "8/8/8/8/8/1k6/1Q6/1K6 w - - 0 1", // Different block
        ];

        for fen in &positions {
            let position = Position::from_fen(fen).unwrap();
            let result = syzygy.probe(&position);
            assert!(
                result.is_ok(),
                "Block navigation should work for position {}",
                fen
            );
        }

        // Cleanup
        std::fs::remove_dir_all(tablebase_path).ok();
    }

    #[test]
    fn test_decompression_algorithm() {
        // RED: Test that we correctly implement decompression (e.g., RE-PAIR)
        let tablebase_path = "/tmp/syzygy_decompress_test";
        std::fs::create_dir_all(tablebase_path).unwrap();

        let file_path = format!("{}/KQvK.rtbw", tablebase_path);
        create_compressed_syzygy_file_with_compression(&file_path);

        let syzygy = SyzygyTablebase::new(tablebase_path).unwrap();
        let position = Position::from_fen("8/8/8/8/8/2k5/2Q5/2K5 w - - 0 1").unwrap();

        // Should decompress correctly and return valid result
        let result = syzygy.probe(&position).unwrap();

        // Verify we get the expected result from the decompressed data
        match result {
            TablebaseResult::Win(dtm) => {
                assert!(dtm > 0, "DTM should be positive for winning position")
            }
            TablebaseResult::Loss(dtm) => {
                assert!(dtm > 0, "DTM should be positive for losing position")
            }
            TablebaseResult::Draw => {} // Draw is valid
        }

        // Cleanup
        std::fs::remove_dir_all(tablebase_path).ok();
    }

    #[test]
    fn test_compressed_vs_uncompressed_consistency() {
        // RED: Test that compressed and uncompressed files give same results
        let tablebase_path = "/tmp/syzygy_consistency_test";
        std::fs::create_dir_all(tablebase_path).unwrap();

        // Create compressed version only (the tablebase will find it by material signature)
        let compressed_path = format!("{}/KQvK.rtbw", tablebase_path);
        create_compressed_syzygy_file(&compressed_path);

        let syzygy = SyzygyTablebase::new(tablebase_path).unwrap();
        let position = Position::from_fen("8/8/8/8/8/2k5/2Q5/2K5 w - - 0 1").unwrap();

        // Both should give consistent results
        // Note: We'll need to adapt the test to handle different file names
        // For now, just test that compressed files work
        let result = syzygy.probe(&position);
        assert!(
            result.is_ok(),
            "Compressed file should give valid result: {:?}",
            result.err()
        );

        // Cleanup
        std::fs::remove_dir_all(tablebase_path).ok();
    }

    #[test]
    fn test_performance_with_compression() {
        // RED: Test that compressed file performance is acceptable
        let tablebase_path = "/tmp/syzygy_perf_test";
        std::fs::create_dir_all(tablebase_path).unwrap();

        let file_path = format!("{}/KQvK.rtbw", tablebase_path);
        create_compressed_syzygy_file(&file_path);

        let syzygy = SyzygyTablebase::new(tablebase_path).unwrap();
        let position = Position::from_fen("8/8/8/8/8/2k5/2Q5/2K5 w - - 0 1").unwrap();

        // Time multiple lookups to ensure decompression is efficient
        let start = std::time::Instant::now();
        for _ in 0..100 {
            let _ = syzygy.probe(&position).unwrap();
        }
        let elapsed = start.elapsed();

        // Should complete 100 lookups in reasonable time (< 1 second)
        assert!(
            elapsed.as_millis() < 1000,
            "Compressed file lookups should be fast"
        );

        // Cleanup
        std::fs::remove_dir_all(tablebase_path).ok();
    }

    // Helper functions for creating test files

    /// Create a minimal uncompressed Syzygy file for testing
    fn create_uncompressed_syzygy_file(file_path: &str) {
        let mut data = Vec::new();

        // Magic number (4 bytes, little-endian): 0x5d23e871
        data.extend_from_slice(&0x5d23e871u32.to_le_bytes());

        // Number of blocks (4 bytes, little-endian): 0 (indicates uncompressed)
        data.extend_from_slice(&0u32.to_le_bytes());

        // Info field (4 bytes): placeholder
        data.extend_from_slice(&0u32.to_le_bytes());

        // Reserved field (4 bytes): placeholder
        data.extend_from_slice(&0u32.to_le_bytes());

        // Size for side 1 (8 bytes)
        data.extend_from_slice(&4u64.to_le_bytes());

        // Size for side 2 (8 bytes)
        data.extend_from_slice(&4u64.to_le_bytes());

        // WDL data: minimal mock data (starting at byte 32)
        data.extend_from_slice(&[0x01, 0x02, 0x00, 0x01]); // Win, Loss, Draw, Win

        std::fs::write(file_path, data).unwrap();
    }

    /// Create a minimal compressed Syzygy file for testing
    fn create_compressed_syzygy_file(file_path: &str) {
        let mut data = Vec::new();

        // Magic number (4 bytes, little-endian): 0x5d23e871
        data.extend_from_slice(&0x5d23e871u32.to_le_bytes());

        // Number of blocks (4 bytes, little-endian): 2 (indicates compressed)
        data.extend_from_slice(&2u32.to_le_bytes());

        // Info field (4 bytes): placeholder
        data.extend_from_slice(&0u32.to_le_bytes());

        // Reserved field (4 bytes): placeholder
        data.extend_from_slice(&0u32.to_le_bytes());

        // Size for side 1 (8 bytes)
        data.extend_from_slice(&4u64.to_le_bytes());

        // Size for side 2 (8 bytes)
        data.extend_from_slice(&4u64.to_le_bytes());

        // Block index table (simplified)
        // Each block entry: offset (8 bytes) + size (4 bytes)
        // Header is 32 bytes, index table is 2 blocks * 12 bytes = 24 bytes
        // So first block starts at 32 + 24 = 56
        data.extend_from_slice(&56u64.to_le_bytes()); // Block 1 offset
        data.extend_from_slice(&32u32.to_le_bytes()); // Block 1 size
        data.extend_from_slice(&88u64.to_le_bytes()); // Block 2 offset (56 + 32)
        data.extend_from_slice(&32u32.to_le_bytes()); // Block 2 size

        // Compressed block data (simplified mock)
        data.extend_from_slice(&vec![0xAA; 32]); // Block 1 data
        data.extend_from_slice(&vec![0x55; 32]); // Block 2 data

        std::fs::write(file_path, data).unwrap();
    }

    /// Create a multi-block compressed file for testing block navigation
    fn create_multi_block_syzygy_file(file_path: &str) {
        let mut data = Vec::new();

        // Magic number
        data.extend_from_slice(&0x5d23e871u32.to_le_bytes());

        // Number of blocks: 4 (multiple blocks)
        data.extend_from_slice(&4u32.to_le_bytes());

        // Info and reserved fields
        data.extend_from_slice(&0u32.to_le_bytes());
        data.extend_from_slice(&0u32.to_le_bytes());

        // Position counts
        data.extend_from_slice(&8u64.to_le_bytes());
        data.extend_from_slice(&8u64.to_le_bytes());

        // Block index table (4 blocks)
        for i in 0..4 {
            let offset = 64 + i * 32; // Start after header + index
            data.extend_from_slice(&(offset as u64).to_le_bytes());
            data.extend_from_slice(&32u32.to_le_bytes());
        }

        // Block data
        for _ in 0..4 {
            data.extend_from_slice(&vec![0xCC; 32]);
        }

        std::fs::write(file_path, data).unwrap();
    }

    /// Create a compressed file with actual compression markers for testing decompression
    fn create_compressed_syzygy_file_with_compression(file_path: &str) {
        let mut data = Vec::new();

        // Standard header (32 bytes)
        data.extend_from_slice(&0x5d23e871u32.to_le_bytes());
        data.extend_from_slice(&1u32.to_le_bytes()); // 1 block
        data.extend_from_slice(&0u32.to_le_bytes());
        data.extend_from_slice(&0u32.to_le_bytes());
        data.extend_from_slice(&4u64.to_le_bytes());
        data.extend_from_slice(&4u64.to_le_bytes());

        // Block index (1 block × 12 bytes = 12 bytes)
        // Block data starts at: 32 (header) + 12 (index) = 44
        data.extend_from_slice(&44u64.to_le_bytes()); // Offset after index
        data.extend_from_slice(&16u32.to_le_bytes()); // Compressed size

        // Mock compressed data (16 bytes as specified above)
        data.extend_from_slice(&[
            0xFF, 0x00, 0xAA, 0x55, 0x33, 0xCC, 0x0F, 0xF0, 0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC,
            0xDE, 0xF0,
        ]);

        std::fs::write(file_path, data).unwrap();
    }
}

// Forward declarations for types that don't exist yet
// These will fail compilation until we implement them (RED phase)
use chess_engine::tablebase::syzygy::SyzygyTablebase;
