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
    /// Program mode mode to start in.
    #[arg(short, long, default_value = "run")]
    pub mode: Mode,

    /// Device (required in properties mode)
    #[arg(short, long)]
    pub device: Option<String>,

    /// Config file to run with
    #[arg(short, long, default_value = "device.conf")]
    pub config: String,
}
