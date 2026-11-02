//! Domain birth records over the qualified A5 identity/message contracts.
//! Fetal entities are not modeled; newborn identities begin at the birth event.
//! Observations retain their availability time and are not policy inputs here.

use careops_core::birth::{
    BirthMessageId, BirthMessageInput, BirthMessageLedger, BirthMessageResult, BirthRecipient,
};
use careops_core::identity::{IdentityLedger, PersonId};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeliveryRoute {
    Vaginal,
    Operative,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct NeonatalObservation {
    pub infant_ordinal: u16,
    pub observed_tick: u64,
    pub available_tick: u64,
    pub variable: String,
    pub value: i64,
    pub unit: String,
    pub provenance: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeliveryRecord {
    pub route: DeliveryRoute,
    pub birth: BirthMessageInput,
    pub observations: Vec<NeonatalObservation>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RecoveryState {
    Recovering,
    Escalated,
    Discharged,
}

#[derive(Clone, Debug, Default)]
pub struct DeliveryLedger {
    births: BirthMessageLedger,
    records: BTreeMap<BirthMessageId, DeliveryRecord>,
    states: BTreeMap<(BirthMessageId, PersonId), (u64, RecoveryState)>,
}

impl DeliveryLedger {
    pub const fn models_fetal_identity(&self) -> bool {
        false
    }

    /// Normalize complete correlated records before an atomic core registration.
    /// A conflicting route or observation cannot be hidden by a birth-message retry.
    /// This bounded record accepts observations taken at birth only; availability
    /// may be delayed. Later observations require a separate update contract.
    pub fn register(
        &mut self,
        identities: &mut IdentityLedger,
        tick: u64,
        mut record: DeliveryRecord,
    ) -> Result<BirthMessageResult, String> {
        record
            .birth
            .infants
            .sort_by_key(|infant| infant.infant_ordinal);
        record.observations.sort();
        let ordinals: BTreeSet<_> = record
            .birth
            .infants
            .iter()
            .map(|infant| infant.infant_ordinal)
            .collect();
        let mut observed = BTreeSet::new();
        let retry = self.records.contains_key(&record.birth.message_id);
        for observation in &record.observations {
            if !ordinals.contains(&observation.infant_ordinal)
                || observation.observed_tick > tick
                || (!retry && observation.observed_tick != tick)
                || observation.available_tick < observation.observed_tick
                || [
                    &observation.variable,
                    &observation.unit,
                    &observation.provenance,
                ]
                .iter()
                .any(|text| text.trim().is_empty())
                || !observed.insert((
                    observation.infant_ordinal,
                    observation.observed_tick,
                    &observation.variable,
                ))
            {
                return Err("invalid or duplicate neonatal observation".into());
            }
        }
        if let Some(previous) = self.records.get(&record.birth.message_id) {
            if previous != &record {
                return Err("delivery payload conflict".into());
            }
            // Core still validates pairing with the supplied identity ledger.
            return self
                .births
                .register(identities, tick, record.birth)
                .map_err(|error| error.to_string());
        }
        let result = self
            .births
            .register(identities, tick, record.birth.clone())
            .map_err(|error| error.to_string())?;
        for recipient in &result.recipients {
            let person_id = match recipient {
                BirthRecipient::Mother { person_id, .. }
                | BirthRecipient::Infant { person_id, .. } => *person_id,
            };
            self.states
                .entry((record.birth.message_id, person_id))
                .or_insert((tick, RecoveryState::Recovering));
        }
        self.records.insert(record.birth.message_id, record);
        Ok(result)
    }

    pub fn record(&self, id: BirthMessageId) -> Option<&DeliveryRecord> {
        self.records.get(&id)
    }
    pub fn state(&self, delivery: BirthMessageId, id: PersonId) -> Option<(u64, RecoveryState)> {
        self.states.get(&(delivery, id)).copied()
    }

    /// Domain recovery is independent for each person. Discharge does not delete
    /// relatives or implicitly close their service episodes in the identity ledger.
    pub fn transition(
        &mut self,
        delivery: BirthMessageId,
        id: PersonId,
        tick: u64,
        state: RecoveryState,
    ) -> Result<(), String> {
        let previous = self
            .states
            .get_mut(&(delivery, id))
            .ok_or("unknown delivery participant")?;
        if tick < previous.0
            || (previous.1 == RecoveryState::Discharged && *previous != (tick, state))
        {
            return Err("late transition or discharged participant".into());
        }
        if tick == previous.0 && previous.1 != state {
            return Err("conflicting transition at the same tick".into());
        }
        *previous = (tick, state);
        Ok(())
    }
}
