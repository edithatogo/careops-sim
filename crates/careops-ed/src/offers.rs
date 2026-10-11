//! Experimental finite external admission offers.
//!
//! The OfferBook models synthetic finite offers and one-time allocation only. It
//! does not start transfer work, release ED occupancy, or infer a terminal event.

use std::collections::{BTreeMap, BTreeSet, HashSet};

use serde::{Deserialize, Serialize};

use crate::pathway::{PathwayError, ServiceProvenance};

const MAX_OFFERS: usize = 100_000;
const MAX_REQUESTS: usize = 100_000;

type OfferKey = (u64, String);
type DestinationPatient = (String, String);

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OfferPolicy {
    pub version: u32,
    pub provenance: ServiceProvenance,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BedOffer {
    pub offer_id: String,
    pub destination: String,
    pub patient_id: Option<String>,
    pub available_tick: u64,
    pub withdrawn_tick: u64,
    pub transfer_duration_ticks: u64,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdmissionRequest {
    pub patient_id: String,
    pub destination: String,
    pub ready_tick: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OfferAllocation {
    pub offer_id: String,
    pub patient_id: String,
    pub destination: String,
    pub allocated_tick: u64,
    pub transfer_duration_ticks: u64,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OfferStatus {
    NotYetAvailable,
    Available,
    Expired,
    Consumed,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OfferObservation {
    pub offer: BedOffer,
    pub status: OfferStatus,
    pub allocation: Option<OfferAllocation>,
}

/// A bounded, deterministic offer inventory with atomic one-time consumption.
///
/// Callers provide the complete set of currently waiting requests once per
/// increasing scheduler tick. The book owns offer state and patient consumption,
/// but not patient boarding state or transfer execution.
pub struct OfferBook {
    offers: BTreeMap<String, BedOffer>,
    activations: BTreeMap<u64, Vec<String>>,
    withdrawals: BTreeMap<u64, Vec<String>>,
    active_general: BTreeMap<String, BTreeSet<OfferKey>>,
    active_restricted: BTreeMap<DestinationPatient, BTreeSet<OfferKey>>,
    allocations: BTreeMap<String, OfferAllocation>,
    consumed_patients: HashSet<String>,
    boundaries: Vec<u64>,
    last_tick: Option<u64>,
}

impl OfferBook {
    pub fn new(policy: &OfferPolicy, offers: &[BedOffer]) -> Result<Self, PathwayError> {
        validate_policy(policy)?;
        if offers.len() > MAX_OFFERS {
            return invalid("offer limit exceeded");
        }
        let mut ids = HashSet::with_capacity(offers.len());
        for offer in offers {
            if !valid_identifier(&offer.offer_id)
                || !valid_identifier(&offer.destination)
                || offer
                    .patient_id
                    .as_deref()
                    .is_some_and(|patient_id| !valid_identifier(patient_id))
            {
                return invalid("offer identifiers are invalid");
            }
            if !ids.insert(offer.offer_id.as_str()) {
                return invalid("offer IDs must be unique");
            }
            if offer.available_tick >= offer.withdrawn_tick {
                return invalid("offer lifetime must be nonempty and half-open");
            }
            if offer.transfer_duration_ticks == 0 {
                return invalid("offer transfer duration must be positive");
            }
        }

        let boundaries = offers
            .iter()
            .flat_map(|offer| [offer.available_tick, offer.withdrawn_tick])
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        let mut book = Self {
            offers: BTreeMap::new(),
            activations: BTreeMap::new(),
            withdrawals: BTreeMap::new(),
            active_general: BTreeMap::new(),
            active_restricted: BTreeMap::new(),
            allocations: BTreeMap::new(),
            consumed_patients: HashSet::new(),
            boundaries,
            last_tick: None,
        };
        for offer in offers {
            book.activations
                .entry(offer.available_tick)
                .or_default()
                .push(offer.offer_id.clone());
            book.withdrawals
                .entry(offer.withdrawn_tick)
                .or_default()
                .push(offer.offer_id.clone());
            book.offers.insert(offer.offer_id.clone(), offer.clone());
        }
        Ok(book)
    }

    /// Allocate available compatible offers after the caller drains the whole tick.
    pub fn allocate(
        &mut self,
        tick: u64,
        waiting: &[AdmissionRequest],
    ) -> Result<Vec<OfferAllocation>, PathwayError> {
        if self.last_tick.is_some_and(|last_tick| tick <= last_tick) {
            return invalid("allocation ticks must strictly increase");
        }
        if waiting.len() > MAX_REQUESTS {
            return invalid("admission request limit exceeded");
        }
        let mut request_ids = HashSet::with_capacity(waiting.len());
        for request in waiting {
            if !valid_identifier(&request.patient_id) || !valid_identifier(&request.destination) {
                return invalid("admission request identifiers are invalid");
            }
            if !request_ids.insert(request.patient_id.as_str()) {
                return invalid("admission request patient IDs must be unique");
            }
            if request.ready_tick > tick {
                return invalid("admission request is not ready at allocation tick");
            }
            if self.consumed_patients.contains(&request.patient_id) {
                return invalid("patient has already consumed an offer");
            }
        }

        // All input rejection happens above, before event queues or allocations mutate.
        self.expire_through(tick);
        self.activate_through(tick);

        let mut ordered: Vec<&AdmissionRequest> = waiting.iter().collect();
        ordered.sort_by(|left, right| {
            left.ready_tick
                .cmp(&right.ready_tick)
                .then_with(|| left.patient_id.cmp(&right.patient_id))
        });
        let mut allocations = Vec::new();
        for request in ordered {
            let Some((available_tick, offer_id)) =
                self.earliest_compatible(&request.destination, &request.patient_id)
            else {
                continue;
            };
            self.remove_active(&offer_id);
            let offer = self
                .offers
                .get(&offer_id)
                .expect("active offer must exist in validated inventory");
            let allocation = OfferAllocation {
                offer_id: offer_id.clone(),
                patient_id: request.patient_id.clone(),
                destination: offer.destination.clone(),
                allocated_tick: tick.max(available_tick),
                transfer_duration_ticks: offer.transfer_duration_ticks,
            };
            self.allocations.insert(offer_id, allocation.clone());
            self.consumed_patients.insert(request.patient_id.clone());
            allocations.push(allocation);
        }
        self.last_tick = Some(tick);
        Ok(allocations)
    }

    /// Return a canonical read-only view of offer state at `tick`.
    pub fn snapshot(&self, tick: u64) -> Result<Vec<OfferObservation>, PathwayError> {
        if self.last_tick.is_some_and(|last_tick| tick < last_tick) {
            return invalid("offer snapshot cannot precede the last allocation tick");
        }
        Ok(self
            .offers
            .values()
            .map(|offer| {
                let allocation = self.allocations.get(&offer.offer_id).cloned();
                let status = if allocation.is_some() {
                    OfferStatus::Consumed
                } else if tick < offer.available_tick {
                    OfferStatus::NotYetAvailable
                } else if tick >= offer.withdrawn_tick {
                    OfferStatus::Expired
                } else {
                    OfferStatus::Available
                };
                OfferObservation {
                    offer: offer.clone(),
                    status,
                    allocation,
                }
            })
            .collect())
    }

    /// Sorted unique availability and withdrawal boundaries for scheduler setup.
    pub fn boundary_ticks(&self) -> Vec<u64> {
        self.boundaries.clone()
    }

    fn expire_through(&mut self, tick: u64) {
        let due_ticks: Vec<u64> = self.withdrawals.range(..=tick).map(|(at, _)| *at).collect();
        for at in due_ticks {
            if let Some(offer_ids) = self.withdrawals.remove(&at) {
                for offer_id in offer_ids {
                    self.remove_active(&offer_id);
                }
            }
        }
    }

    fn activate_through(&mut self, tick: u64) {
        let due_ticks: Vec<u64> = self.activations.range(..=tick).map(|(at, _)| *at).collect();
        for at in due_ticks {
            if let Some(offer_ids) = self.activations.remove(&at) {
                for offer_id in offer_ids {
                    let offer = self
                        .offers
                        .get(&offer_id)
                        .expect("scheduled offer must exist in validated inventory");
                    if offer.withdrawn_tick > tick && !self.allocations.contains_key(&offer_id) {
                        let key = (offer.available_tick, offer_id);
                        if let Some(patient_id) = &offer.patient_id {
                            self.active_restricted
                                .entry((offer.destination.clone(), patient_id.clone()))
                                .or_default()
                                .insert(key);
                        } else {
                            self.active_general
                                .entry(offer.destination.clone())
                                .or_default()
                                .insert(key);
                        }
                    }
                }
            }
        }
    }

    fn earliest_compatible(&self, destination: &str, patient_id: &str) -> Option<OfferKey> {
        let general = self
            .active_general
            .get(destination)
            .and_then(|offers| offers.first())
            .cloned();
        let restricted = self
            .active_restricted
            .get(&(destination.to_owned(), patient_id.to_owned()))
            .and_then(|offers| offers.first())
            .cloned();
        match (general, restricted) {
            (Some(general), Some(restricted)) => Some(general.min(restricted)),
            (Some(general), None) => Some(general),
            (None, Some(restricted)) => Some(restricted),
            (None, None) => None,
        }
    }

    fn remove_active(&mut self, offer_id: &str) {
        let Some(offer) = self.offers.get(offer_id) else {
            return;
        };
        let key = (offer.available_tick, offer_id.to_owned());
        if let Some(patient_id) = &offer.patient_id {
            remove_index_entry(
                &mut self.active_restricted,
                (offer.destination.clone(), patient_id.clone()),
                &key,
            );
        } else {
            remove_index_entry(&mut self.active_general, offer.destination.clone(), &key);
        }
    }
}

fn remove_index_entry<K: Ord>(
    index: &mut BTreeMap<K, BTreeSet<OfferKey>>,
    key: K,
    offer_key: &OfferKey,
) {
    if let Some(offers) = index.get_mut(&key) {
        offers.remove(offer_key);
        if offers.is_empty() {
            index.remove(&key);
        }
    }
}

fn validate_policy(policy: &OfferPolicy) -> Result<(), PathwayError> {
    if policy.version != 1
        || policy.provenance.class != "synthetic"
        || policy.provenance.notes.trim().is_empty()
        || policy.provenance.notes.trim() != policy.provenance.notes
    {
        return invalid("offer policy must use version 1 and trimmed synthetic provenance");
    }
    Ok(())
}

fn valid_identifier(value: &str) -> bool {
    let bytes = value.as_bytes();
    !bytes.is_empty()
        && bytes.len() <= 64
        && bytes
            .iter()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(*byte, b'_' | b'.' | b'-'))
}

fn invalid<T>(message: &str) -> Result<T, PathwayError> {
    Err(PathwayError::InvalidInput(format!(
        "invalid offer: {message}"
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn policy() -> OfferPolicy {
        OfferPolicy {
            version: 1,
            provenance: ServiceProvenance {
                class: "synthetic".into(),
                notes: "offer fixture".into(),
            },
        }
    }

    fn offer(
        offer_id: &str,
        destination: &str,
        patient_id: Option<&str>,
        available_tick: u64,
        withdrawn_tick: u64,
        transfer_duration_ticks: u64,
    ) -> BedOffer {
        BedOffer {
            offer_id: offer_id.into(),
            destination: destination.into(),
            patient_id: patient_id.map(str::to_owned),
            available_tick,
            withdrawn_tick,
            transfer_duration_ticks,
        }
    }

    fn request(patient_id: &str, destination: &str, ready_tick: u64) -> AdmissionRequest {
        AdmissionRequest {
            patient_id: patient_id.into(),
            destination: destination.into(),
            ready_tick,
        }
    }

    fn expect_invalid<T>(result: Result<T, PathwayError>) {
        assert!(matches!(result, Err(PathwayError::InvalidInput(_))));
    }

    #[test]
    fn empty_book_and_snapshots_are_well_formed() {
        let mut book = OfferBook::new(&policy(), &[]).unwrap();
        assert!(book.boundary_ticks().is_empty());
        assert!(book.snapshot(0).unwrap().is_empty());
        assert!(book.allocate(0, &[]).unwrap().is_empty());
        assert!(book.allocate(1, &[]).unwrap().is_empty());
    }

    #[test]
    fn half_open_lifetime_availability_and_consumption_are_exact() {
        let offers = vec![offer("offer-a", "medical", None, 2, 4, 2)];
        let mut book = OfferBook::new(&policy(), &offers).unwrap();
        assert_eq!(
            book.snapshot(1).unwrap()[0].status,
            OfferStatus::NotYetAvailable
        );
        let allocated = book
            .allocate(2, &[request("patient-a", "medical", 2)])
            .unwrap();
        assert_eq!(allocated.len(), 1);
        assert_eq!(allocated[0].offer_id, "offer-a");
        assert_eq!(allocated[0].patient_id, "patient-a");
        assert_eq!(allocated[0].destination, "medical");
        assert_eq!(allocated[0].allocated_tick, 2);
        assert_eq!(allocated[0].transfer_duration_ticks, 2);
        let snapshot = book.snapshot(4).unwrap();
        assert_eq!(snapshot[0].status, OfferStatus::Consumed);
        assert_eq!(snapshot[0].allocation, Some(allocated[0].clone()));

        let mut expired =
            OfferBook::new(&policy(), &[offer("offer-b", "medical", None, 1, 4, 1)]).unwrap();
        assert!(expired
            .allocate(4, &[request("patient-b", "medical", 4)])
            .unwrap()
            .is_empty());
        assert_eq!(expired.snapshot(4).unwrap()[0].status, OfferStatus::Expired);
    }

    #[test]
    fn wrong_destinations_and_patient_restrictions_do_not_consume() {
        let mut general =
            OfferBook::new(&policy(), &[offer("offer-med", "medical", None, 1, 10, 2)]).unwrap();
        assert!(general
            .allocate(2, &[request("patient-a", "surgery", 2)])
            .unwrap()
            .is_empty());
        assert_eq!(
            general.snapshot(2).unwrap()[0].status,
            OfferStatus::Available
        );
        assert_eq!(
            general
                .allocate(3, &[request("patient-a", "medical", 3)])
                .unwrap()[0]
                .offer_id,
            "offer-med"
        );

        let mut targeted = OfferBook::new(
            &policy(),
            &[offer(
                "offer-targeted",
                "medical",
                Some("patient-b"),
                1,
                10,
                2,
            )],
        )
        .unwrap();
        assert!(targeted
            .allocate(2, &[request("patient-a", "medical", 2)])
            .unwrap()
            .is_empty());
        assert_eq!(
            targeted.snapshot(2).unwrap()[0].status,
            OfferStatus::Available
        );
        assert_eq!(
            targeted
                .allocate(3, &[request("patient-b", "medical", 3)])
                .unwrap()[0]
                .offer_id,
            "offer-targeted"
        );
    }

    #[test]
    fn request_and_offer_order_is_canonical_and_input_permutation_invariant() {
        let offers = vec![
            offer("offer-z", "medical", None, 3, 20, 1),
            offer("offer-a", "medical", None, 3, 20, 2),
            offer("offer-b-target", "medical", Some("patient-b"), 2, 20, 3),
        ];
        let requests = vec![
            request("patient-a", "medical", 5),
            request("patient-b", "medical", 4),
        ];
        let mut first = OfferBook::new(&policy(), &offers).unwrap();
        let expected = first.allocate(5, &requests).unwrap();
        assert_eq!(expected[0].patient_id, "patient-b");
        assert_eq!(expected[0].offer_id, "offer-b-target");
        assert_eq!(expected[1].patient_id, "patient-a");
        assert_eq!(expected[1].offer_id, "offer-a");

        let mut permuted_offers = offers;
        permuted_offers.reverse();
        let mut permuted_requests = requests;
        permuted_requests.reverse();
        let mut second = OfferBook::new(&policy(), &permuted_offers).unwrap();
        assert_eq!(second.allocate(5, &permuted_requests).unwrap(), expected);
        assert_eq!(second.snapshot(5).unwrap(), first.snapshot(5).unwrap());
    }

    #[test]
    fn same_tick_request_tie_uses_bytewise_patient_id_not_input_order() {
        let offers = [offer("offer-a", "medical", None, 3, 10, 1)];
        let requests = [
            request("patient-b", "medical", 3),
            request("patient-a", "medical", 3),
        ];
        let mut reversed = OfferBook::new(&policy(), &offers).unwrap();
        let first = reversed.allocate(3, &requests).unwrap();
        assert_eq!(first.len(), 1);
        assert_eq!(first[0].patient_id, "patient-a");
        assert_eq!(first[0].offer_id, "offer-a");

        let mut canonical = OfferBook::new(&policy(), &offers).unwrap();
        let second = canonical
            .allocate(3, &[requests[1].clone(), requests[0].clone()])
            .unwrap();
        assert_eq!(second, first);
        assert_eq!(
            canonical.snapshot(3).unwrap(),
            reversed.snapshot(3).unwrap()
        );
    }

    #[test]
    fn older_pending_request_keeps_priority_across_tick_batches() {
        let mut book =
            OfferBook::new(&policy(), &[offer("offer-a", "medical", None, 8, 12, 2)]).unwrap();
        assert!(book
            .allocate(5, &[request("patient-z", "medical", 3)])
            .unwrap()
            .is_empty());
        let allocations = book
            .allocate(
                8,
                &[
                    request("patient-a", "medical", 7),
                    request("patient-z", "medical", 3),
                ],
            )
            .unwrap();
        assert_eq!(allocations.len(), 1);
        assert_eq!(allocations[0].patient_id, "patient-z");
        assert_eq!(allocations[0].allocated_tick, 8);
    }

    #[test]
    fn failed_batch_is_atomic_and_can_retry_at_same_tick() {
        let mut book =
            OfferBook::new(&policy(), &[offer("offer-a", "medical", None, 1, 20, 2)]).unwrap();
        let before = book.snapshot(5).unwrap();
        expect_invalid(book.allocate(
            5,
            &[
                request("patient-a", "medical", 1),
                request("patient-b", "medical", 6),
            ],
        ));
        assert_eq!(book.snapshot(5).unwrap(), before);
        let allocations = book
            .allocate(5, &[request("patient-a", "medical", 1)])
            .unwrap();
        assert_eq!(allocations.len(), 1);
        assert_eq!(allocations[0].patient_id, "patient-a");
    }

    #[test]
    fn duplicate_request_and_consumed_patient_are_rejected_without_mutation() {
        let mut book = OfferBook::new(
            &policy(),
            &[
                offer("offer-a", "medical", None, 1, 20, 1),
                offer("offer-b", "medical", None, 1, 20, 1),
            ],
        )
        .unwrap();
        expect_invalid(book.allocate(
            2,
            &[
                request("patient-a", "medical", 1),
                request("patient-a", "medical", 2),
            ],
        ));
        assert_eq!(book.snapshot(2).unwrap()[0].status, OfferStatus::Available);
        assert_eq!(
            book.allocate(2, &[request("patient-a", "medical", 1)])
                .unwrap()
                .len(),
            1
        );
        let before = book.snapshot(3).unwrap();
        expect_invalid(book.allocate(3, &[request("patient-a", "medical", 1)]));
        assert_eq!(book.snapshot(3).unwrap(), before);
    }

    #[test]
    fn snapshots_and_boundary_ticks_cover_future_available_expired_and_consumed() {
        let mut book = OfferBook::new(
            &policy(),
            &[
                offer("offer-c", "medical", None, 8, 12, 1),
                offer("offer-a", "medical", None, 1, 5, 1),
                offer("offer-b", "surgery", None, 3, 12, 1),
            ],
        )
        .unwrap();
        assert_eq!(book.boundary_ticks(), vec![1, 3, 5, 8, 12]);
        assert_eq!(book.snapshot(0).unwrap()[0].offer.offer_id, "offer-a");
        assert_eq!(
            book.snapshot(0).unwrap()[0].status,
            OfferStatus::NotYetAvailable
        );
        assert_eq!(book.snapshot(6).unwrap()[0].status, OfferStatus::Expired);
        assert_eq!(book.snapshot(6).unwrap()[1].status, OfferStatus::Available);
        assert_eq!(
            book.snapshot(6).unwrap()[2].status,
            OfferStatus::NotYetAvailable
        );
        book.allocate(8, &[request("patient-c", "medical", 7)])
            .unwrap();
        assert_eq!(book.snapshot(12).unwrap()[2].status, OfferStatus::Consumed);
        assert_eq!(book.boundary_ticks(), vec![1, 3, 5, 8, 12]);
    }

    #[test]
    fn allocation_ticks_must_increase_and_snapshot_cannot_rewind() {
        let mut book = OfferBook::new(&policy(), &[]).unwrap();
        book.allocate(5, &[]).unwrap();
        expect_invalid(book.allocate(5, &[]));
        expect_invalid(book.allocate(4, &[]));
        expect_invalid(book.snapshot(4));
        assert!(book.allocate(6, &[]).unwrap().is_empty());
    }

    #[test]
    fn policies_offers_and_requests_are_validated() {
        let mut bad_policy = policy();
        bad_policy.version = 2;
        expect_invalid(OfferBook::new(&bad_policy, &[]));
        let mut bad_policy = policy();
        bad_policy.provenance.notes = "  ".into();
        expect_invalid(OfferBook::new(&bad_policy, &[]));
        let mut bad_policy = policy();
        bad_policy.provenance.class = "observed".into();
        expect_invalid(OfferBook::new(&bad_policy, &[]));
        for item in [
            offer("", "medical", None, 1, 2, 1),
            offer("offer", "bad:id", None, 1, 2, 1),
            offer("offer", "medical", Some("bad:id"), 1, 2, 1),
            offer("offer", "medical", None, 2, 2, 1),
            offer("offer", "medical", None, 3, 2, 1),
            offer("offer", "medical", None, 1, 2, 0),
        ] {
            expect_invalid(OfferBook::new(&policy(), &[item]));
        }
        expect_invalid(OfferBook::new(
            &policy(),
            &[
                offer("same", "medical", None, 1, 2, 1),
                offer("same", "surgery", None, 1, 2, 1),
            ],
        ));
        let mut book = OfferBook::new(&policy(), &[]).unwrap();
        expect_invalid(book.allocate(1, &[request("bad:id", "medical", 1)]));
        expect_invalid(book.allocate(1, &[request("p", "bad:id", 1)]));
        expect_invalid(book.allocate(1, &[request("p", "medical", 2)]));
    }

    #[test]
    fn unrestricted_and_patient_restricted_offers_share_availability_order() {
        let mut book = OfferBook::new(
            &policy(),
            &[
                offer("general", "medical", None, 3, 20, 1),
                offer("targeted", "medical", Some("patient-a"), 2, 20, 1),
            ],
        )
        .unwrap();
        let allocated = book
            .allocate(3, &[request("patient-a", "medical", 3)])
            .unwrap();
        assert_eq!(allocated[0].offer_id, "targeted");
        assert_eq!(book.snapshot(3).unwrap()[0].status, OfferStatus::Available);
    }

    #[test]
    fn max_bounds_are_checked_before_indexing_and_allow_exact_limit() {
        let offers: Vec<BedOffer> = (0..MAX_OFFERS)
            .map(|index| offer(&format!("o{index:06}"), "medical", None, 1, 10, 1))
            .collect();
        let mut book = OfferBook::new(&policy(), &offers).unwrap();
        let requests: Vec<AdmissionRequest> = (0..MAX_REQUESTS)
            .map(|index| request(&format!("p{index:06}"), "surgery", 1))
            .collect();
        assert!(book.allocate(1, &requests).unwrap().is_empty());
        let mut too_many_requests = requests;
        too_many_requests.push(request("extra", "surgery", 1));
        expect_invalid(book.allocate(2, &too_many_requests));
        assert!(book.allocate(2, &[]).unwrap().is_empty());

        let too_many_offers: Vec<BedOffer> = (0..=MAX_OFFERS)
            .map(|index| offer(&format!("o{index:06}"), "medical", None, 1, 10, 1))
            .collect();
        expect_invalid(OfferBook::new(&policy(), &too_many_offers));
    }
}
