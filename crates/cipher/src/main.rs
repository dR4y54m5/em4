use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "cipher")]
struct Cli {
    #[command(subcommand)]
    machine: Machine,
}

#[derive(Subcommand)]
enum Machine {
    Enigma {
        #[arg(long, value_delimiter = ',')]
        rotors: Vec<String>,
        #[arg(long)]
        reflector: String,
        #[arg(long)]
        rings: String,
        #[arg(long)]
        positions: String,
        #[arg(long, default_value = "")]
        plugboard: String,
        #[arg(long)]
        greek: Option<String>,
        #[arg(long)]
        text: Option<String>,
    },
}

fn main() -> std::process::ExitCode {
    match run(Cli::parse()) {
        Ok(out) => {
            println!("{out}");
            std::process::ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("error: {e:?}");
            std::process::ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> Result<String, enigma::EnigmaError> {
    match cli.machine {
        Machine::Enigma {
            rotors,
            reflector,
            rings,
            positions,
            plugboard,
            greek,
            text,
        } => {
            if rotors.len() != 3 {
                return Err(enigma::EnigmaError::BadPositions(format!(
                    "need 3 rotors, got {}",
                    rotors.len()
                )));
            }
            let input = text.unwrap_or_else(read_stdin);
            let r = [rotors[0].as_str(), rotors[1].as_str(), rotors[2].as_str()];
            let mut m = match &greek {
                Some(g) => enigma::Enigma::m4(g, r, &reflector, &rings, &positions, &plugboard)?,
                None => enigma::Enigma::m3(r, &reflector, &rings, &positions, &plugboard)?,
            };
            Ok(input.chars().map(|c| m.encode_char(c)).collect())
        }
    }
}

fn read_stdin() -> String {
    use std::io::Read;
    let mut s = String::new();
    std::io::stdin().read_to_string(&mut s).ok();
    s.trim().to_string()
}
