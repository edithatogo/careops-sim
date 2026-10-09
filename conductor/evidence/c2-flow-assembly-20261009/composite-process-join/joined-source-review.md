# Joined source review

Source `58827f30fde5bc4330bc38eff1875470592b74e0`, tree
`175d44d7812a1e04df873be74cdfc6b1e6173578`, clean checkout.

Provider reviewer cross-reviewed the generic-context fixture and broader lifecycle
coverage. Component reviewer cross-reviewed the derived-identity/nonzero RNG fixture
and composite transaction. Both found no blocking issue. Fidelity writer supplied
supplemental matrix/CI verification and manual accounting; this is not represented
as an independent review of its own matrix implementation. Root inspected the
integration, executed the locked join and separately reconciled actual artifacts.
All three reviews were read-only, with no new tests claimed by the reviewers.

The generic fixture restores a context changed by an actual callback, three decoded
context rows, two concrete context/template types and two distinct factory IDs.
Binding/cap rejection precedes caller decode; a later resolver failure preserves
the original source. The native bridge contract still requires template `T: Clone`.
Representative payloads exercise generic protocols, not exhaustive payload fuzzing.

The RNG child reconstructs trusted fixture code/schema descriptors, graph canonical
bytes, provider strata, registry and keys before image load. The actual Bound stream
is at cursor 2 before capture, then continued through u64, weighted sample, u32 and
u64 draws; child/control outputs and post-continuation full-image digests match.
This dedicated RNG proof is one transit-ready case. Separate native provider tests
and the 18-cut process matrix retain their own scope.

The 18 cuts cover all declared legal lifecycle classes, including actual intermediate
Suspend/Restart checkpoints and rejected Start/Progress retries. A stale rejected
control cut rejects capture and preserves its source, rather than becoming a
runnable image. Artifacts contain 20 real child-process inputs, logs and invocation
records; hashes and actual/control traces independently reconcile.

The root fixes test module paths so clean checkouts need no untracked local tests
directory. The CI source-hash paths now exist in the joined tree. No full C2,
new-head hosted pass, release, clinical or C5 acceptance is inferred here.
