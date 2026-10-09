# Hosted component foundation qualification

Published headfad0380 in draftPR243:65successful checks,two conditional skips,no
failures/pending. The current CareOps native run37915988082 succeeded on both hosts;
actual downloaded C2 runtime outputs show472passed,0failed,5ignored for the
DES/ABM/calibration command on each host. Core/state component tests are separately
confirmed in the successful reusable-owner-package logs. The local five-package
join includes those packages and has527passes,0failures,5ignores/54groups.

GitHub uses a synthetic merge checkout. Commit identity API confirms each tested
checkout has the reviewed PR head as parent and exactly the same Git tree; actual
logs and commit objects are retained. Repeated sealed replay suffixes are exact
within and across hosts; sentinel attempts return1 and leave bytes unchanged.
This qualifies component interfaces and existing runtime regressions, not a byte
codec/full Flow restart. The complete checkpoint work continues in current-state.
