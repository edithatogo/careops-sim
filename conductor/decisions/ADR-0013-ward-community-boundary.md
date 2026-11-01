# ADR-0013: Minimal ward-to-community boarding boundary

- Status: accepted after reviewer-agent review
- Date: 2026-10-27
- Scope: CareOps A4.3 synthetic boundary model

## Context

The A4 acceptance fixture must show that a ward patient's clinical readiness for
community discharge does not itself release the ward bed. A confirmed community
place and available transport are separate constraints. Until the patient
actually departs and handover completes, the ward bed claim can delay an
emergency patient who is ready for admission. The model needs to expose this downstream-to-upstream
boarding feedback without adding clinical policy or a full ward/community
simulator.

## Decision

Implement a deterministic single-bed, single-episode boundary fixture in
`careops-core`. The fixture receives exogenous virtual-time observations:

- the ward patient's clinically-ready tick;
- the emergency patient's ready-for-ward tick;
- an optional tick at which a confirmed community place is available;
- an optional tick at which transport pickup is available;
- transport duration in integer ticks;
- an optional nonnegative handover-acceptance delay after community arrival; and
- an inclusive simulation horizon.

Both readiness ticks must be at or before the horizon. A community-place or
pickup tick may be absent or beyond the horizon to represent unavailability
within the observed window. No wall clock, random draw, local clinical threshold,
or inferred destination suitability is used.

The earliest actual ward departure is the maximum of ward clinical readiness,
community-place availability, and transport-pickup availability, provided both
availability times exist and the resulting departure is at or before the
horizon. A readiness event alone never changes bed state. Physical occupancy of
the ward bed ends at actual departure, but its origin resource claim remains
held through travel, as required by ADR-0012. Arrival and handover completion are
separate events. The optional acceptance delay is added to arrival with checked
integer arithmetic; `None` means handover is not completed within the modeled
window. The origin bed claim becomes allocatable only at handover completion.
The emergency patient is admitted at the later of their ready-for-ward tick and
origin-claim release; their boarding duration is admission minus readiness. If
no departure occurs by the horizon, the ward patient still physically occupies
the bed. After departure the bed is physically vacant while its origin claim
remains held through travel and any pending handover, so the emergency patient
remains boarded until that claim is released or the horizon is reached.

At actual departure, community arrival is departure plus the supplied transport
duration. Use checked integer addition. An arrival exactly at the inclusive
horizon is observed. Handover completion occurs at arrival plus the acceptance
delay, when provided, and only if that tick is at or before the horizon. Thus an
arrival at the horizon can be observed while handover remains pending. Transport
duration zero is valid. The outcome reports ward physical-occupancy ticks,
origin-claim hold ticks, emergency boarding ticks, departure/admission/arrival/
handover ticks when observed, physical occupancy and claim state at the horizon,
and one of `waiting_for_community`, `waiting_for_transport`,
`waiting_for_both`, `in_transit`, `handover_pending`, or `handover_complete`.

If destination-place or pickup timing prevents departure, the waiting status
identifies which exogenous constraint is unavailable by the horizon. This is a
single case with one occupied ward bed and one waiting emergency encounter; it
does not allocate or contend for multiple beds, reserve or expire community
capacity, or simulate a transport fleet. A6 may compose the boundary with
`TransferManager`; this ADR does not duplicate transfer ownership, claims,
message handling or compensation.

## Consequences

- Tests can show delayed community availability or transport increasing ward
  boarding and, while the single bed is occupied, emergency boarding.
- Clinical readiness, actual departure, physical bed vacancy, origin-claim
  release/handover, emergency readiness/admission and transport arrival remain
  separate observable times.
- All times are synthetic integer ticks. This contract makes no claims about
  clinical validity, transport distributions, site policy or operational
  performance.
- Multi-bed contention, capacity reservation, competing arrivals, exogenous
  event scheduling and full ED integration remain later work.

## Required oracle

The focused suite must cover downstream delay feedback, availability before
readiness, missing/late community place and pickup, inclusive-horizon departure,
arrival and handover, arrival with pending handover at the horizon, in-transit
origin-claim retention, zero-duration transport, invalid readiness beyond
horizon, checked-add overflow, and conservation of the
one physical ward bed, its held/released origin claim, and emergency encounter
in every outcome. Deterministic replay of an identical input must produce an
equal outcome.
