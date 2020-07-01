use std::fs::File;
use std::io::Read;
use rand::{SeedableRng, thread_rng};
use rand::rngs::SmallRng;
use time::PreciseTime;
use namegen::{NamePart, FormattingRule, SampleSet, Sample, WorkingSet};
use std::collections::BTreeSet;

fn main() {
    let mut part = NamePart::new_cfgrammar(
        // Give the part a unique name.
        "first",

        // Formatting rules to apply to the finished name.
        &[
            FormattingRule::CapitalizeFirst,
        ],

        // These will be treated as if they were one letter by the generator,
        // but not the formatter. This is useful for clusters of consonants or
        // vowels that always appear together and should be generated together
        // as well.
        &["th", "sh", "ts", "tz", "ll", "ae", "ss", "nn"],

        // Restrict token frequencies and adjacent subtokens/letters.
        // These sacrifice potential variations for faithfulness.
        true, true
    );

    // Load sample file.
    let mut file = File::open("./examples/res/cfgrammar.txt").unwrap();
    let mut data = String::with_capacity(2048);
    file.read_to_string(&mut data).unwrap();

    // Parse sample file.
    let mut new_set = true;
    let mut sets: Vec<SampleSet> = Vec::new();
    for line in data.lines() {
        if line.len() < 2 {
            new_set = true;
            continue;
        }

        if new_set {
            let labels: Vec<&str> = line.split(' ').filter(|t| t.len() > 0).collect();

            sets.push(SampleSet::new(&labels));
            new_set = false;
        } else {
            sets.last_mut().unwrap().add_sample(
                Sample::Tokens(line.split(' ').filter(|t| t.len() > 0).map(|t| t.to_owned()).collect()),
            );
        }
    }
    for set in sets.iter() {
        if let Err(e) = part.learn(set) {
            eprintln!("{:?}: {}", set.labels(), e);
        }
    }

    // Local state bundle (to save allocs) and rng impl to use.
    let mut ws = WorkingSet::new();
    let mut rng = SmallRng::from_rng(thread_rng()).unwrap();

    // A little benchmark.
    let start = PreciseTime::now();
    for _ in 0..100000 {
        part.generate(&mut ws, &mut rng);
    }
    let end = PreciseTime::now();

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
    println!("Potential: {}", set.len());

    // Show the structure
    #[cfg(feature = "serde")]
    let part: NamePart = {
        let json =  serde_json::to_string(&part).unwrap();
        println!("JSON:\n{}", &json);

        let part = serde_json::from_str(&json).unwrap();
        assert_eq!(json, serde_json::to_string(&part).unwrap());

        part
    };

    // Here's the output.
    for n in 1..=77 {
        part.generate(&mut ws, &mut rng);

        print!("{result:<width$} ", result = ws.get_result(), width = 10);
        if n % 7 == 0 {
            print!("\n")
        }
    }

    println!("Benchmark: {}ns per name", start.to(end).num_nanoseconds().unwrap() / 100000);
}
