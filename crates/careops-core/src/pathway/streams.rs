//! Explicit draw ownership and stream namespaces preventing cross-module draw leakage (CC-07).

/// Namespace domain tags ensuring disjoint stream seeds.
pub const STREAM_DOMAIN_PROGRESSION: u64 = 0x5052_4F47_5245_5353; // "PROGRESS"
pub const STREAM_DOMAIN_SERVICE: u64 = 0x5345_5256_4943_4553; // "SERVICES"
pub const STREAM_DOMAIN_ROUTING: u64 = 0x524F_5554_494E_4753; // "ROUTINGS"

/// Simple deterministic splitmix pseudo-random stream.
#[derive(Clone, Debug, PartialEq)]
pub struct SubsystemStream {
    pub domain_tag: u64,
    pub draw_count: usize,
    state: u64,
}

impl SubsystemStream {
    pub fn new(base_seed: u64, domain_tag: u64) -> Self {
        let initial_state = base_seed
            .wrapping_add(domain_tag)
            .wrapping_mul(0x9E3779B97F4A7C15);
        Self {
            domain_tag,
            draw_count: 0,
            state: initial_state,
        }
    }

    /// Draws next uniform floating point number in [0.0, 1.0).
    pub fn next_uniform(&mut self) -> f64 {
        self.draw_count += 1;
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1);
        let u = (self.state >> 11) as f64 / 9007199254740992.0; // 53-bit resolution
        u.min(0.9999999999999999)
    }

    /// Draws an exponential random variate with rate lambda.
    pub fn next_exponential(&mut self, lambda: f64) -> f64 {
        if lambda <= 0.0 {
            return f64::INFINITY;
        }
        let u = self.next_uniform();
        -(1.0 - u).ln() / lambda
    }
}

/// Explicit bundle of decoupled random streams for a patient episode.
#[derive(Clone, Debug, PartialEq)]
pub struct PathwayRngStreams {
    pub progression_stream: SubsystemStream,
    pub service_stream: SubsystemStream,
    pub routing_stream: SubsystemStream,
}

impl PathwayRngStreams {
    pub fn new(patient_base_seed: u64) -> Self {
        Self {
            progression_stream: SubsystemStream::new(patient_base_seed, STREAM_DOMAIN_PROGRESSION),
            service_stream: SubsystemStream::new(patient_base_seed, STREAM_DOMAIN_SERVICE),
            routing_stream: SubsystemStream::new(patient_base_seed, STREAM_DOMAIN_ROUTING),
        }
    }
}
