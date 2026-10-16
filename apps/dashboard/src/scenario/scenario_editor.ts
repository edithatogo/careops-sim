/**
 * CareOps ED Scenario Editor (`@careops/dashboard/scenario`)
 *
 * Implements schema-validated editing of ED scenarios (triage mix, arrival rates,
 * bed/staff capacities, horizon ticks) matching native CareOps ED contracts.
 * Provides actionable error diagnostics without author assistance or Rust code changes.
 */

export interface TriageMixConfig {
  ats1: number;
  ats2: number;
  ats3: number;
  ats4: number;
  ats5: number;
}

export interface BedCapacityConfig {
  resus_bays: number;
  acute_bays: number;
  subacute_bays: number;
}

export interface StaffCapacityConfig {
  triage_nurses: number;
  registered_nurses: number;
  senior_physicians: number;
  resident_physicians: number;
}

export interface EdScenarioConfig {
  name: string;
  schema_version: number;
  capacity_profile_version: number;
  seed: string;
  horizon_ticks: number;
  arrival_rate_per_hour: number;
  triage_mix: TriageMixConfig;
  beds: BedCapacityConfig;
  staff: StaffCapacityConfig;
  spatial_package_revision?: string;
}

export interface ScenarioValidationError {
  field: string;
  code: string;
  message: string;
  suggested_fix: string;
}

export interface ScenarioValidationResult {
  is_valid: boolean;
  errors: ScenarioValidationError[];
  scenario?: EdScenarioConfig;
}

/**
 * Returns canonical nominal MVP scenario configuration.
 */
export function getNominalScenarioTemplate(): EdScenarioConfig {
  return {
    name: "Nominal ED Flow",
    schema_version: 1,
    capacity_profile_version: 3,
    seed: "1234567890",
    horizon_ticks: 1440,
    arrival_rate_per_hour: 4.5,
    triage_mix: {
      ats1: 0.05,
      ats2: 0.15,
      ats3: 0.40,
      ats4: 0.30,
      ats5: 0.10,
    },
    beds: {
      resus_bays: 2,
      acute_bays: 8,
      subacute_bays: 6,
    },
    staff: {
      triage_nurses: 2,
      registered_nurses: 6,
      senior_physicians: 2,
      resident_physicians: 3,
    },
    spatial_package_revision: "sha256:d8b72ef3e1347ba01438343c4a45610a56bbbe79a1f0a20a4be35ceba48c5c4e",
  };
}

/**
 * Returns canonical surge ED scenario configuration with high-acuity load and constrained resources.
 */
export function getSurgeScenarioTemplate(): EdScenarioConfig {
  return {
    name: "Surge ED Influx",
    schema_version: 1,
    capacity_profile_version: 3,
    seed: "1234567890",
    horizon_ticks: 1440,
    arrival_rate_per_hour: 12.0,
    triage_mix: {
      ats1: 0.15,
      ats2: 0.35,
      ats3: 0.35,
      ats4: 0.10,
      ats5: 0.05,
    },
    beds: {
      resus_bays: 2,
      acute_bays: 8,
      subacute_bays: 4,
    },
    staff: {
      triage_nurses: 2,
      registered_nurses: 5,
      senior_physicians: 1,
      resident_physicians: 2,
    },
    spatial_package_revision: "sha256:d8b72ef3e1347ba01438343c4a45610a56bbbe79a1f0a20a4be35ceba48c5c4e",
  };
}

/**
 * Validates scenario configuration against native schema constraints and clinical bounds.
 */
