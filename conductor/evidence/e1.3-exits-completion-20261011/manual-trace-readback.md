# Actual abandonment trace readback

Root inspected the two actual JSON records in the combined native log, extracted
without substituting expected arrays. Source/command/log hashes are in native.json.

| Scenario / patient | Actual event reconciliation | Outcome |
| --- | --- | --- |
| Outside / p1 | arrival1, triage1–3, assessment3–6, treatment6–11, space1–H20 | boarding, unfinished, occupancy19 |
| Outside / p2 | arrival1, deadline2, no acquired space or clinical episode | abandoned2, LOS1, occupancy0 |
| Queued / p1 | arrival1, triage1–3, assessment startH3 | unfinished, zero assessment work, space occupancy2 |
| Queued / p2 | arrival1, acquired space1, triage queued without start, deadline2 releases space | abandoned2, LOS1, occupancy1 |
| Queued / p3 | arrival2, acquired released space2, triage starts at deadlineH3 | saved from abandonment, unfinished, zero triage work, occupancy1 |

Outside conservation is2=1abandoned+1unfinished; queued conservation is3=
1abandoned+2unfinished. No abandoned patient acquires staff or reports fabricated
clinical completion. Historical unstarted triage data is explicitly distinguished
by Terminal/Abandoned and terminal tick. This is the optional abandonment trace
only; full acuity/pathway grid and generator composition remain E1.4 obligations.
