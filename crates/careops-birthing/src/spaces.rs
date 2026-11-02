//! Synthetic birthing spaces over neutral capability/calendar/capacity contracts.
//! One tick is one UTC second from synthetic epoch zero. No clinical room,
//! staffing, equipment or eligibility requirement is supplied by these labels.

use careops_core::capacity::{Calendar, CapabilitySet, CapacitySnapshot};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpaceKind {
    Ldr,
    Pool,
    Monitoring,
    Stabilisation,
}

#[derive(Clone, Debug)]
pub struct SpaceProfile {
    pub id: u64,
    pub revision: String,
    pub kind: SpaceKind,
    pub capabilities: CapabilitySet,
    pub open: Calendar,
    pub staffed: Option<Calendar>,
    pub equipped: Option<bool>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum State {
    Vacant,
    Reserved(u64),
    Occupied(u64),
    Cleaning(u64),
}

#[derive(Clone, Debug)]
struct Space {
    profile: SpaceProfile,
    state: State,
}

#[derive(Clone, Debug)]
struct Reservation {
    room: u64,
    requirements: CapabilitySet,
    reserved_at: i64,
    started_at: Option<i64>,
    departed_at: Option<i64>,
    cancelled_at: Option<i64>,
    cleaned_at: Option<i64>,
}

#[derive(Clone, Debug)]
pub struct SpacePool {
    rooms: BTreeMap<u64, Space>,
    reservations: BTreeMap<u64, Reservation>,
    now: i64,
}

impl SpacePool {
    pub fn new(profiles: Vec<SpaceProfile>) -> Result<Self, String> {
        let mut rooms = BTreeMap::new();
        for profile in profiles {
            if profile.id == 0
                || profile.revision.trim().is_empty()
                || rooms.contains_key(&profile.id)
            {
                return Err("duplicate/invalid room ID or missing revision".into());
            }
            rooms.insert(
                profile.id,
                Space {
                    profile,
                    state: State::Vacant,
                },
            );
        }
        Ok(Self {
            rooms,
            reservations: BTreeMap::new(),
            now: 0,
        })
    }

    fn check_time(&self, tick: i64) -> Result<(), String> {
        if tick < self.now {
            Err("late space event".into())
        } else {
            Ok(())
        }
    }

    /// Stable lowest-ID matching, not a clinical priority or queue policy.
    /// Replaying an exact historical reservation returns its receipt without
    /// reacquiring a released room. An episode cannot request a second room here.
    pub fn reserve(
        &mut self,
        episode: u64,
        requirements: CapabilitySet,
        tick: i64,
    ) -> Result<u64, String> {
        if let Some(previous) = self.reservations.get(&episode) {
            return if previous.requirements == requirements && previous.reserved_at == tick {
                Ok(previous.room)
            } else {
                Err("reservation payload conflict".into())
            };
        }
        self.check_time(tick)?;
        if episode == 0 {
            return Err("episode ID must be nonzero".into());
        }
        let room = self
            .rooms
            .iter()
            .find_map(|(id, space)| {
                if space.state == State::Vacant
                    && space.profile.capabilities.satisfies(&requirements)
                    && snapshot_at(space, tick)
                        .assignable_capacity()
                        .ok()
                        .flatten()
                        == Some(1)
                {
                    Some(*id)
                } else {
                    None
                }
            })
            .ok_or("no compatible known assignable room")?;
        self.rooms
            .get_mut(&room)
            .expect("selected room exists")
            .state = State::Reserved(episode);
        self.reservations.insert(
            episode,
            Reservation {
                room,
                requirements,
                reserved_at: tick,
                started_at: None,
                departed_at: None,
                cancelled_at: None,
                cleaned_at: None,
            },
        );
        self.now = tick;
        Ok(room)
    }

    pub fn start(&mut self, episode: u64, tick: i64) -> Result<(), String> {
        let reservation = self
            .reservations
            .get(&episode)
            .ok_or("unknown reservation")?;
        if reservation.started_at == Some(tick) {
            return Ok(());
        }
        self.check_time(tick)?;
        let room = reservation.room;
        if self.rooms[&room].state != State::Reserved(episode) {
            return Err("reservation is not held".into());
        }
        let capacity = snapshot_at(&self.rooms[&room], tick);
        if capacity.open != Some(1) || capacity.staffed != Some(1) || capacity.equipped != Some(1) {
            return Err("reserved work cannot start without open, staffed equipment".into());
        }
        self.rooms
            .get_mut(&room)
            .expect("reservation room exists")
            .state = State::Occupied(episode);
        self.reservations
            .get_mut(&episode)
            .expect("reservation exists")
            .started_at = Some(tick);
        self.now = tick;
        Ok(())
    }

