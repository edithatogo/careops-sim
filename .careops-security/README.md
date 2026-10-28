# Privacy protection

This standalone gate uses Python 3.15.1 standard library and Gitleaks 8.30.1.
No PII service, model download, telemetry or additional Python package is needed.
CI uses read-only permissions, SHA-pinned actions, a hash-verified scanner and no
cross-repository credentials. It uploads no scan reports or source artifacts.

Create a separate local environment with
`uv venv --python 3.15.1 .venv-careops-privacy`, verify its actual version with
`.venv-careops-privacy/bin/python --version`, then run
`sh .careops-security/install.sh`. Existing hook configuration is preserved;
chain these hooks explicitly when another hook is already installed. Local hooks
are bypassable and must be paired with the required CI check `Privacy and secrets`.
The pre-push hook audits all locally reachable history and all outgoing commit content;
legacy findings deliberately block pushing until resolved or specifically reviewed.

Pre-commit inspects staged bytes, preserving partial staging. CI scans changed
files and intermediate commit blobs against the PR/push base, validates the entire tracked-file inventory and
scans full Git history for secrets. Manual dispatch audits all tracked content.
Run the full local privacy audit with
`.venv-careops-privacy/bin/python -I .careops-security/check.py --all`.
Submodules require their own gate; gitlinks are not recursively inspected.

`PUBLIC_SOURCE_FILES.json` remains authoritative where present. Otherwise
`inventory.json` records the initial tracked paths as a continuity baseline,
not a privacy approval of legacy content. Add/remove exact paths when changing
source; review inventory changes with the code. Staging an inventory update
cannot approve sensitive text or credential files.

The scanner rejects organisation health email addresses, private workstation
paths, labelled numeric patient identifiers, private keys and common credential
filenames. It requires review for documents, archives, images and other binary
files, and refuses files above 5 MiB. It logs rules and escaped paths only.
These patterns do not identify every name, person, health record or confidential
fact. Fictional text and structured CSV/JSON still require ordinary review.

Before adding a document or binary, inspect its contents and metadata locally,
including comments, tracked edits, hidden sheets and embedded objects. Record an
exact path, SHA-256 and substantive review reason under `reviewedArtifacts` in
`policy.json`. A changed byte requires renewed review. No legacy artifacts are
automatically approved. Large files need a separately reviewed policy change.
No document parsers execute on untrusted attachments in this baseline.

For a solo developer, require passing CI without a mandatory second reviewer;
keep force pushes/deletion blocked and an owner recovery route. Prefer passkeys
or hardware-backed MFA, narrowly scoped expiring tokens, OIDC for supported CI
deployments, and short artifact retention. Review collaborator/app access and
workflow changes. Keep real exports, auth state and private receipts outside Git.
An exposed credential needs revocation/rotation; private visibility and history
cleanup cannot retract copies already downloaded.
