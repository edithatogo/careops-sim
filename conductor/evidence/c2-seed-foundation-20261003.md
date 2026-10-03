# C2 seed-map foundation integration — partial

Exact Kairos76c04dc2ae2280deb8ec342f9f1ffc139929323c, owner CI
https://github.com/edithatogo/kairos/actions/runs/37089725060 successful on native
Linuxx86_64 and macOSARM64. Both tested current reusable owners including Arrow
and new calibration package with Rust1.98.1 and separate calibration1.88 floor.
Child compiler paths and host assertions are explicit. Parent pin/contract hashes
match these exact committed bytes. Source RNG unchanged.

Local seed qualification13 tests actual1.88 and1.98; all four cargo-deny categories
passed, targeted Clippy -Dwarnings and fmt passed. Normative independent SHA/
seed/draw goldens, identifier/framing/collision/continuation guards reviewed.
Repeated register returns fresh stream; single advancing-owner remains caller
responsibility. Reordered/interleaved draw test is not worker-count execution.
No portablecodec or global64bit collision-free promise. See upstream frozen
seed-map and qualification docs for exact API/limits.

Full C2.1–C2.4 stay unchecked: mode/providers, transit/fidelity adapters and real
simulation-qualified outcomes remain. No calibrated prediction, release readiness
or unknown local clinical mapping follows from this foundation. Parent hosted CI
remains required at this integration commit before merge.
