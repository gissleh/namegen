use time::PreciseTime;
use rand::{SeedableRng, thread_rng};
use rand::rngs::SmallRng;
use std::fs::File;
use std::io::Read;
use namegen::NamePart;
use namegen::formatting::FormattingRule;
use namegen::core::{WorkingSet, Sample};

fn main() {
    let mut part = NamePart::new_markov(
        // Give the part a unique name.
        "first",

        // Formatting rules to apply to the finished name.
        &[
            FormattingRule::CapitalizeFirst,
            FormattingRule::CapitalizeAfter('\''),
        ],

        // These will be treated as if they were one letter by the generator,
        // but not the formatter. This is useful for clusters of consonants or
        // vowels that always appear together and should be generated together
        // as well.
        &["th", "ae", "ss", "nn"],

        // Length restrict start, middle, end, and token frequency.
        // These sacrifice potential variations for faithfulness.
        true, false, false, true
    );

    // Load samples from file. Providing it is an exercise to the reader.
    let mut file = File::open("./examples/res/markov.txt").unwrap();
    let mut data = String::with_capacity(2048);
    file.read_to_string(&mut data).unwrap();
    for line in data.lines().filter(|l| l.len() > 1) {
        if let Err(e) = part.learn(&Sample::Word(line.to_owned())) {
            eprintln!("{}", e);
        }
    }

    // A WorkingSet is a bundle of state used throughout the generator. This can be reused between
    // generator runs. It's there to save on expensive allocations.
    let mut ws = WorkingSet::new();

    // This isn't cryptography, so you can opt for speed over security.
    // If you're considering using this as part of a bigger procedural generation system,
    // this is where you would plug in an Rng trait impl.
    let mut rng = SmallRng::from_rng(thread_rng()).unwrap();

    // A little benchmark.
    // Expect 50% increase in time per name if you have several hundred samples.
    let start = PreciseTime::now();
    for _ in 0..100000 {
        part.generate(&mut ws, &mut rng);
    }
    let end = PreciseTime::now();

    // Here's the output.
    for n in 1..=70 {
        part.generate(&mut ws, &mut rng);

        print!("{result:<width$} ", result = ws.get_result(), width = 10);
        if n % 7 == 0 {
            print!("\n")
        }
    }

    println!("Benchmark: {}ns per name", start.to(end).num_nanoseconds().unwrap() / 100000);
}