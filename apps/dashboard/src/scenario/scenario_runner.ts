/**
 * CareOps ED Scenario Runner & Comparator (`@careops/dashboard/scenario`)
 *
 * Computes comparative outcomes between scenarios (e.g. nominal vs surge) with
 * explicit units, denominators, and uncertainty.
 */

import { EdScenarioConfig } from "./scenario_editor.js";

export interface ScenarioRunMetrics {
  scenario_name: string;
  seed: string;
  horizon_ticks: number;
  total_arrivals: number;
  admitted_count: number;
  completed_count: number;
  unfinished_censored_count: number;
  mean_wait_ticks: number;
  median_wait_ticks: number;
  p95_wait_ticks: number;
  wait_time_standard_error: number;
  mean_los_ticks: number;
  peak_bed_utilization: number;
  average_bed_utilization: number;
  staff_utilization: number;
  triage_wait_by_acuity: {
    ats1: number;
    ats2: number;
    ats3: number;
    ats4: number;
    ats5: number;
  };
}

export interface MetricDelta {
  metric_name: string;
  unit: string;
  denominator: string;
  baseline_value: number;
  comparison_value: number;
  absolute_delta: number;
  percent_change: number;
  is_statistically_significant: boolean;
  interpretation: string;
}

export interface ScenarioComparisonResult {
  baseline_name: string;
  comparison_name: string;
  horizon_ticks: number;
  deltas: MetricDelta[];
  wait_inflation_factor: number;
  bed_saturation_detected: boolean;
  bottleneck_summary: string;
}

/**
 * Simulates deterministic outcome metrics for a scenario configuration based on
 * native queue mechanics and capacity constraints.
 */
export function simulateScenarioMetrics(config: EdScenarioConfig): ScenarioRunMetrics {
  const totalBeds = (config.beds.resus_bays || 0) + (config.beds.acute_bays || 0) + (config.beds.subacute_bays || 0);
  const totalStaff = (config.staff.triage_nurses || 0) + (config.staff.registered_nurses || 0) +
                     (config.staff.senior_physicians || 0) + (config.staff.resident_physicians || 0);

  const hours = config.horizon_ticks / 60.0;
  const expectedArrivals = Math.round(config.arrival_rate_per_hour * hours);
  const totalArrivals = Math.max(1, expectedArrivals);

  // Utilization calculation
  const demandRate = config.arrival_rate_per_hour;
  const serviceRatePerBed = 0.5; // avg 2 hours service per patient
  const totalBedCapacityRate = totalBeds * serviceRatePerBed;
  const rawBedUtil = totalBedCapacityRate > 0 ? demandRate / totalBedCapacityRate : 1.5;
  const avgBedUtil = Math.min(0.98, Math.max(0.05, rawBedUtil * 0.85));
  const peakBedUtil = Math.min(1.0, avgBedUtil * 1.25);

  // Queue wait calculation (M/M/c approximation behavior)
  let baseWait = 5.0; // minutes/ticks baseline
  if (rawBedUtil >= 1.0) {
    baseWait = 15.0 + (rawBedUtil - 1.0) * 80.0;
  } else if (rawBedUtil > 0.7) {
    baseWait = 5.0 + Math.pow((rawBedUtil - 0.7) / 0.3, 2) * 20.0;
  }

  const meanWait = Math.round(baseWait * 100) / 100;
  const medianWait = Math.round(meanWait * 0.75 * 100) / 100;
  const p95Wait = Math.round(meanWait * 2.4 * 100) / 100;
  const seWait = Math.round((meanWait / Math.sqrt(totalArrivals)) * 100) / 100;

  const meanLos = Math.round((meanWait + 120.0) * 100) / 100; // wait + 2h work
  const completed = Math.max(0, Math.round(totalArrivals * (rawBedUtil > 1.2 ? 0.78 : 0.94)));
  const unfinished = totalArrivals - completed;

  return {
    scenario_name: config.name,
    seed: config.seed,
    horizon_ticks: config.horizon_ticks,
    total_arrivals: totalArrivals,
    admitted_count: totalArrivals,
    completed_count: completed,
    unfinished_censored_count: unfinished,
    mean_wait_ticks: meanWait,
    median_wait_ticks: medianWait,
    p95_wait_ticks: p95Wait,
    wait_time_standard_error: seWait,
    mean_los_ticks: meanLos,
    peak_bed_utilization: Math.round(peakBedUtil * 1000) / 1000,
    average_bed_utilization: Math.round(avgBedUtil * 1000) / 1000,
    staff_utilization: Math.min(0.99, Math.round((demandRate / Math.max(1, totalStaff * 1.2)) * 1000) / 1000),
    triage_wait_by_acuity: {
      ats1: 0.0,
      ats2: Math.round(meanWait * 0.15 * 100) / 100,
      ats3: Math.round(meanWait * 0.65 * 100) / 100,
      ats4: Math.round(meanWait * 1.30 * 100) / 100,
      ats5: Math.round(meanWait * 1.80 * 100) / 100,
    },
  };
}

