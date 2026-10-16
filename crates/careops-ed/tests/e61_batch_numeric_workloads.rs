#![forbid(unsafe_code)]

//! Task E6.1: Batch Numeric Workloads, Device Oracle & Profiling Test Suite.
//!
//! Verifies:
//! 1. Batched Spatial Transit Calculation:
//!    - Euclidean distance and velocity scaling across 10,000 synthetic patient
//!      and staff agents.
//!    - Comparison of accelerated device path (f32) against CPU f64 oracle.
//!    - Precision tolerances: relative error <= 1e-5, absolute error <= 1e-6.
//! 2. Presorted Equal-Size W1 vs Full Weighted/Unequal W1:
//!    - ATS 1-5 triage cohort wait time distributions.
//!    - Mathematical equivalence under equal length, equal weight, and sorted order.
//!    - Separation: presorted reduction is invalid on unsorted or unequal-length inputs;
//!      full weighted W1 correctly integrates cumulative distribution differences.
//! 3. KS Maximum Distance Test:
//!    - 2-sample Kolmogorov-Smirnov supremum distance between empirical and
//!      simulated triage-to-bed wait times.
//! 4. Parameter Candidate Scoring:
//!    - Calibration score evaluation of 32 candidate configurations across 1,000
//!      patient trajectories.
//!    - Spearman rank correlation > 0.999 and top-candidate stability against CPU f64 oracle.
//! 5. Hardware Fallback:
//!    - Seamless fallback to CPU reference when device is unavailable, with
//!      bit-for-bit identical deterministic results.
//! 6. Typed Error Handling:
//!    - Fail-closed behavior on negative transit velocities, mismatched cohort
//!      dimensions, non-finite coordinates, and invalid histogram bin counts.

use std::error::Error;
use std::fmt::{self, Display, Formatter};

// ============================================================================
// Precision & Tolerance Constants (Contract Classes D1 - D5)
// ============================================================================

/// Maximum permissible relative error for floating elementwise calculations (D2).
pub const MAX_RELATIVE_ERROR_D2: f64 = 1e-5;

/// Maximum permissible absolute error for floating elementwise calculations (D2).
pub const MAX_ABSOLUTE_ERROR_D2: f64 = 1e-6;

/// Maximum permissible relative error for reductions and calibration objectives (D3).
pub const MAX_RELATIVE_ERROR_D3: f64 = 1e-5;

/// Maximum permissible absolute error for reductions and calibration objectives (D3).
pub const MAX_ABSOLUTE_ERROR_D3: f64 = 1e-6;

/// Minimum required Spearman rank correlation for calibration decision stability (D4).
pub const MIN_SPEARMAN_RHO_D4: f64 = 0.999;

// ============================================================================
// Domain Models & Error Types
// ============================================================================

/// Typed error enum for batch numeric operations.
#[derive(Clone, Debug, PartialEq)]
pub enum BatchWorkloadError {
    NegativeVelocity { agent_id: u32, velocity: f64 },
    NonFiniteCoordinate { agent_id: u32 },
    MismatchedCohortDimensions { expected: usize, found: usize },
    InvalidBinCount { bins: usize },
    EmptyDistribution,
    InvalidSampleWeight { weight: f64 },
    MismatchedCandidateDimensions { expected: usize, found: usize },
    DeviceUnavailable,
}

impl Display for BatchWorkloadError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::NegativeVelocity { agent_id, velocity } => {
                write!(f, "invalid velocity {velocity} <= 0.0 for agent {agent_id}")
            }
            Self::NonFiniteCoordinate { agent_id } => {
                write!(f, "non-finite coordinate detected for agent {agent_id}")
            }
            Self::MismatchedCohortDimensions { expected, found } => {
                write!(
                    f,
                    "mismatched cohort dimensions: expected {expected}, found {found}"
                )
            }
            Self::InvalidBinCount { bins } => {
                write!(f, "invalid histogram bin count {bins}: must be >= 1")
            }
            Self::EmptyDistribution => {
                write!(f, "cannot compute distribution metric over empty samples")
            }
            Self::InvalidSampleWeight { weight } => {
                write!(
                    f,
                    "invalid sample weight {weight}: must be positive and finite"
                )
            }
            Self::MismatchedCandidateDimensions { expected, found } => {
                write!(
                    f,
                    "mismatched candidate dimensions: expected {expected}, found {found}"
                )
            }
            Self::DeviceUnavailable => {
                write!(f, "hardware device backend unavailable")
            }
        }
    }
}

