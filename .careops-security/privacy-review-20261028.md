# Current-source privacy review — 28 October 2026

Personal workstation prefixes were replaced with explicit private-location aliases
in 238 text files and four archived evidence copies. The exact original and
redacted digests are recorded in privacy-redactions.json. Unredacted originals
are retained privately; Git history is assessed separately and is not rewritten.
Historical manifest digests refer to originals. Edited copies cannot be used as
byte-identical execution receipts or a new runtime qualification. No runtime
result, acceptance claim, submodule pin or operational binding was changed.

Six hosted-runner-only text paths are false positives for personal workstation
disclosure. They describe disposable hosted CI runners. The detector remains
strict; no directory or pattern is globally ignored. Generic-key alerts refer
to source file hashes, a public API digest not service authentication. Two historical local advisory lease tokens were
also redacted as unnecessary capability breadcrumbs; they grant no remote service
access.

All 120 standalone Arrow/Parquet sidecars were independently logically decoded
with the existing qualified Python 3.15.1 / PyArrow 26.0.0 reader: 744 rows, all
columns and schema metadata, full validation. These are synthetic calibration
fixtures; JSON companions and generator provenance agree. Each binary approval
is bound to the exact reviewed file hash, not a wildcard or format exemption.
All 726 archive-contained Arrow/Parquet sidecars were also logically decoded
and fully validated (5,865 rows). Archives were inspected offline without executing
their contents. Encryption is
outside this review. Current-source cleanup does not erase prior publication.
