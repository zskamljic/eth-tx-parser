use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version)]
pub struct Args {
    #[arg(short, long)]
    pub estimate: bool,
    pub transaction: String,
}
