use chess_engine::uci::UciEngine;
use std::io::{self, BufRead, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut engine = UciEngine::new()?;
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    for line in stdin.lock().lines() {
        let input = line?;
        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        let command = match UciEngine::parse_command(input) {
            Ok(cmd) => cmd,
            Err(e) => {
                eprintln!("Error parsing command: {}", e);
                continue;
            }
        };

        let should_quit = matches!(command, chess_engine::uci::UciCommand::Quit);

        match engine.handle_command(command) {
            Ok(responses) => {
                for response in responses {
                    let output = UciEngine::format_response(&response);
                    println!("{}", output);
                    stdout.flush()?;
                }
            }
            Err(e) => {
                eprintln!("Error handling command: {}", e);
            }
        }

        if should_quit {
            break;
        }
    }

    Ok(())
}