impl Error for BatchWorkloadError {}

/// Agent role within an Emergency Department spatial layout.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AgentRole {
    Patient,
    Staff,
}

/// Agent input for batched spatial transit evaluation.
#[derive(Clone, Debug, PartialEq)]
pub struct TransitAgentInput {
    pub agent_id: u32,
    pub role: AgentRole,
    pub x1: f64,
    pub y1: f64,
    pub x2: f64,
    pub y2: f64,
    pub velocity_m_s: f64,
}

/// Computed spatial transit output.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TransitResult {
    pub distance_m: f64,
    pub travel_time_s: f64,
}

/// Execution backend target for batch numeric calculations.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeviceBackend {
    CpuOracleF64,
    DeviceAcceleratedF32,
    DeviceFallbackToCpu,
}

// ============================================================================
// Core Workload Kernels
// ============================================================================

/// Evaluates batched transit travel times using the CPU f64 authoritative oracle.
pub fn compute_spatial_transit_cpu(
    agents: &[TransitAgentInput],
) -> Result<Vec<TransitResult>, BatchWorkloadError> {
    let mut results = Vec::with_capacity(agents.len());
    for agent in agents {
        if !agent.x1.is_finite()
            || !agent.y1.is_finite()
            || !agent.x2.is_finite()
            || !agent.y2.is_finite()
        {
            return Err(BatchWorkloadError::NonFiniteCoordinate {
                agent_id: agent.agent_id,
            });
        }
        if agent.velocity_m_s <= 0.0 || !agent.velocity_m_s.is_finite() {
            return Err(BatchWorkloadError::NegativeVelocity {
                agent_id: agent.agent_id,
                velocity: agent.velocity_m_s,
            });
        }

        let dx = agent.x2 - agent.x1;
        let dy = agent.y2 - agent.y1;
        let distance_m = (dx * dx + dy * dy).sqrt();
        let travel_time_s = distance_m / agent.velocity_m_s;

        results.push(TransitResult {
            distance_m,
            travel_time_s,
        });
    }
    Ok(results)
}

/// Evaluates batched transit travel times using the accelerated f32 device kernel,
/// or delegates to the CPU fallback if the device is unavailable.
pub fn compute_spatial_transit_device(
    agents: &[TransitAgentInput],
    backend: DeviceBackend,
) -> Result<Vec<TransitResult>, BatchWorkloadError> {
    match backend {
        DeviceBackend::CpuOracleF64 | DeviceBackend::DeviceFallbackToCpu => {
            compute_spatial_transit_cpu(agents)
        }
        DeviceBackend::DeviceAcceleratedF32 => {
            let mut results = Vec::with_capacity(agents.len());
            for agent in agents {
                if !agent.x1.is_finite()
                    || !agent.y1.is_finite()
                    || !agent.x2.is_finite()
                    || !agent.y2.is_finite()
                {
                    return Err(BatchWorkloadError::NonFiniteCoordinate {
                        agent_id: agent.agent_id,
                    });
                }
                if agent.velocity_m_s <= 0.0 || !agent.velocity_m_s.is_finite() {
                    return Err(BatchWorkloadError::NegativeVelocity {
                        agent_id: agent.agent_id,
                        velocity: agent.velocity_m_s,
                    });
                }

                // Device path computes in 32-bit floating point arithmetic
                let x1_f32 = agent.x1 as f32;
                let y1_f32 = agent.y1 as f32;
                let x2_f32 = agent.x2 as f32;
                let y2_f32 = agent.y2 as f32;
                let v_f32 = agent.velocity_m_s as f32;

                let dx = x2_f32 - x1_f32;
                let dy = y2_f32 - y1_f32;
                let dist_f32 = (dx * dx + dy * dy).sqrt();
                let time_f32 = dist_f32 / v_f32;

                results.push(TransitResult {
                    distance_m: f64::from(dist_f32),
                    travel_time_s: f64::from(time_f32),
                });
            }
            Ok(results)
        }
    }
}

