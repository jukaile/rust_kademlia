mod observer;
use observer::Observer;

fn main() {
    let bootstrap = std::env::args()
        .nth(1)
        .expect("Usage: observer <bootstrap_addr>");

    let mut obs = Observer::new(0);
    obs.observe(&bootstrap);
}