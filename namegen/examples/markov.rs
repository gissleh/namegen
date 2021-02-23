use time::PreciseTime;
use rand::{SeedableRng, thread_rng};
use rand::rngs::SmallRng;
use std::fs::File;
use std::io::Read;
use namegen::{NamePart, FormattingRule, SampleSet, Sample, WorkingSet};
use std::collections::BTreeSet;

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
        false, false, true, true
    );

    // Load samples from file. Providing it is an exercise to the reader.
    let mut file = File::open("./examples/res/markov.txt").unwrap();
    let mut data = String::with_capacity(2048);
    file.read_to_string(&mut data).unwrap();
    for line in data.lines().filter(|l| l.len() > 1) {
        let mut sample_set = SampleSet::new(&[]);
        sample_set.add_sample(Sample::Word(line.to_owned().to_lowercase()));

        if let Err(e) = part.learn(&sample_set) {
            eprintln!("{}", e);
        }
    }

    // Show the structure
    #[cfg(feature = "serde")]
        let part: NamePart = {
        let json =  serde_json::to_string(&part).unwrap();
        println!("JSON:\n{}", &json);

        let part = serde_json::from_str(&json).unwrap();
        assert_eq!(json, serde_json::to_string(&part).unwrap());

        part
    };

    // Validate it.
    if let Err(e) = part.validate() {
        eprintln!("Validation failed: {}", e);
        return
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

    // Potential test
    let mut set: BTreeSet<String> = BTreeSet::new();
    let mut last_added = 0usize;
    for i in 0usize.. {
        part.generate(&mut ws, &mut rng);
        if !set.contains(ws.get_result()) {
            set.insert(ws.get_result().to_owned());
            last_added = i;
        } else if i > last_added + 50000 {
            break;
        }
    }
    println!("Potential: {} (not exact)", set.len());
}