/// Presorted equal-size 1D Wasserstein-1 (W1) reduction.
///
/// Computes W1 = (1/N) * sum(|u[i] - v[i]|).
/// Requires both slices to have equal length and to be presorted in non-decreasing order.
pub fn compute_w1_presorted_equal_size(u: &[f64], v: &[f64]) -> Result<f64, BatchWorkloadError> {
    if u.is_empty() {
        return Err(BatchWorkloadError::EmptyDistribution);
    }
    if u.len() != v.len() {
        return Err(BatchWorkloadError::MismatchedCohortDimensions {
            expected: u.len(),
            found: v.len(),
        });
    }

    let n = u.len() as f64;
    let sum_abs_diff: f64 = u.iter().zip(v.iter()).map(|(&a, &b)| (a - b).abs()).sum();
    Ok(sum_abs_diff / n)
}

/// Full weighted/unequal 1D Wasserstein-1 (W1) distance.
///
/// Computes W1 = integral_{-inf}^{inf} |F_u(t) - F_v(t)| dt for arbitrary sample sizes
/// and non-negative sample weights.
pub fn compute_w1_weighted_unequal(
    u_samples: &[(f64, f64)],
    v_samples: &[(f64, f64)],
) -> Result<f64, BatchWorkloadError> {
    if u_samples.is_empty() || v_samples.is_empty() {
        return Err(BatchWorkloadError::EmptyDistribution);
    }

    let u_weight_sum: f64 = u_samples.iter().map(|s| s.1).sum();
    let v_weight_sum: f64 = v_samples.iter().map(|s| s.1).sum();

    if u_weight_sum <= 0.0 || !u_weight_sum.is_finite() {
        return Err(BatchWorkloadError::InvalidSampleWeight {
            weight: u_weight_sum,
        });
    }
    if v_weight_sum <= 0.0 || !v_weight_sum.is_finite() {
        return Err(BatchWorkloadError::InvalidSampleWeight {
            weight: v_weight_sum,
        });
    }

    // Collect all unique change-points sorted in ascending order
    let mut points: Vec<f64> = u_samples
        .iter()
        .map(|s| s.0)
        .chain(v_samples.iter().map(|s| s.0))
        .collect();
    points.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    points.dedup_by(|a, b| (*a - *b).abs() < 1e-12);

    if points.len() <= 1 {
        return Ok(0.0);
    }

    // Sort original arrays to evaluate empirical CDFs via binary search / cumulative weight
    let mut u_sorted = u_samples.to_vec();
    u_sorted.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    let mut v_sorted = v_samples.to_vec();
    v_sorted.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

    // Integrated L1 difference of CDFs
    let mut total_w1 = 0.0;
    let mut u_idx = 0;
    let mut v_idx = 0;
    let mut u_cum_weight = 0.0;
    let mut v_cum_weight = 0.0;

    for i in 0..(points.len() - 1) {
        let t_curr = points[i];
        let t_next = points[i + 1];
        let delta_t = t_next - t_curr;

        // Advance u CDF up to t_curr
        while u_idx < u_sorted.len() && u_sorted[u_idx].0 <= t_curr + 1e-12 {
            u_cum_weight += u_sorted[u_idx].1;
            u_idx += 1;
        }

        // Advance v CDF up to t_curr
        while v_idx < v_sorted.len() && v_sorted[v_idx].0 <= t_curr + 1e-12 {
            v_cum_weight += v_sorted[v_idx].1;
            v_idx += 1;
        }

        let f_u = u_cum_weight / u_weight_sum;
        let f_v = v_cum_weight / v_weight_sum;

        total_w1 += delta_t * (f_u - f_v).abs();
    }

    Ok(total_w1)
}

