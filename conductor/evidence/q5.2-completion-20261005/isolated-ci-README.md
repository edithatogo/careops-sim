# Isolated Q5.2 CI evidence archive

This local archive preserves the completed PR and push CI readbacks and their downloaded artifact folders. It is a byte-integrity catalog of CI evidence, not a Q5.2 acceptance decision, source change, pin update, or release approval.

Both run readbacks identify commit `8daa0978578b8a5b5b6427e84db1a3e6c54a1123`, report `success`, and contain 12 successful jobs. Each side has 15 downloaded artifact directories. The bundle retains their logs, JSON readbacks/results, and synthetic Arrow output fixtures. It excludes build targets, compiled binaries/libraries/objects, symlinks, and private session/lease/token/store files.

`inventory.json` lists every copied file by relative archive path, byte count, and SHA-256. `q52-isolated-ci.tar.gz` uses deterministic sorted members and normalized tar/gzip metadata. No broader queue acceptance is inferred from workflow conclusion alone; canonical gate result files remain evidence to review separately.
