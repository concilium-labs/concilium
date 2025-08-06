use ahash::AHashSet;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;

pub fn generate_random_number_by_seed(hash: [u8; 32], to: u32, how_many: u32) -> Vec<u32> {
    let mut rng = ChaCha20Rng::from_seed(hash);
    let mut seen = AHashSet::with_capacity(how_many as usize);
    let mut result = Vec::with_capacity(how_many as usize);

    while result.len() < how_many as usize {
        let num = rng.random_range(1..=to);
        if seen.contains(&num) {
            continue;
        }
        seen.insert(num);
        result.push(num);
    }

    result
}
