use clap::Parser;

mod weak_random;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Observed first output of 'Math.random()'
    first_random_output: f64,
    /// If provided, will output the next 'Math.random()' output to expect
    /// directly without outputting the seed.
    #[arg(long)]
    next_pred: bool,
}

fn main() {
    let args = Args::parse();

    println!("Todo: brute-force here... Args: {:?}", args);
}
