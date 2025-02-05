use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[arg(short, long)]
    pub port: Option<u16>,

    #[arg(short, long)]
    pub dump_path: Option<String>,

    #[arg(short, long)]
    pub bitmap_path: Option<String>
}
