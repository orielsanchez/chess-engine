use chess_engine::interactive::InteractiveEngine;
use log::{debug, error, info, warn};
use std::io::{self, BufRead, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    info!("Chess Engine Interactive Mode Starting");
    debug!("Logger initialized");
    let mut engine = InteractiveEngine::new()?;
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    println!("Chess Engine - Interactive Analysis Mode");
    println!("Type 'help' for available commands, 'quit' to exit");
    print!("> ");
    stdout.flush()?;

    for line in stdin.lock().lines() {
        let input = line?;
        let input = input.trim();

        if input.is_empty() {
            print!("> ");
            stdout.flush()?;
            continue;
        }

        if input == "quit" || input == "exit" {
            println!("Goodbye!");
            break;
        }

        let command = match InteractiveEngine::parse_command(input) {
            Ok(cmd) => {
                debug!("Parsed command: {:?}", cmd);
                cmd
            }
            Err(e) => {
                warn!("Failed to parse command '{}': {}", input, e);
                eprintln!("Error: {}", e);
                print!("> ");
                stdout.flush()?;
                continue;
            }
        };

        info!("Processing command: {:?}", command);
        match engine.handle_command(command) {
            Ok(response) => {
                debug!("Command successful: {:?}", response);
                let output = InteractiveEngine::format_response(&response);
                println!("{}", output);
            }
            Err(e) => {
                error!("Command failed: {}", e);
                eprintln!("Error: {}", e);
            }
        }

        print!("> ");
        stdout.flush()?;
    }

    Ok(())
}