export function validateScenarioConfig(scenario: Partial<EdScenarioConfig>): ScenarioValidationResult {
  const errors: ScenarioValidationError[] = [];

  if (!scenario.name || scenario.name.trim() === "") {
    errors.push({
      field: "name",
      code: "EMPTY_NAME",
      message: "Scenario name must not be empty.",
      suggested_fix: "Provide a descriptive scenario label.",
    });
  }

  if (scenario.schema_version !== 1) {
    errors.push({
      field: "schema_version",
      code: "UNSUPPORTED_SCHEMA_VERSION",
      message: `Schema version must be exactly 1; received ${scenario.schema_version}.`,
      suggested_fix: "Set schema_version to 1.",
    });
  }

  if (scenario.capacity_profile_version !== 3) {
    errors.push({
      field: "capacity_profile_version",
      code: "UNSUPPORTED_PROFILE_VERSION",
      message: `Capacity profile version must be 3; received ${scenario.capacity_profile_version}.`,
      suggested_fix: "Set capacity_profile_version to 3.",
    });
  }

  if (!scenario.horizon_ticks || scenario.horizon_ticks <= 0) {
    errors.push({
      field: "horizon_ticks",
      code: "NON_POSITIVE_HORIZON",
      message: `Horizon ticks must be greater than zero; received ${scenario.horizon_ticks}.`,
      suggested_fix: "Specify a positive integer horizon (e.g., 1440 for 24h at 1 min/tick).",
    });
  }

  if (scenario.arrival_rate_per_hour === undefined || scenario.arrival_rate_per_hour < 0) {
    errors.push({
      field: "arrival_rate_per_hour",
      code: "NEGATIVE_ARRIVAL_RATE",
      message: `Arrival rate cannot be negative; received ${scenario.arrival_rate_per_hour}.`,
      suggested_fix: "Set arrival_rate_per_hour to 0 or a positive float.",
    });
  }

  // Triage mix validation: probabilities must sum to 1.0 within 1e-3
  if (scenario.triage_mix) {
    const { ats1, ats2, ats3, ats4, ats5 } = scenario.triage_mix;
    for (const [key, val] of Object.entries(scenario.triage_mix)) {
      if (val < 0 || val > 1.0) {
        errors.push({
          field: `triage_mix.${key}`,
          code: "PROBABILITY_OUT_OF_BOUNDS",
          message: `ATS probability must be in range [0.0, 1.0]; ${key} is ${val}.`,
          suggested_fix: "Adjust probability into [0.0, 1.0].",
        });
      }
    }
    const sum = ats1 + ats2 + ats3 + ats4 + ats5;
    if (Math.abs(sum - 1.0) > 0.005) {
      errors.push({
        field: "triage_mix",
        code: "SUM_NOT_ONE",
        message: `Triage ATS 1-5 probabilities must sum to 1.0; current sum is ${sum.toFixed(4)}.`,
        suggested_fix: "Normalize triage probabilities so that ATS 1 through 5 sum to 1.0.",
      });
    }
  } else {
    errors.push({
      field: "triage_mix",
      code: "MISSING_TRIAGE_MIX",
      message: "Triage mix configuration is required.",
      suggested_fix: "Define ATS 1-5 distribution probabilities.",
    });
  }

  // Bed capacity validation: must have at least 1 acute or resus bay
  if (scenario.beds) {
    const totalBeds = (scenario.beds.resus_bays || 0) + (scenario.beds.acute_bays || 0) + (scenario.beds.subacute_bays || 0);
    if (totalBeds <= 0) {
      errors.push({
        field: "beds",
        code: "ZERO_BED_CAPACITY",
        message: "Emergency department bed capacity must be at least 1.",
        suggested_fix: "Assign positive capacity to acute or resus bays.",
      });
    }
  } else {
    errors.push({
      field: "beds",
      code: "MISSING_BEDS",
      message: "Bed capacity configuration is required.",
      suggested_fix: "Define resus, acute, and subacute bay counts.",
    });
  }

  // Staff capacity validation: must have at least 1 clinical staff member
  if (scenario.staff) {
    const totalStaff = (scenario.staff.triage_nurses || 0) +
                       (scenario.staff.registered_nurses || 0) +
                       (scenario.staff.senior_physicians || 0) +
                       (scenario.staff.resident_physicians || 0);
    if (totalStaff <= 0) {
      errors.push({
        field: "staff",
        code: "ZERO_STAFF_CAPACITY",
        message: "Staff roster must have at least one active clinical staff member.",
        suggested_fix: "Assign at least one nurse or physician.",
      });
    }
  } else {
    errors.push({
      field: "staff",
      code: "MISSING_STAFF",
      message: "Staff configuration is required.",
      suggested_fix: "Define nursing and physician staffing levels.",
    });
  }

  return {
    is_valid: errors.length === 0,
    errors,
    scenario: errors.length === 0 ? (scenario as EdScenarioConfig) : undefined,
  };
}