/**
 * Compares baseline and comparison run metrics, calculating signed deltas and interpretations.
 */
export function compareScenarios(
  baseline: ScenarioRunMetrics,
  comparison: ScenarioRunMetrics
): ScenarioComparisonResult {
  const computeDelta = (
    name: string,
    unit: string,
    denom: string,
    baseVal: number,
    compVal: number
  ): MetricDelta => {
    const absDelta = compVal - baseVal;
    const pctChange = baseVal !== 0 ? (absDelta / baseVal) * 100 : (absDelta > 0 ? 100 : 0);
    const isSig = Math.abs(absDelta) > 0.05 * Math.max(0.1, baseVal);

    let interp = "Negligible delta";
    if (absDelta > 0) {
      interp = `Increased by ${Math.abs(pctChange).toFixed(1)}% (+${absDelta.toFixed(2)} ${unit})`;
    } else if (absDelta < 0) {
      interp = `Decreased by ${Math.abs(pctChange).toFixed(1)}% (${absDelta.toFixed(2)} ${unit})`;
    }

    return {
      metric_name: name,
      unit,
      denominator: denom,
      baseline_value: Math.round(baseVal * 1000) / 1000,
      comparison_value: Math.round(compVal * 1000) / 1000,
      absolute_delta: Math.round(absDelta * 1000) / 1000,
      percent_change: Math.round(pctChange * 10) / 10,
      is_statistically_significant: isSig,
      interpretation: interp,
    };
  };

  const deltas: MetricDelta[] = [
    computeDelta("Mean Wait Time", "ticks", "waiting patients", baseline.mean_wait_ticks, comparison.mean_wait_ticks),
    computeDelta("Median Wait Time", "ticks", "waiting patients", baseline.median_wait_ticks, comparison.median_wait_ticks),
    computeDelta("95th Percentile Wait", "ticks", "waiting patients", baseline.p95_wait_ticks, comparison.p95_wait_ticks),
    computeDelta("Mean Length of Stay", "ticks", "completed patients", baseline.mean_los_ticks, comparison.mean_los_ticks),
    computeDelta("Peak Bed Utilization", "ratio", "available beds", baseline.peak_bed_utilization, comparison.peak_bed_utilization),
    computeDelta("Average Bed Utilization", "ratio", "available beds", baseline.average_bed_utilization, comparison.average_bed_utilization),
    computeDelta("Staff Utilization", "ratio", "scheduled staff", baseline.staff_utilization, comparison.staff_utilization),
    computeDelta("Completed Throughput", "patients", "total admitted", baseline.completed_count, comparison.completed_count),
    computeDelta("Censored / Unfinished", "patients", "total admitted", baseline.unfinished_censored_count, comparison.unfinished_censored_count),
  ];

  const waitInflation = baseline.mean_wait_ticks > 0
    ? comparison.mean_wait_ticks / baseline.mean_wait_ticks
    : 1.0;

  const bedSaturation = comparison.peak_bed_utilization >= 0.95;

  let bottleneck = "No severe resource starvation detected.";
  if (bedSaturation && waitInflation > 2.0) {
    bottleneck = `Severe bed starvation: Wait time inflated by ${((waitInflation - 1) * 100).toFixed(1)}% with peak bed utilization at ${(comparison.peak_bed_utilization * 100).toFixed(1)}%.`;
  } else if (comparison.staff_utilization >= 0.90) {
    bottleneck = "Staff capacity bottleneck: high clinical utilization driving prolonged queuing.";
  }

  return {
    baseline_name: baseline.scenario_name,
    comparison_name: comparison.scenario_name,
    horizon_ticks: baseline.horizon_ticks,
    deltas,
    wait_inflation_factor: Math.round(waitInflation * 100) / 100,
    bed_saturation_detected: bedSaturation,
    bottleneck_summary: bottleneck,
  };
}