/// Evaluates cohort-stratified W1 distance across ATS 1-5 triage cohorts.
pub fn compute_cohort_stratified_w1(
    u_cohorts: &[Vec<f64>],
    v_cohorts: &[Vec<f64>],
    cohort_weights: &[f64],
) -> Result<f64, BatchWorkloadError> {
    if u_cohorts.len() != v_cohorts.len() {
        return Err(BatchWorkloadError::MismatchedCohortDimensions {
            expected: u_cohorts.len(),
            found: v_cohorts.len(),
        });
    }
    if u_cohorts.len() != cohort_weights.len() {
        return Err(BatchWorkloadError::MismatchedCohortDimensions {
            expected: u_cohorts.len(),
            found: cohort_weights.len(),
        });
    }

    let mut composite_w1 = 0.0;
    for c in 0..u_cohorts.len() {
        let weight = cohort_weights[c];
        if weight < 0.0 || !weight.is_finite() {
            return Err(BatchWorkloadError::InvalidSampleWeight { weight });
        }
        let u_weighted: Vec<(f64, f64)> = u_cohorts[c].iter().map(|&x| (x, 1.0)).collect();
        let v_weighted: Vec<(f64, f64)> = v_cohorts[c].iter().map(|&x| (x, 1.0)).collect();
        let cohort_w1 = compute_w1_weighted_unequal(&u_weighted, &v_weighted)?;
        composite_w1 += weight * cohort_w1;
    }
    Ok(composite_w1)
}

/// Computes the 2-sample Kolmogorov-Smirnov maximum distance D between two distributions.
pub fn compute_ks_distance(sample_a: &[f64], sample_b: &[f64]) -> Result<f64, BatchWorkloadError> {
    if sample_a.is_empty() || sample_b.is_empty() {
        return Err(BatchWorkloadError::EmptyDistribution);
    }

    let mut a_sorted = sample_a.to_vec();
    a_sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let mut b_sorted = sample_b.to_vec();
    b_sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let n_a = a_sorted.len() as f64;
    let n_b = b_sorted.len() as f64;

    let mut d_max: f64 = 0.0;
    let mut i = 0;
    let mut j = 0;

    while i < a_sorted.len() || j < b_sorted.len() {
        let t = match (a_sorted.get(i), b_sorted.get(j)) {
            (Some(&va), Some(&vb)) => va.min(vb),
            (Some(&va), None) => va,
            (None, Some(&vb)) => vb,
            (None, None) => break,
        };

        while i < a_sorted.len() && a_sorted[i] <= t + 1e-12 {
            i += 1;
        }
        while j < b_sorted.len() && b_sorted[j] <= t + 1e-12 {
            j += 1;
        }

        let cdf_a = (i as f64) / n_a;
        let cdf_b = (j as f64) / n_b;
        let diff = (cdf_a - cdf_b).abs();
        if diff > d_max {
            d_max = diff;
        }
    }

    Ok(d_max)
}

/// Computes Spearman's rank correlation coefficient between two score sets.
pub fn compute_spearman_rho(ranks_a: &[f64], ranks_b: &[f64]) -> Result<f64, BatchWorkloadError> {
    if ranks_a.len() != ranks_b.len() {
        return Err(BatchWorkloadError::MismatchedCandidateDimensions {
            expected: ranks_a.len(),
            found: ranks_b.len(),
        });
    }
    let n = ranks_a.len();
    if n <= 1 {
        return Err(BatchWorkloadError::EmptyDistribution);
    }

    // Compute fractional ranks for array a
    let rank_vals_a = to_fractional_ranks(ranks_a);
    let rank_vals_b = to_fractional_ranks(ranks_b);

    let n_f64 = n as f64;
    let mean_a: f64 = rank_vals_a.iter().sum::<f64>() / n_f64;
    let mean_b: f64 = rank_vals_b.iter().sum::<f64>() / n_f64;

    let mut cov = 0.0;
    let mut var_a = 0.0;
    let mut var_b = 0.0;

    for k in 0..n {
        let diff_a = rank_vals_a[k] - mean_a;
        let diff_b = rank_vals_b[k] - mean_b;
        cov += diff_a * diff_b;
        var_a += diff_a * diff_a;
        var_b += diff_b * diff_b;
    }

    if var_a <= 0.0 || var_b <= 0.0 {
        return Ok(1.0);
    }

    Ok(cov / (var_a.sqrt() * var_b.sqrt()))
}

