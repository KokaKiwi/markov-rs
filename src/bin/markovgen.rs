extern crate markov;

use std::collections::HashMap;
use markov::MarkovGenerator;

fn main() {
    let args = ::std::os::args();
    let path = Path::new(args[1].as_slice());

    let mut markov = MarkovGenerator::new(HashMap::new());
    markov.feed_from_file(&path);

    let text = markov.generate_text(30);
    println!("{}", text);
}
