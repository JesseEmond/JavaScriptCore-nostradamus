use std::process::ExitCode;
use clap::Parser;

mod weak_random;

use weak_random::WeakRandom;

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

fn bruteforce_seed(first_output_seen: f64) -> Option<WeakRandom> {
    for seed in 0..=(u32::MAX as u64) {
        let seed = seed as u32;
        let mut rng = WeakRandom::from_seed(seed);
        if rng.get() == first_output_seen {
            return Some(rng);
        }
    }
    None
}

fn main() -> ExitCode {
    let args = Args::parse();

    if let Some(mut rng) = bruteforce_seed(args.first_random_output) {
        if args.next_pred {
            println!("{}", rng.get());
        } else {
            println!("Seed: {}", rng.seed());
        }
        ExitCode::SUCCESS
    } else {
        eprintln!("No possible 32-bit seed yielded the output {}", args.first_random_output);
        ExitCode::FAILURE
    }
}
