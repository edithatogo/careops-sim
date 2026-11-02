# ADR-0015: Idempotent birth messages and independent destinations

- Status: Accepted for A5.3 implementation
- Date: 2026-10-27
- Related: CC-01, CC-02 and CC-07; [ADR-0014](ADR-0014-person-episode-identity.md)

## Context

A5.3 must qualify multiple-infant identity creation, duplicate birth-message
handling and split maternal/infant destinations. The current CareOps source has
birth-origin `PersonId` records, birth-event IDs and infant ordinals, but no
pre-birth fetal entity, fetal ID, fetal state machine or fetal-to-newborn API.
There is therefore no modeled fetal identity to map. A5.3 records that boundary;
it does not invent fetal records. If a later consumer adds fetal state, it must
define an explicit, provenance-preserving mapping before claiming this behavior.

## Decision

1. A birth message ID is a transport/idempotency key, distinct from a birth event
   ID. Replaying the same message ID with byte-equivalent typed payload returns
   its stored result without changing identity state. Reusing that message ID
   with different content fails atomically.
2. One birth event may be registered by only one message ID. Infant ordinals
   must be unique within that event, and each registered event has at least one
   infant. A conflict in any infant or relationship
   rejects the whole batch, with no partial identities or links.
3. Each infant is created as its own person with `BirthEventId` and ordinal
   provenance and a typed `BirthParentOf` link to the mother. Relationship
   removal remains independent of person and episode lifecycle.
4. Maternal and each infant destination are explicit independent service IDs.
   The result exposes one destination intent per person. Downstream transfer
   operations remain one person/episode per operation; split outcomes do not
   imply an atomic multi-recipient transfer or shared failure state.
5. Persist the message ledger separately from the identity ledger only as a
   paired checkpoint: restore validates every message against the matching
   person origins and original parent-link event history. A later explicit
   relationship removal remains valid and must not invalidate birth history.
   This is local CareOps state and does
   not claim portable Kairos checkpoint support.
6. No maternal vocabulary, birth-message protocol or fetal concepts are added
   to Kairos. The contract lives in CareOps core and can be consumed by ED and
   future birthing modules.

## Consequences

Duplicate delivery is safe at the birth-message boundary, a multiple birth is
atomic, and each recipient can be sent to a different service. Fetal mapping is
explicitly unmodeled and remains a future contract if a pre-birth entity is
introduced. Destination intents do not themselves accept offers, reserve beds,
or complete transfers; callers must create and verify independent transfer
operations.
