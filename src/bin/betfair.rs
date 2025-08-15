use clap::Parser;

#[derive(Parser)]
#[command(name = "betfair")]
#[command(about = "Betfair trading CLI", long_about = None)]
struct Cli {
    /// The mode to run (e.g., dashboard)
    mode: String,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    
    match cli.mode.as_str() {
        "dashboard" => {
            // Include and run the dashboard code directly
            dashboard::run()
        }
        _ => {
            eprintln!("Unknown mode: {}", cli.mode);
            eprintln!("Available modes: dashboard");
            std::process::exit(1);
        }
    }
}

// Include the dashboard module
mod dashboard {
    include!("../../examples/dashboard.rs");
    
    pub fn run() -> anyhow::Result<()> {
        main()
    }
}