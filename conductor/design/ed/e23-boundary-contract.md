# E2.3 Boundary Contract: ST001–ST009 Acceptance Cases

Status: approved-contract
Date: 2026-10-12
Authority: coordinator contract for Generic ED leaf `E2.3.boundary` (ST001–ST009 acceptance cases covering compatible/expired offers, retained occupancy/care, offload sensitivity, and explicit limits).

## 1. Scope and Oracles

Implements formal integration tests for Report 29 cases ST001 through ST009:
- **ST001 / ST002 (Compatible & Expired Offers)**:
  - Compatible offers allocate and transition to Consumed upon transfer.
  - Expired / unused offers do not allocate and leave boarding patients in waiting state without space leaks.
- **ST003 / ST004 (Occupancy Retention & Recurring Care Claims)**:
  - Space is retained throughout physical departure delay.
  - Recurring / episodic boarder care work items do not drop or corrupt space ownership.
- **ST005 / ST006 (Ambulance Arrival / Receiver Staff Sensitivity)**:
  - Offload delay sensitivity: Arrival with constrained triage/assessment capacity queues deterministically.
- **ST007 / ST008 (Multi-person Procedure Rejection & Composite Approximation)**:
  - Concurrent multi-person procedures fail closed with unsupported status unless modelled as dedicated composite teams.
- **ST009 (Causal Ward Boundary Limitation)**:
  - Synthetic external offer streams do not establish causal ward/fleet policies.
