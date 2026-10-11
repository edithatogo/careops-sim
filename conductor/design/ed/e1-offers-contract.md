# E1.3 finite offers contract v1

Coordinator frozen after independent GPT-6.1-sol review. Rust1.99.0; synthetic external finite admission boundary only. Module crates/careops-ed/src/offers.rs; coordinator owns facade. No ward capacity, causal ward policy or ambulance model is inferred.

## Exact interface

Inputs Clone/Debug/Deserialize, strict fields, outputs Clone/Debug/Eq/PartialEq/Serialize; enums snake_case. Reuse pathway::{PathwayError,ServiceProvenance}. Identifiers use existing ASCII alphanumeric/underscore/dot/hyphen1..64 convention.

OfferPolicy { version:u32, provenance:ServiceProvenance }.
BedOffer { offer_id:String, destination:String, patient_id:Option<String>, available_tick:u64, withdrawn_tick:u64, transfer_duration_ticks:u64 }.
AdmissionRequest { patient_id:String, destination:String, ready_tick:u64 }.
OfferAllocation { offer_id:String, patient_id:String, destination:String, allocated_tick:u64, transfer_duration_ticks:u64 }.
OfferStatus { NotYetAvailable, Available, Expired, Consumed }.
OfferObservation { offer:BedOffer, status:OfferStatus, allocation:Option<OfferAllocation> }.
OfferBook::new(policy:&OfferPolicy, offers:&[BedOffer]) -> Result<OfferBook,PathwayError>.
OfferBook::allocate(&mut self, tick:u64, waiting:&[AdmissionRequest]) -> Result<Vec<OfferAllocation>,PathwayError>.
OfferBook::snapshot(&self, tick:u64) -> Result<Vec<OfferObservation>,PathwayError>.
OfferBook::boundary_ticks(&self) -> Vec<u64>.

BedOffer must also derive Serialize/Eq/PartialEq for echoed provenance. Policy version1 and nonempty trimmed synthetic provenance. At most100000 offers or requests per batch. Validate all IDs, uniqueness, available<withdrawn, positive transfer duration before cloning or mutating state. Empty valid books/batches allowed. Validate duplicate request patient IDs, ready_tick<=tick and previously consumed patient IDs before any change. No patient can consume two offers. Distinct destinations and optional patient restriction must both match.

## Event contract

Caller is the shared runtime, after draining all events at a tick. ready_tick means admission-decision/boarding eligibility time, preserved while unmatched. The caller retains stable destination/readiness and removes allocated patients; the helper does not certify completeness of the supplied waiting set. Admission requests become waiting only after actual treatment completion; future/latent decisions must not be passed. allocate may be called once per tick with complete current waiting set; strictly increasing tick after first successful call. A failed call leaves the entire book unchanged, including last tick. snapshot may inspect current/later tick but rejects a tick earlier than the last successful allocation call. Queries do not mutate allocation state. boundary_ticks returns sorted unique availability and withdrawal ticks for scheduling. The caller filters to its horizon; beyond-horizon offers are valid future offers.

Validity is [available_tick,withdrawn_tick): withdrawal exactly at allocation tick wins. Allocate requests sorted by (ready_tick,bytewise patient_id), selecting compatible available offers by (available_tick,bytewise offer_id). Optional patient_id restricts eligibility; it is external offer eligibility, not realized future patient outcome. Consumption is atomic at allocation, once only. Consumed offers survive later withdrawal; duration is a positive supplied transfer-work requirement, not proof transfer started or completed. Return allocations in canonical request order; snapshot in offer_id order. Unconsumed offers classify by queried tick; consumed observations echo allocation.

Avoid O(patients*offers) matching. Index active unrestricted and patient-restricted offers per destination; maintain time-indexed activation/withdrawal and select earliest compatible candidate. A strictly monotonic batched caller can update each offer at most once per boundary. Inputs bounded before cloning; event/work arithmetic uses checked or u128 arithmetic where needed. No implicit patient-resource reservation, departure event or full E1.3 completion is claimed by this helper.

## Native oracles

One offer consumed once, one patient at most once. Exact availability accepts, exact withdrawal rejects; allocation before withdrawal survives later withdrawal. Wrong destination and wrong restricted patient cannot consume. Deterministic competing request readiness then IDs, competing offer availability then IDs; all input/request permutations preserve allocations/snapshots. Restricted and unrestricted pools compete by same offer ordering. Future/expired/unused/consumed snapshots and boundary ticks independently asserted. Empty book/batch, duplicate/invalid IDs, invalid interval/zero duration, malformed policy, request before readiness, nonmonotonic/repeated allocation ticks and duplicate consumed patient reject. Invalid later item cannot partially consume earlier valid item.An older compatible pending request retains priority across separate tick calls when a later request joins.100000 bound behavior checked without excess cloning. No retrospective whole-pathway or terminal claim; boarding integration consumes this API on shared scheduler in next leaf.

Available means time-valid and unconsumed, independent of any specific patient compatibility. Consumed status takes precedence over later expiry. OfferBook does not compute transfer completion or certify transfer started; runtime uses u128 or checked time arithmetic on actual transfer start. Failed input calls leave snapshots and same-tick retry availability unchanged.
