//! Running example and opposing repetition diagnostic.
use topic_039_adaptive_string_search::{Candidate, Finder};
fn main() {
    for c in [
        Candidate::Direct,
        Candidate::Horspool,
        Candidate::BoyerMoore,
        Candidate::TwoWay,
        Candidate::Adaptive,
    ] {
        println!("{c:?}: {:?}", Finder::new(b"ABCD", c).diagnose(b"ABAAABCD"));
        let mut p = vec![b'a'; 32];
        p[0] = b'b';
        println!(
            "suffix trap {c:?}: {:?}",
            Finder::new(&p, c).diagnose(&vec![b'a'; 32768])
        );
    }
}
