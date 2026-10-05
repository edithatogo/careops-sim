# ADR-0008 — C2 test-first preparation and runtime join

Date: 2026-10-05. Status: reviewed architecture; implementation and acceptance open.
Owners: parent coordinator; Kairos Track03/21/01/22/12/25 boundaries preserved.

## Problem and evidence

C2.1 requires executable common-random-number Macro/zero-transit Micro oracles.
Its provider is implemented by C2.2 and real transit observation by C2.3. The
original serial DAG made C2.2 wait for accepted C2.1; accepting the latter against
actual runtime was impossible before its own implementation prerequisites.
Independent gpt-6-luna source review confirmed Flow currently requires an upfront
duration and contains no transit API; the private mode adapter only resolves
admission decisions. Comparing fixed durations or disconnected seed streams
would not meet the original oracle. No such shortcut is accepted.

## Decision

Add C2.0 for frozen interfaces and reviewed test-first fixtures, explicitly
separating test-definition acceptance from runtime acceptance. Plan order becomes
C2.0 -> C2.2 -> C2.3 -> C2.1 -> C2.4. Original C2.1 text/oracles and final phase
requirements are preserved. No task is marked complete by this decision. C2.0
is coordinator-owned; bounded Luna workers may implement settled test packets.

C2.0's paired tests may have expected missing-API compile-red evidence, which
proves only the test-first boundary. Code review must establish intended actual
Flow/provider/transit assertions. C2.2 supplies the real empirical work provider,
policy and stream integration; C2.3 supplies deterministic routes, progress and
observable transit. C2.1 joins actual paired executions and every lifecycle test
after those implementations. Both native hosted gates and independent review
remain required. Existing mode source can be reused but is not an accepted C2.1
leaf before its new runtime prerequisites. Each future packet must be rebound.

## Preserved requirements and ownership

Macro emits no transit and uses no Transit draw; zero-transit Micro matches its
paired Macro service draws and canonical outcomes; nonzero transit and Behavior
use their own purpose streams; Suspend retains remaining/sample and Restart
reuses its original service sample. Switching never drops active/suspended work.
Providers separate work from wait/transit and preserve mode/owned stream state.
In-memory snapshot proof is distinct from Track22's portable codec/rebinding gate.
No DES-to-calibration dependency, new backend, renderer, private patient input or
runtime dependency is authorized by this planning edit. Native engine1.76 and
optional calibration1.88 floors, Rust1.99 canonical toolchain, stable release
baseline hold, Q5.2/C1.4 thresholds and parallel Track49 are unchanged.

## Machine scheduling and review

Task catalog derives sibling edges from the new plan order. MVP recipes retain
all original runtime leaves and add bounded C2.0 mode/interface/red-test leaves.
C2.1 cannot be prepared before accepted C2.3. Recipe changes are prospective;
old reviewed work remains source evidence and must be rebound for future dispatch.
No generated catalog, context file, red result or claim grants execution authority.

Independent c21_paired_preparation reviewed the source gap and generic DAG/recipe
rules read-only; confirmed numeric ID order is not execution order. Coordinator
must validate the rebuilt DAG and complete E2.4 dependency closure before commit.
