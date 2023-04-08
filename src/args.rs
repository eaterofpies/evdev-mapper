use clap::{Parser, ValueEnum};

#[derive(Debug, Clone, ValueEnum)]
pub enum Mode {
    Devices,
    Properties,
    Run,
}

/// Combine multiple input devices into a single virtual device.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// mode to start in. Default is run.
    #[arg(short, long)]
    pub mode: Option<Mode>,

    /// Device (required in properties mode)
    #[arg(short, long)]
    pub device: Option<String>,
}
