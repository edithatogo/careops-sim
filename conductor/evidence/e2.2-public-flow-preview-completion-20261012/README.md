# Public Flow preview qualification evidence

Kairos source head: `5c57dc807cf6a6524d174adbb88502a92cbf5673`; parent base: `ebca26da62c53265c49679a3c5fc5a2180aaa5ad`; parent Kairos pin remains `6d733f489225da4bf6ceec2673b02369c7918bbe`.

The archive preserves the source-stage packet and its four captured gate attempts (including the one failed Clippy attempt), plus the clean-head full qualification packet and its three successful final gates. The final test log is checked against the packet driver’s inherited requirements and all seven public-preview proofs; its actual suite counts are derived from the log. All 26 child process inputs, exits, raw hashes and decoded traces are independently matched. Nineteen literal source golden hashes are checked against their raw checkpoint artifacts. Fourteen frozen source/document hashes are captured.

Prior V3 consumer evidence is referenced in place at `conductor/evidence/e2.2-v3-consumer-completion-20261012`; its 293-file history is not copied again. The parent pin is unchanged. Next work is queue-order and ABM contract/native work, versioned persistence, and joined consumer qualification before pin update and ED integration.
