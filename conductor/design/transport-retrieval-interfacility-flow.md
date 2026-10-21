# Transport and inter-service boundaries

The same transfer protocol supports within-hospital movement, interfacility
escalation/repatriation, community/public/private interfaces and cross-region
flows. Provider names and local permissions belong in versioned profiles.
Earlier unsourced speeds, radii, crew-category mappings and dispatch targets are
candidate assumptions awaiting profile validation, not runtime defaults.

## Network and transport profiles

Use facility/service IDs, directed routes, mode capabilities, capacities, travel
laws, opening/operating windows, crew roles and escort requirements. Road, rotary
and fixed-wing transport, non-acute vans, public/commercial transport and private
vehicles differ in capability, capacity and availability. Derive eligibility from
required care and service permissions, not an ATS-to-crew lookup.

Model dispatch, mobilisation, pickup, travel, arrival/offload, handover, cleaning
and crew/vehicle return explicitly where material. Weather, outages and geography
may share scenario-level drivers; independent travel draws are an assumption to
justify. A destination reservation is not an available staffed bed forever.

## Transfer protocol

Implement CC-02: request → offer/reservation → departure → transit → arrival →
handover completion, with rejection, cancellation, expiry and compensating routes.
Retain operation IDs and one authoritative owner. Define origin bed/escort release
and destination reservation/occupancy separately. Test retries, late messages,
rejection after departure and horizon censoring without losing or duplicating people.

## Healthcare interfaces

- Hospital escalation and repatriation, including rural and tertiary facilities.
- Public/private contracted activity and deteriorating-patient transfers, subject to receiving-service acceptance and capability.
- Primary-care/private-specialist referrals, direct admission and shared antenatal care, with eligibility and information availability explicit.
- Urgent-care diversion/escalation based on capabilities and access; urgency categories alone do not establish suitability.
- Residential aged/disability care, outreach and community follow-up, including discharge readiness, assessment/placement delays and readmissions.
- Regional/interregional and international flows represented through explicit network boundaries. Brisbane transfers from Cairns are intrastate; jurisdiction tags must come from geography rather than the mode of care.

First qualify ED → minimal ward → community and birthing → bounded theatre →
maternal/infant destinations in one process. Reference CHHHS/provider examples
remain site-profile candidates. Neither an EgressSink callback nor arrival at a
facility establishes treatment, birth, discharge or clinical handover completion.
See [composition contracts](composition-contracts-20261020.md).