    /// Actual departure releases occupancy into cleaning. Clinical readiness,
    /// a destination offer or the simulation horizon never calls this implicitly.
    pub fn depart(&mut self, episode: u64, tick: i64) -> Result<(), String> {
        let reservation = self
            .reservations
            .get(&episode)
            .ok_or("unknown reservation")?;
        if reservation.departed_at == Some(tick) {
            return Ok(());
        }
        self.check_time(tick)?;
        let room = reservation.room;
        if self.rooms[&room].state != State::Occupied(episode) {
            return Err("episode is not occupying room".into());
        }
        self.rooms
            .get_mut(&room)
            .expect("reservation room exists")
            .state = State::Cleaning(episode);
        self.reservations
            .get_mut(&episode)
            .expect("reservation exists")
            .departed_at = Some(tick);
        self.now = tick;
        Ok(())
    }

    pub fn finish_cleaning(&mut self, episode: u64, tick: i64) -> Result<(), String> {
        let reservation = self
            .reservations
            .get(&episode)
            .ok_or("unknown reservation")?;
        if reservation.cleaned_at == Some(tick) {
            return Ok(());
        }
        self.check_time(tick)?;
        let room = reservation.room;
        if self.rooms[&room].state != State::Cleaning(episode) {
            return Err("episode has no cleaning hold".into());
        }
        self.rooms
            .get_mut(&room)
            .expect("reservation room exists")
            .state = State::Vacant;
        self.reservations
            .get_mut(&episode)
            .expect("reservation exists")
            .cleaned_at = Some(tick);
        self.now = tick;
        Ok(())
    }

    pub fn cancel(&mut self, episode: u64, tick: i64) -> Result<(), String> {
        let reservation = self
            .reservations
            .get(&episode)
            .ok_or("unknown reservation")?;
        if reservation.cancelled_at == Some(tick) {
            return Ok(());
        }
        self.check_time(tick)?;
        let room = reservation.room;
        if self.rooms[&room].state != State::Reserved(episode) {
            return Err("only an unstarted reservation can cancel".into());
        }
        self.rooms
            .get_mut(&room)
            .expect("reservation room exists")
            .state = State::Vacant;
        self.reservations
            .get_mut(&episode)
            .expect("reservation exists")
            .cancelled_at = Some(tick);
        self.now = tick;
        Ok(())
    }

    /// Closure changes admission availability and never evicts current holds.
    pub fn set_open(&mut self, room: u64, open: Calendar, tick: i64) -> Result<(), String> {
        self.check_time(tick)?;
        self.rooms
            .get_mut(&room)
            .ok_or("unknown room")?
            .profile
            .open = open;
        self.now = tick;
        Ok(())
    }

    /// Advancing/censoring time preserves all reservations and occupancy.
    pub fn advance_to(&mut self, tick: i64) -> Result<(), String> {
        self.check_time(tick)?;
        self.now = tick;
        Ok(())
    }

    pub fn snapshot(&self, room: u64) -> Result<CapacitySnapshot, String> {
        let value = snapshot_at(self.rooms.get(&room).ok_or("unknown room")?, self.now);
        value.validate().map_err(|error| error.to_string())?;
        Ok(value)
    }

    pub fn occupied_seconds(&self, episode: u64) -> Result<u128, String> {
        let reservation = self
            .reservations
            .get(&episode)
            .ok_or("unknown reservation")?;
        Ok(reservation.started_at.map_or(0, |start| {
            (i128::from(reservation.departed_at.unwrap_or(self.now)) - i128::from(start)).max(0)
                as u128
        }))
    }
}

fn snapshot_at(space: &Space, tick: i64) -> CapacitySnapshot {
    CapacitySnapshot {
        physical: Some(1),
        equipped: space.profile.equipped.map(u64::from),
        open: Some(u64::from(space.profile.open.contains(tick))),
        staffed: space
            .profile
            .staffed
            .as_ref()
            .map(|calendar| u64::from(calendar.contains(tick))),
        reserved: u64::from(matches!(space.state, State::Reserved(_))),
        occupied: u64::from(matches!(space.state, State::Occupied(_))),
        cleaning: u64::from(matches!(space.state, State::Cleaning(_))),
        unavailable: 0,
    }
}