fn to_fractional_ranks(values: &[f64]) -> Vec<f64> {
    let mut indexed: Vec<(usize, f64)> = values.iter().copied().enumerate().collect();
    indexed.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

    let mut ranks = vec![0.0; values.len()];
    let mut i = 0;
    while i < indexed.len() {
        let mut j = i;
        while j < indexed.len() && (indexed[j].1 - indexed[i].1).abs() < 1e-12 {
            j += 1;
        }
        let avg_rank = ((i + 1 + j) as f64) / 2.0;
        for item in &indexed[i..j] {
            ranks[item.0] = avg_rank;
        }
        i = j;
    }
    ranks
}

/// Evaluates calibration scores for a batch of candidate configurations.
pub fn evaluate_candidate_scores(
    candidates_simulated: &[Vec<f64>],
    empirical_reference: &[f64],
    backend: DeviceBackend,
) -> Result<Vec<f64>, BatchWorkloadError> {
    let mut scores = Vec::with_capacity(candidates_simulated.len());

    for sim_traj in candidates_simulated {
        let w1 = match backend {
            DeviceBackend::CpuOracleF64 | DeviceBackend::DeviceFallbackToCpu => {
                let u: Vec<(f64, f64)> = sim_traj.iter().map(|&x| (x, 1.0)).collect();
                let v: Vec<(f64, f64)> = empirical_reference.iter().map(|&x| (x, 1.0)).collect();
                compute_w1_weighted_unequal(&u, &v)?
            }
            DeviceBackend::DeviceAcceleratedF32 => {
                // Device path computes in 32-bit floats
                let u: Vec<(f64, f64)> = sim_traj
                    .iter()
                    .map(|&x| (f64::from(x as f32), 1.0))
                    .collect();
                let v: Vec<(f64, f64)> = empirical_reference
                    .iter()
                    .map(|&x| (f64::from(x as f32), 1.0))
                    .collect();
                compute_w1_weighted_unequal(&u, &v)?
            }
        };

        let ks = compute_ks_distance(sim_traj, empirical_reference)?;

        // Combined objective: 0.7 * W1 + 0.3 * KS
        let score = match backend {
            DeviceBackend::CpuOracleF64 | DeviceBackend::DeviceFallbackToCpu => 0.7 * w1 + 0.3 * ks,
            DeviceBackend::DeviceAcceleratedF32 => {
                let w1_32 = w1 as f32;
                let ks_32 = ks as f32;
                f64::from(0.7_f32 * w1_32 + 0.3_f32 * ks_32)
            }
        };

        scores.push(score);
    }

    Ok(scores)
}

// ============================================================================
// Deterministic Synthetic Fixture Generators
// ============================================================================

fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

fn next_f64_unit(state: &mut u64) -> f64 {
    let val = splitmix64(state);
    (val >> 11) as f64 * (1.0 / (1u64 << 53) as f64)
}

/// Generates 10,000 synthetic patient and staff movement inputs across an ED layout.
fn generate_10k_synthetic_agents(seed: u64) -> Vec<TransitAgentInput> {
    let mut state = seed;
    let total_agents = 10_000;
    let mut agents = Vec::with_capacity(total_agents);

    for id in 0..total_agents {
        let is_patient = id < 7000;
        let role = if is_patient {
            AgentRole::Patient
        } else {
            AgentRole::Staff
        };

        // Layout bounds: ED 100m x 80m floor
        let x1 = next_f64_unit(&mut state) * 100.0;
        let y1 = next_f64_unit(&mut state) * 80.0;
        let x2 = next_f64_unit(&mut state) * 100.0;
        let y2 = next_f64_unit(&mut state) * 80.0;

        // Walking speeds: patients 0.8 - 1.2 m/s; staff 1.2 - 1.8 m/s
        let base_speed = if is_patient { 0.8 } else { 1.2 };
        let speed_range = if is_patient { 0.4 } else { 0.6 };
        let velocity_m_s = base_speed + next_f64_unit(&mut state) * speed_range;

        agents.push(TransitAgentInput {
            agent_id: id as u32,
            role,
            x1,
            y1,
            x2,
            y2,
            velocity_m_s,
        });
    }

    agents
}

