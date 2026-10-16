/**
 * CareOps ED Scenario Exporter (`@careops/dashboard/scenario`)
 *
 * Multi-format export utilities: JSON run documents, CSV tabular summaries,
 * and Arrow-compatible column schema representations.
 */

import { ScenarioRunMetrics, ScenarioComparisonResult } from "./scenario_runner.js";

/**
 * Serializes run metrics into a JSON run document conforming to CareOps ED export schema.
 */
export function exportRunDocumentJson(metrics: ScenarioRunMetrics): string {
  return JSON.stringify(
    {
      schema_version: "careops_ed.run_document.v1",
      exported_at_utc: new Date().toISOString(),
      run_metrics: metrics,
    },
    null,
    2
  );
}

/**
 * Formats scenario comparison metrics as a CSV document.
 */
export function exportComparisonCsv(comparison: ScenarioComparisonResult): string {
  const lines: string[] = [
    `# CareOps ED Scenario Comparison: ${comparison.baseline_name} vs ${comparison.comparison_name}`,
    `# Horizon Ticks: ${comparison.horizon_ticks}`,
    `# Wait Inflation Factor: ${comparison.wait_inflation_factor}x`,
    `# Bottleneck: ${comparison.bottleneck_summary}`,
    "metric_name,unit,denominator,baseline,comparison,absolute_delta,percent_change,statistically_significant,interpretation",
  ];

  for (const delta of comparison.deltas) {
    const row = [
      `"${delta.metric_name}"`,
      `"${delta.unit}"`,
      `"${delta.denominator}"`,
      delta.baseline_value,
      delta.comparison_value,
      delta.absolute_delta,
      delta.percent_change,
      delta.is_statistically_significant,
      `"${delta.interpretation}"`,
    ];
    lines.push(row.join(","));
  }

  return lines.join("\n");
}

/**
 * Generates an Apache Arrow-compatible schema description for tabular run telemetry.
 */
export function getArrowTelemetrySchema(): Record<string, unknown> {
  return {
    format: "arrow-ipc-schema-v1",
    fields: [
      { name: "tick", type: "int64", nullable: false },
      { name: "entity_id", type: "int64", nullable: false },
      { name: "entity_kind", type: "utf8", nullable: false },
      { name: "location_id", type: "utf8", nullable: false },
      { name: "x_mm", type: "int64", nullable: false },
      { name: "y_mm", type: "int64", nullable: false },
      { name: "floor_level", type: "int32", nullable: false },
      { name: "state", type: "utf8", nullable: false },
      { name: "provenance", type: "utf8", nullable: false },
    ],
    metadata: {
      "careops.abi.version": "careops_ed.abi.v1",
      "careops.spatial.version": "careops_ed.spatial.v1",
    },
  };
}
