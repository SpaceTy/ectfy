use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "ectfy")]
#[command(about = "Encrypt and decrypt files using AES-256-GCM")]
#[command(version)]
pub struct Cli {
    #[arg(help = "Path to file or folder to encrypt/decrypt")]
    pub path: Option<PathBuf>,
    
    #[arg(short = 's', long = "show-password", help = "Show password as it's being entered")]
    pub show_password: bool,
}

impl Cli {
    pub fn parse_args() -> Self {
        Self::parse()
    }
}

