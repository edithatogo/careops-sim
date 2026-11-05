// Apache-2.0. Frozen pre-migration public-error oracle; no expected values from derives.
use careops_core::{
    birth::BirthMessageError, capacity::CapacityError, identity::IdentityError,
    transfer::TransferError,
};
use serde_json::Value;
use std::{
    collections::BTreeSet,
    error::Error,
    fmt::Debug,
    panic::{catch_unwind, AssertUnwindSafe},
};
fn assert_copy<T: Copy>(_: &T) {}
fn check<E: Error + Debug + Clone + Eq + Send + Sync + 'static>(
    cases: &Value,
    seen: &mut BTreeSet<String>,
    key: &str,
    error: E,
) {
    assert!(seen.insert(key.to_owned()), "duplicate case {key}");
    let expected = &cases[key];
    assert!(expected["source"].is_null());
    assert!(error.source().is_none(), "unexpected source for {key}");
    assert_eq!(
        error.to_string(),
        expected["display"].as_str().unwrap(),
        "display {key}"
    );
    assert_eq!(
        format!("{error:?}"),
        expected["debug"].as_str().unwrap(),
        "debug {key}"
    );
    assert_eq!(error.clone(), error);
}
fn oracle() -> Value {
    serde_json::from_str(include_str!("fixtures/r57_error_core.json")).unwrap()
}
#[test]
fn all_frozen_variants_messages_traits_and_absent_sources() {
    let data = oracle();
    let cases = &data["cases"];
    let mut seen = BTreeSet::new();
    macro_rules! case {
        ($label:literal, $error:expr) => {
            check(cases, &mut seen, $label, $error)
        };
    }
    assert_copy(&IdentityError::InvalidIdentifier);
    assert_copy(&BirthMessageError::InvalidIdentifier);
    assert_copy(&TransferError::UnsupportedEnvelopeVersion);
    case!(
        "CapacityError::InvalidCapacity0",
        CapacityError::InvalidCapacity("fixture")
    );
    case!(
        "CapacityError::InvalidCapacity1",
        CapacityError::InvalidCapacity("unicode-λ:{opaque}")
    );
    case!(
        "CapacityError::InvalidCapacity2",
        CapacityError::InvalidCapacity("quoted-\"fixture\"\nnext")
    );
    case!(
        "CapacityError::InvalidTransition0",
        CapacityError::InvalidTransition("fixture")
    );
    case!(
        "CapacityError::InvalidTransition1",
        CapacityError::InvalidTransition("unicode-λ:{opaque}")
    );
    case!(
        "CapacityError::InvalidTransition2",
        CapacityError::InvalidTransition("quoted-\"fixture\"\nnext")
    );
    case!(
        "CapacityError::InvalidIdentifier",
        CapacityError::InvalidIdentifier
    );
    case!(
        "CapacityError::InvalidInterval",
        CapacityError::InvalidInterval
    );
    case!(
        "CapacityError::OverlappingIntervals",
        CapacityError::OverlappingIntervals
    );
    case!(
        "CapacityError::ConflictingPool0",
        CapacityError::ConflictingPool("fixture".to_owned())
    );
    case!(
        "CapacityError::ConflictingPool1",
        CapacityError::ConflictingPool("unicode-λ:{opaque}".to_owned())
    );
    case!(
        "CapacityError::ConflictingPool2",
        CapacityError::ConflictingPool("quoted-\"fixture\"\nnext".to_owned())
    );
    case!(
        "CapacityError::IncompatiblePoolAggregate",
        CapacityError::IncompatiblePoolAggregate
    );
    case!(
        "CapacityError::NonexistentLocalTime",
        CapacityError::NonexistentLocalTime
    );
    case!(
        "CapacityError::ArithmeticOverflow",
        CapacityError::ArithmeticOverflow
    );
    case!(
        "IdentityError::InvalidIdentifier",
        IdentityError::InvalidIdentifier
    );
    case!(
        "IdentityError::InvalidSourceKey",
        IdentityError::InvalidSourceKey
    );
    case!("IdentityError::EmptyBatch", IdentityError::EmptyBatch);
    case!(
        "IdentityError::TimeAlreadySealed",
        IdentityError::TimeAlreadySealed
    );
    case!("IdentityError::LateEvent", IdentityError::LateEvent);
    case!("IdentityError::EventConflict", IdentityError::EventConflict);
    case!("IdentityError::IdCollision", IdentityError::IdCollision);
    case!("IdentityError::UnknownPerson", IdentityError::UnknownPerson);
    case!(
        "IdentityError::UnknownEpisode",
        IdentityError::UnknownEpisode
    );
    case!(
        "IdentityError::PersonDeceased",
        IdentityError::PersonDeceased
    );
    case!(
        "IdentityError::OpenEpisodesAtDeath",
        IdentityError::OpenEpisodesAtDeath
    );
    case!(
        "IdentityError::EpisodeAlreadyExited",
        IdentityError::EpisodeAlreadyExited
    );
    case!(
        "IdentityError::InvalidRelationship",
        IdentityError::InvalidRelationship
    );
    case!(
        "IdentityError::RelationshipExists",
        IdentityError::RelationshipExists
    );
    case!(
        "IdentityError::UnknownRelationship",
        IdentityError::UnknownRelationship
    );
    case!(
        "IdentityError::BirthOrdinalExists",
        IdentityError::BirthOrdinalExists
    );
    case!(
        "IdentityError::InvalidCheckpoint",
        IdentityError::InvalidCheckpoint
    );
    case!(
        "BirthMessageError::InvalidIdentifier",
        BirthMessageError::InvalidIdentifier
    );
    case!(
        "BirthMessageError::EmptyInfantSet",
        BirthMessageError::EmptyInfantSet
    );
    case!(
        "BirthMessageError::DuplicateInfantOrdinal",
        BirthMessageError::DuplicateInfantOrdinal
    );
    case!(
        "BirthMessageError::DuplicateEventKey",
        BirthMessageError::DuplicateEventKey
    );
    case!(
        "BirthMessageError::BirthEventAlreadyRegistered",
        BirthMessageError::BirthEventAlreadyRegistered
    );
    case!(
        "BirthMessageError::MessagePayloadConflict",
        BirthMessageError::MessagePayloadConflict
    );
    case!(
        "BirthMessageError::UnknownMother",
        BirthMessageError::UnknownMother
    );
    case!(
        "BirthMessageError::Identity(InvalidIdentifier)",
        BirthMessageError::Identity(IdentityError::InvalidIdentifier)
    );
    case!(
        "BirthMessageError::Identity(InvalidSourceKey)",
        BirthMessageError::Identity(IdentityError::InvalidSourceKey)
    );
    case!(
        "BirthMessageError::Identity(EmptyBatch)",
        BirthMessageError::Identity(IdentityError::EmptyBatch)
    );
    case!(
        "BirthMessageError::Identity(TimeAlreadySealed)",
        BirthMessageError::Identity(IdentityError::TimeAlreadySealed)
    );
    case!(
        "BirthMessageError::Identity(LateEvent)",
        BirthMessageError::Identity(IdentityError::LateEvent)
    );
    case!(
        "BirthMessageError::Identity(EventConflict)",
        BirthMessageError::Identity(IdentityError::EventConflict)
    );
    case!(
        "BirthMessageError::Identity(IdCollision)",
        BirthMessageError::Identity(IdentityError::IdCollision)
    );
    case!(
        "BirthMessageError::Identity(UnknownPerson)",
        BirthMessageError::Identity(IdentityError::UnknownPerson)
    );
    case!(
        "BirthMessageError::Identity(UnknownEpisode)",
        BirthMessageError::Identity(IdentityError::UnknownEpisode)
    );
    case!(
        "BirthMessageError::Identity(PersonDeceased)",
        BirthMessageError::Identity(IdentityError::PersonDeceased)
    );
    case!(
        "BirthMessageError::Identity(OpenEpisodesAtDeath)",
        BirthMessageError::Identity(IdentityError::OpenEpisodesAtDeath)
    );
    case!(
        "BirthMessageError::Identity(EpisodeAlreadyExited)",
        BirthMessageError::Identity(IdentityError::EpisodeAlreadyExited)
    );
    case!(
        "BirthMessageError::Identity(InvalidRelationship)",
        BirthMessageError::Identity(IdentityError::InvalidRelationship)
    );
    case!(
        "BirthMessageError::Identity(RelationshipExists)",
        BirthMessageError::Identity(IdentityError::RelationshipExists)
    );
    case!(
        "BirthMessageError::Identity(UnknownRelationship)",
        BirthMessageError::Identity(IdentityError::UnknownRelationship)
    );
    case!(
        "BirthMessageError::Identity(BirthOrdinalExists)",
        BirthMessageError::Identity(IdentityError::BirthOrdinalExists)
    );
    case!(
        "BirthMessageError::Identity(InvalidCheckpoint)",
        BirthMessageError::Identity(IdentityError::InvalidCheckpoint)
    );
    case!(
        "BirthMessageError::InvalidCheckpoint",
        BirthMessageError::InvalidCheckpoint
    );
    case!(
        "TransferError::UnsupportedEnvelopeVersion",
        TransferError::UnsupportedEnvelopeVersion
    );
    case!(
        "TransferError::InvalidIdentifier",
        TransferError::InvalidIdentifier
    );
    case!(
        "TransferError::InvalidRequest",
        TransferError::InvalidRequest
    );
    case!(
        "TransferError::UnknownOperation",
        TransferError::UnknownOperation
    );
    case!("TransferError::UnknownPerson", TransferError::UnknownPerson);
    case!(
        "TransferError::InvalidTransition",
        TransferError::InvalidTransition
    );
    case!(
        "TransferError::RevisionOverflow",
        TransferError::RevisionOverflow
    );
    case!(
        "TransferError::RevisionGapOverflow",
        TransferError::RevisionGapOverflow
    );
    case!(
        "TransferError::MessageConflict",
        TransferError::MessageConflict
    );
    case!(
        "TransferError::TooManyPendingMessages",
        TransferError::TooManyPendingMessages
    );
    case!(
        "TransferError::ResourceConflict",
        TransferError::ResourceConflict
    );
    case!(
        "TransferError::TimeRegression",
        TransferError::TimeRegression
    );
    case!(
        "TransferError::SequenceOverflow",
        TransferError::SequenceOverflow
    );
    case!(
        "TransferError::InvalidCheckpoint",
        TransferError::InvalidCheckpoint
    );
    assert_eq!(seen.len(), 71);
    assert_eq!(seen.len(), cases.as_object().unwrap().len());
    println!("validated {} frozen public error cases", seen.len());
}

