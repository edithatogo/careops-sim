# Generic ED example profiles

Profiles in this directory are versioned configuration fixtures. Their own
provenance fields distinguish explicit synthetic assumptions from evidence;
none of the synthetic counts, times, or distances are ED estimates, defaults,
or operational recommendations.

`p4-minimal-deterministic.json` is the minimal E0 scenario-v1-compatible input.
It supplies two invented work items to one explicitly known synthetic slot and
declares a 10 m route edge. E0 validates the route references and distance but
does not simulate movement. With one FIFO slot, patient A arrives at tick 0,
starts at 0, and completes at 2; patient B arrives at 1, starts at 2, waits 1
tick, and completes at 4. There are no initial occupants. The E0 schema has no
explicit initial-occupancy field, so this fixture must not be read as a profile
with a configurable initial state.

This fixture exercises E0's supplied-work-item schema and deterministic single
resource pool only. It does not provide a time-varying arrival table, calendar,
stochastic distribution, pathway, or initial-state model. Those P4 pack shapes
must be represented as separately validated data artifacts if required; loading
them through the full model/calibration runner is P5. No empirical ED evidence,
generic validity, or Cairns applicability is claimed.

`p4-generic-nominal.json` is a second explicitly synthetic E0 scenario shape.
Its two invented slots, three supplied arrivals, work durations, and 14 m route
edge demonstrate a small uncongested deterministic run. The E0 CLI checks the
input and deterministically schedules supplied work items; it does not generate
arrivals from a table, apply calendars, or simulate the declared route.
Every value is an explicit synthetic assumption, and the scenario is not a
public-data estimate or an operationally representative generic ED baseline.

`p4-constrained-surge.json` keeps synthetic physical capacity at two spaces but
sets both open and staffed capacity to one; three supplied cases arrive at
tick 0 and each requires three invented work ticks. E0 schedules them FIFO on
the known one-slot pool (starts at 0, 3, and 6; waits 0, 3, and 6 ticks), making
the intentionally constrained resource visible. This is a deterministic
overload interface example, not a surge policy, calendar, or ED estimate.