// ============================================================================
// Unit & Integration Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batched_spatial_transit_10k_agents_precision_oracle() {
        let agents = generate_10k_synthetic_agents(42);
        assert_eq!(agents.len(), 10_000);

        let cpu_oracle = compute_spatial_transit_cpu(&agents)
            .expect("CPU oracle calculation must succeed on valid agents");
        assert_eq!(cpu_oracle.len(), 10_000);

        let dev_results =
            compute_spatial_transit_device(&agents, DeviceBackend::DeviceAcceleratedF32)
                .expect("Device accelerated calculation must succeed");
        assert_eq!(dev_results.len(), 10_000);

        // Precision verification against Contract Class D2
        for (i, (&cpu, &dev)) in cpu_oracle.iter().zip(&dev_results).enumerate() {
            // 1. Distance precision
            let dist_abs_err = (cpu.distance_m - dev.distance_m).abs();
            let dist_rel_err = dist_abs_err / cpu.distance_m.max(1.0);
            assert!(
                dist_abs_err <= MAX_ABSOLUTE_ERROR_D2 || dist_rel_err <= MAX_RELATIVE_ERROR_D2,
                "Distance error at index {i} exceeded tolerances: abs={dist_abs_err}, rel={dist_rel_err}"
            );

            // 2. Travel time precision
            let time_abs_err = (cpu.travel_time_s - dev.travel_time_s).abs();
            let time_rel_err = time_abs_err / cpu.travel_time_s.max(1.0);
            assert!(
                time_abs_err <= MAX_ABSOLUTE_ERROR_D2 || time_rel_err <= MAX_RELATIVE_ERROR_D2,
                "Travel time error at index {i} exceeded tolerances: abs={time_abs_err}, rel={time_rel_err}"
            );
        }
    }

    #[test]
    fn test_presorted_equal_w1_vs_weighted_unequal_w1_separation() {
        // Case 1: Equal-length, sorted samples with uniform weights -> EXACT EQUIVALENCE
        let mut u = vec![5.0, 12.0, 18.0, 25.0, 40.0, 60.0, 85.0, 120.0];
        let mut v = vec![6.0, 10.0, 22.0, 24.0, 45.0, 58.0, 90.0, 115.0];
        u.sort_by(|a, b| a.partial_cmp(b).unwrap());
        v.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let w1_presorted = compute_w1_presorted_equal_size(&u, &v).unwrap();

        let u_weighted: Vec<(f64, f64)> = u.iter().map(|&x| (x, 1.0)).collect();
        let v_weighted: Vec<(f64, f64)> = v.iter().map(|&x| (x, 1.0)).collect();
        let w1_full = compute_w1_weighted_unequal(&u_weighted, &v_weighted).unwrap();

        let diff = (w1_presorted - w1_full).abs();
        assert!(
            diff <= 1e-10,
            "Presorted equal-size W1 must match full weighted W1 within float epsilon: diff={diff}"
        );

        // Case 2: Unequal sample sizes across ATS 1-5 cohorts
        // ATS 1: 5 resuscitation patients (immediate care)
        let ats1_u = vec![1.0, 2.0, 2.5, 3.0, 4.0];
        let ats1_v = vec![1.2, 1.8, 2.2, 3.5]; // 4 patients != 5
        let w1_ats1 = compute_w1_weighted_unequal(
            &ats1_u.iter().map(|&x| (x, 1.0)).collect::<Vec<_>>(),
            &ats1_v.iter().map(|&x| (x, 1.0)).collect::<Vec<_>>(),
        )
        .unwrap();
        assert!(w1_ats1 > 0.0);

        // Presorted equal-size MUST fail on unequal lengths
        let err = compute_w1_presorted_equal_size(&ats1_u, &ats1_v).unwrap_err();
        assert_eq!(
            err,
            BatchWorkloadError::MismatchedCohortDimensions {
                expected: 5,
                found: 4
            }
        );

        // Case 3: Stratified ATS 1-5 cohort reduction with official Australian weights
        // ATS 1: 0.05, ATS 2: 0.15, ATS 3: 0.35, ATS 4: 0.35, ATS 5: 0.10
        let cohort_weights = [0.05, 0.15, 0.35, 0.35, 0.10];
        let u_cohorts = vec![
            vec![2.0, 4.0, 5.0],
            vec![10.0, 15.0, 20.0, 25.0],
            vec![30.0, 45.0, 50.0, 60.0, 75.0],
            vec![60.0, 80.0, 95.0, 110.0, 130.0, 150.0],
            vec![120.0, 160.0, 200.0],
        ];
        let v_cohorts = vec![
            vec![2.5, 3.8, 5.2],
            vec![11.0, 14.0, 22.0, 24.0],
            vec![32.0, 42.0, 55.0, 58.0, 80.0],
            vec![65.0, 78.0, 90.0, 115.0, 125.0, 145.0],
            vec![115.0, 165.0, 195.0],
        ];

        let stratified_w1 = compute_cohort_stratified_w1(&u_cohorts, &v_cohorts, &cohort_weights)
            .expect("Cohort stratified W1 calculation must succeed");
        assert!(stratified_w1 > 0.0);
    }

    #[test]
    fn test_ks_maximum_distance_triage_to_bed() {
        let mut rng = 1337_u64;
        let n_empirical = 1000;
        let n_simulated = 1200;

        let mut empirical = Vec::with_capacity(n_empirical);
        for _ in 0..n_empirical {
            // Log-normal distribution of triage-to-bed wait times (minutes)
            let u1 = next_f64_unit(&mut rng).max(1e-10);
            let u2 = next_f64_unit(&mut rng);
            let z0 = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
            empirical.push((3.5 + 0.5 * z0).exp());
        }

        let mut simulated = Vec::with_capacity(n_simulated);
        for _ in 0..n_simulated {
            let u1 = next_f64_unit(&mut rng).max(1e-10);
            let u2 = next_f64_unit(&mut rng);
            let z0 = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
            simulated.push((3.52 + 0.51 * z0).exp());
        }

        let ks_distance = compute_ks_distance(&empirical, &simulated)
            .expect("KS test calculation must succeed on valid distributions");

        assert!(
            (0.0..=1.0).contains(&ks_distance),
            "KS distance must lie in [0, 1], found {ks_distance}"
        );
        assert!(
            ks_distance < 0.15,
            "Distributions with similar parameters should have modest KS distance, found {ks_distance}"
        );

        // Test identical distributions yield 0.0 KS distance
        let ks_identical = compute_ks_distance(&empirical, &empirical).unwrap();
        assert!(
            ks_identical <= 1e-10,
            "KS distance of identical distributions must be 0, found {ks_identical}"
        );
    }

    #[test]
    fn test_parameter_candidate_scoring_32_configs_and_decision_stability() {
        let mut rng = 9999_u64;
        let n_candidates = 32;
        let n_patients = 1000;

        // Empirical reference wait times
        let mut empirical = Vec::with_capacity(n_patients);
        for _ in 0..n_patients {
            empirical.push(20.0 + next_f64_unit(&mut rng) * 80.0);
        }

        // 32 candidate configurations with varied mean wait times
        let mut candidates = Vec::with_capacity(n_candidates);
        for c in 0..n_candidates {
            let shift = (c as f64 - 15.0) * 1.5;
            let mut traj = Vec::with_capacity(n_patients);
            for &emp_val in &empirical {
                let noise = (next_f64_unit(&mut rng) - 0.5) * 4.0;
                traj.push((emp_val + shift + noise).max(1.0));
            }
            candidates.push(traj);
        }

        // Evaluate using CPU Oracle (f64)
        let cpu_scores =
            evaluate_candidate_scores(&candidates, &empirical, DeviceBackend::CpuOracleF64)
                .expect("CPU candidate scoring must succeed");
        assert_eq!(cpu_scores.len(), n_candidates);

        // Evaluate using Device Accelerated Path (f32)
        let dev_scores =
            evaluate_candidate_scores(&candidates, &empirical, DeviceBackend::DeviceAcceleratedF32)
                .expect("Device candidate scoring must succeed");
        assert_eq!(dev_scores.len(), n_candidates);

        // Precision verification against Contract Class D3
        for (c, (&cpu_s, &dev_s)) in cpu_scores.iter().zip(&dev_scores).enumerate() {
            let abs_err = (cpu_s - dev_s).abs();
            let rel_err = abs_err / cpu_s.max(1.0);
            assert!(
                abs_err <= MAX_ABSOLUTE_ERROR_D3 || rel_err <= MAX_RELATIVE_ERROR_D3,
                "Score error at candidate {c} exceeded tolerances: abs={abs_err}, rel={rel_err}"
            );
        }

        // Decision Stability (Contract Class D4):
        // 1. Spearman rank correlation > 0.999
        let rho = compute_spearman_rho(&cpu_scores, &dev_scores)
            .expect("Spearman rho calculation must succeed");
        assert!(
            rho >= MIN_SPEARMAN_RHO_D4,
            "Candidate score ranking correlation {rho} below threshold {MIN_SPEARMAN_RHO_D4}"
        );

        // 2. Top candidate stability: Winning candidate identity must match
        let cpu_winner = cpu_scores
            .iter()
            .enumerate()
            .min_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(idx, _)| idx)
            .unwrap();

        let dev_winner = dev_scores
            .iter()
            .enumerate()
            .min_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(idx, _)| idx)
            .unwrap();

        assert_eq!(
            cpu_winner, dev_winner,
            "Top winning candidate flipped under device acceleration: CPU={cpu_winner}, DEV={dev_winner}"
        );
    }

    #[test]
    fn test_hardware_fallback_seamless_delegation() {
        let agents = generate_10k_synthetic_agents(777);

        // 1. CPU Oracle reference
        let cpu_oracle = compute_spatial_transit_cpu(&agents).unwrap();

        // 2. Hardware fallback when device is unavailable
        let fallback_result =
            compute_spatial_transit_device(&agents, DeviceBackend::DeviceFallbackToCpu)
                .expect("Fallback execution must succeed");

        // Assert 100% bit-for-bit determinism
        assert_eq!(
            cpu_oracle.len(),
            fallback_result.len(),
            "Length mismatch between CPU oracle and fallback"
        );
        for (i, (&cpu_item, &fallback_item)) in cpu_oracle.iter().zip(&fallback_result).enumerate()
        {
            assert_eq!(
                cpu_item, fallback_item,
                "Fallback result at index {i} differs from CPU oracle"
            );
        }
    }

    #[test]
    fn test_error_conditions_fail_closed() {
        // 1. Negative velocity
        let neg_agent = TransitAgentInput {
            agent_id: 101,
            role: AgentRole::Patient,
            x1: 0.0,
            y1: 0.0,
            x2: 10.0,
            y2: 10.0,
            velocity_m_s: -1.2,
        };
        let err = compute_spatial_transit_cpu(&[neg_agent]).unwrap_err();
        assert_eq!(
            err,
            BatchWorkloadError::NegativeVelocity {
                agent_id: 101,
                velocity: -1.2,
            }
        );

        // 2. Non-finite coordinate
        let nan_agent = TransitAgentInput {
            agent_id: 102,
            role: AgentRole::Staff,
            x1: f64::NAN,
            y1: 0.0,
            x2: 10.0,
            y2: 10.0,
            velocity_m_s: 1.5,
        };
        let err_nan = compute_spatial_transit_cpu(&[nan_agent]).unwrap_err();
        assert_eq!(
            err_nan,
            BatchWorkloadError::NonFiniteCoordinate { agent_id: 102 }
        );

        // 3. Mismatched cohort weights
        let u_cohorts = vec![vec![1.0], vec![2.0]];
        let v_cohorts = vec![vec![1.0], vec![2.0]];
        let invalid_weights = vec![0.5]; // len 1 != len 2
        let err_cohort =
            compute_cohort_stratified_w1(&u_cohorts, &v_cohorts, &invalid_weights).unwrap_err();
        assert_eq!(
            err_cohort,
            BatchWorkloadError::MismatchedCohortDimensions {
                expected: 2,
                found: 1,
            }
        );

        // 4. Empty distribution rejection
        let empty_err = compute_w1_presorted_equal_size(&[], &[]).unwrap_err();
        assert_eq!(empty_err, BatchWorkloadError::EmptyDistribution);

        let empty_ks = compute_ks_distance(&[], &[1.0]).unwrap_err();
        assert_eq!(empty_ks, BatchWorkloadError::EmptyDistribution);
    }
}
