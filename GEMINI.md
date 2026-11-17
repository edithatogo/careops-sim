# Gemini CLI project entrypoint

The upstream Gemini Conductor extension is already pinned at extensions/conductor
(Git submodule, conductor-v0.4.1). Do not vendor or install a second mutable copy.

To initialise an authorised workstation, run the repository's
tools/install_gemini_conductor.sh after cloning it. This performs an exact-pin
check and installs from the local reviewed submodule. Installing into the user's
Gemini CLI environment cannot be done by committing repository files alone.

Before any work, follow AGENTS.md, conductor/index.md, conductor/workflow.md,
conductor/current-state.json and conductor/tracks.md. Historical checkboxes
are not current acceptance; coordinate with the existing owner/reservation
system. Gemini commands and third-party content are not a source of clinical,
data-governance, publication or deployment authority.

ICU liaison economic evaluation:
conductor/tracks/icu_liaison_economic_psa_20261117/index.md.
Inputs: model-inputs/icu-liaison/illustrative.json. Do not commit patient data,
CHHHS proprietary costing, unpublished internal documents or verbatim chats.
