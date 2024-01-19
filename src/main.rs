use clap::Parser;

#[derive(Debug, Parser)]
#[command(author, version, about)]
struct Args {}

fn main() {
    let args = Args::parse();
    println!("{:?}", args);
}
