use clap::Parser;

/// Combine multiple input devices into a single virtual device
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// mode to run in [devices, properties, run]
    #[arg(short, long)]
    pub mode: Option<String>,

    /// Device (required in properties mode)
    #[arg(short, long)]
    pub device: Option<String>,
}
