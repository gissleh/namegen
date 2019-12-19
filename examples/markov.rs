use namegen::markov::{Markov};
use namegen::core::{WorkingSet};
use time::PreciseTime;
use rand::{SeedableRng, thread_rng};
use rand::rngs::SmallRng;

fn main() {
    let mut gen = Markov::with_constraints(
        // These will be treated as if they were one letter.
        &["th", "ae", "ss", "nn"],
        
        // Length restrict start, middle, end; and token frequency restriction.
        // These sacrifice potential variations for faithfulness.
        true, false, false, true
    );
    
    // This generator only starts to shine when there's 100 samples,
    // but providing that is outside the scope of this example. Unlike
    // the other generator, this one takes the samples raw. You should
    // add digraphs and other clusters of letters in the constructor, however.
    gen.learn("aeyna").unwrap();
    gen.learn("ilyna").unwrap();
    gen.learn("ehanis").unwrap();
    gen.learn("renala").unwrap();
    gen.learn("vynira").unwrap();
    gen.learn("nalena").unwrap();
    gen.learn("seyanere").unwrap();
    gen.learn("eriane").unwrap();
    gen.learn("yseli").unwrap();
    gen.learn("janysa").unwrap();
    gen.learn("kanaya").unwrap();
    gen.learn("aylena").unwrap();
    gen.learn("illeya").unwrap();
    gen.learn("natia").unwrap();
    gen.learn("enila").unwrap();
    gen.learn("marissa").unwrap();
    gen.learn("lema").unwrap();

    // A WorkingSet is a bundle of local variables.
    let mut ws = WorkingSet::new();

    // You don't need a powerful RNG, speed is much more important.
    // This is where you'd hook in your RNG if this was part of a
    // procedurally generated game world.
    let mut rng = SmallRng::from_rng(thread_rng()).unwrap();

    // This will print the "state machine". As you can see, there aren't many branches
    // with the pitiful amount of samples.
    gen.print();

    // A little benchmark.
    // Expect 50% increase in time per name if you have several hundred samples.
    let start = PreciseTime::now();
    for _ in 0..10000000 {
        gen.generate(&mut ws, &mut rng);
    }
    let end = PreciseTime::now();
    println!("Benchmark: {}ns per name", start.to(end).num_nanoseconds().unwrap() / 10000000);

    // Here's the output.
    for n in 1..=256 {
        gen.generate(&mut ws, &mut rng);

        print!("{result:<width$} ", result = ws.get_result(), width = 10);
        if n % 8 == 0 {
            print!("\n")
        }
    }
}