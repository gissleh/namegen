use namegen::{NamePart, FormattingRule, SampleSet, Sample, Name};
use rand::{SeedableRng, thread_rng};
use rand::rngs::SmallRng;
use std::fs::File;
use std::io::Read;
use time::PreciseTime;

fn main() {
    let mut part1 = NamePart::new_cfgrammar("first", &[FormattingRule::CapitalizeFirst], &[], true, true);
    let mut part2 = NamePart::new_cfgrammar("last", &[FormattingRule::CapitalizeFirst, FormattingRule::CapitalizeAfter('\'')], &[], true, true);

    // Load sample file.
    let mut file = File::open("./examples/res/name.txt").unwrap();
    let mut data = String::with_capacity(2048);
    file.read_to_string(&mut data).expect("File could not be read.");

    // Parse sample file.
    let mut new_set = true;
    let mut cutoff = 0;
    let mut sets: Vec<SampleSet> = Vec::new();
    for line in data.lines() {
        if line.len() < 2 {
            new_set = true;
            continue;
        }

        if line == "***" {
            cutoff = sets.len();
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
    for set in sets[..cutoff].iter() {
        if let Err(e) = part1.learn(set) {
            eprintln!("{:?}: {}", set.labels(), e);
        }
    }
    for set in sets[cutoff..].iter() {
        if let Err(e) = part2.learn(set) {
            eprintln!("{:?}: {}", set.labels(), e);
        }
    }

    // Setup name
    let mut name = Name::new();
    name.add_part(part1);
    name.add_part(part2);
    name.add_format("first_name", "{first}");
    name.add_format("last_name", "{last}");
    name.add_format("full_name", "{first} {last}");

    // Show the structure
    #[cfg(feature = "serde")]
    let name: Name = {
        let json =  serde_json::to_string(&name).unwrap();
        println!("JSON:\n{}", &json);

        let name = serde_json::from_str(&json).unwrap();
        assert_eq!(json, serde_json::to_string(&name).unwrap());

        name
    };

    let start = PreciseTime::now();
    name.generate( "full_name").unwrap().take(100000).for_each(drop);
    let end = PreciseTime::now();

    for (i, result) in name.generate( "full_name").unwrap().enumerate().take(72) {
        print!("{result:<width$} ", result = result, width = 19);
        if i > 0 && (i % 4 == 3) {
            print!("\n")
        }
    }

    println!("Benchmark: {}ns per name", start.to(end).num_nanoseconds().unwrap() / 100000);
}