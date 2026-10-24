//! Deterministic seed partitioning for multi-replication simulation experiments.

use std::collections::HashSet;

/// Allocates deterministic, unique seeds for replications using golden-ratio integer dispersion.
pub fn allocate_replication_seeds(base_seed: u64, replications: usize) -> Vec<u64> {
    let mut seeds = Vec::with_capacity(replications);
    let mut seen = HashSet::new();
    for r in 0..replications {
        let seed = base_seed.wrapping_add((r as u64).wrapping_mul(0x9E3779B97F4A7C15));
        assert!(seen.insert(seed), "replication seeds must be unique");
        seeds.push(seed);
    }
    seeds
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seed_partitioning_uniqueness_and_determinism() {
        let seeds1 = allocate_replication_seeds(42, 100);
        let seeds2 = allocate_replication_seeds(42, 100);
        assert_eq!(seeds1, seeds2);
        assert_eq!(seeds1.len(), 100);

        let unique: HashSet<u64> = seeds1.into_iter().collect();
        assert_eq!(unique.len(), 100);
    }
}
