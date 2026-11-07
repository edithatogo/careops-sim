# Material risks

The current provider branch may expose APIs absent on main; verify actual pins.
Movement cancellation can leak leases; usage replay can double-count; economic
units/perspectives can be mixed. The specific negative cases and stopping rules
are in [plan.md](plan.md). Do not create another risk dossier per leaf.
