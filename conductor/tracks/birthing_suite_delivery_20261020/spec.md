# Specification: generic birthing suite composition

## Purpose

Deliver a standalone configurable birthing model as the second composition
consumer. Generic/synthetic fixtures precede separately qualified jurisdiction
and site profiles. [CC-01–CC-08](../../design/composition-contracts-20261020.md)
define shared contracts.

## Requirements

- B-R1: source/version/units/provenance for perinatal distributions and policy targets, retaining missing evidence and correlation/identifiability limits. Earlier numeric examples are hypotheses, not accepted constants.
- B-R2: distinct patient and service state with competing progression/intervention/deterioration during waits and transport; explicit draw ownership and stale-event handling.
- B-R3: births through vaginal and operative routes, multiple infants, typed relationships and distinct person/episode IDs; neonatal observations and independent maternal/infant trajectories.
- B-R4: rooms/equipment declared through capabilities with staffing, opening, reservation, turnover and boarding constraints.
- B-R5: profile-defined staffing ratios, role eligibility, breaks, handovers, call-in latency and shared-team bundles; no double booking or lost responsibility.
- B-R6: policy-defined escalation and safe interruption, with target breaches observable. Bounded theatre adapter returns procedure-start and birth/completion events so transfer is not mistaken for delivery.
- B-R7: complete transfer lifecycle for maternal, neonatal, theatre, ward and retrieval destinations; cancellation, expiry, retries and split destinations conserve people/resources.
- B-R8: batch CLI, independently readable Arrow IPC/Parquet, replay/worker/recovery equivalence and delay/occupancy/target metrics with provenance and uncertainty.

## Acceptance and exclusions

Qualify B0 evidence separately from core implementation. B1 consumes A5; B7 joins
with the ED composition after A6. A7 is the shared acceptance join, avoiding a
circular dependency. Full theatre and postnatal models remain later work; a bounded
theatre response model is required now for decision-to-delivery metrics.
No fixed jurisdiction-wide clinical threshold, clinical prediction or operational
acceptance follows from synthetic tests.