#[derive(Clone, PartialEq, Eq)]
struct Counterfactual {
    message: String,
    debug: String,
    has_source: bool,
}
impl std::fmt::Debug for Counterfactual {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.debug)
    }
}
impl std::fmt::Display for Counterfactual {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}
impl Error for Counterfactual {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        static INNER: IdentityError = IdentityError::UnknownPerson;
        if self.has_source {
            Some(&INNER)
        } else {
            None
        }
    }
}
#[test]
fn oracle_refuses_message_and_source_regressions() {
    let data = oracle();
    let cases = &data["cases"];
    let key = "CapacityError::InvalidCapacity0";
    for has_source in [false, true] {
        let mut message = cases[key]["display"].as_str().unwrap().to_owned();
        if !has_source {
            message.push_str(" changed");
        }
        let error = Counterfactual {
            message,
            debug: cases[key]["debug"].as_str().unwrap().to_owned(),
            has_source,
        };
        assert!(catch_unwind(AssertUnwindSafe(|| check(
            cases,
            &mut BTreeSet::new(),
            key,
            error
        )))
        .is_err());
    }
}
#[test]
fn nested_identity_from_preserves_variant_and_absent_source() {
    let error = BirthMessageError::from(IdentityError::UnknownPerson);
    assert_eq!(
        error,
        BirthMessageError::Identity(IdentityError::UnknownPerson)
    );
    assert!(error.source().is_none());
    assert_eq!(
        error.to_string(),
        "identity command references an unknown person"
    );
}
