//! Print canonical component labels of the running example for each algorithm.
fn main() {
    let g = topic033::example();
    for (name, f) in [
        ("tarjan", topic033::tarjan as fn(&_) -> _),
        ("kosaraju", topic033::kosaraju),
        ("gabow", topic033::gabow),
    ] {
        let id = f(&g);
        assert!(topic033::certify(&g, &id));
        println!("{name}: {:?}", topic033::canonical(&id));
    }
}
