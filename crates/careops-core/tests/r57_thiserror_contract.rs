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
    {
        let error = CapacityError::InvalidCapacity("fixture");
        check(cases, &mut seen, "CapacityError::InvalidCapacity0", error);
    }
    {
        let error = CapacityError::InvalidCapacity("unicode-λ:{opaque}");
        check(cases, &mut seen, "CapacityError::InvalidCapacity1", error);
    }
    {
        let error = CapacityError::InvalidCapacity("quoted-\"fixture\"\nnext");
        check(cases, &mut seen, "CapacityError::InvalidCapacity2", error);
    }
    {
        let error = CapacityError::InvalidTransition("fixture");
        check(cases, &mut seen, "CapacityError::InvalidTransition0", error);
    }
    {
        let error = CapacityError::InvalidTransition("unicode-λ:{opaque}");
        check(cases, &mut seen, "CapacityError::InvalidTransition1", error);
    }
    {
        let error = CapacityError::InvalidTransition("quoted-\"fixture\"\nnext");
        check(cases, &mut seen, "CapacityError::InvalidTransition2", error);
    }
    {
        let error = CapacityError::InvalidIdentifier;
        check(cases, &mut seen, "CapacityError::InvalidIdentifier", error);
    }
    {
        let error = CapacityError::InvalidInterval;
        check(cases, &mut seen, "CapacityError::InvalidInterval", error);
    }
    {
        let error = CapacityError::OverlappingIntervals;
        check(
            cases,
            &mut seen,
            "CapacityError::OverlappingIntervals",
            error,
        );
    }
    {
        let error = CapacityError::ConflictingPool("fixture".to_owned());
        check(cases, &mut seen, "CapacityError::ConflictingPool0", error);
    }
    {
        let error = CapacityError::ConflictingPool("unicode-λ:{opaque}".to_owned());
        check(cases, &mut seen, "CapacityError::ConflictingPool1", error);
    }
    {
        let error = CapacityError::ConflictingPool("quoted-\"fixture\"\nnext".to_owned());
        check(cases, &mut seen, "CapacityError::ConflictingPool2", error);
    }
    {
        let error = CapacityError::IncompatiblePoolAggregate;
        check(
            cases,
            &mut seen,
            "CapacityError::IncompatiblePoolAggregate",
            error,
        );
    }
    {
        let error = CapacityError::NonexistentLocalTime;
        check(
            cases,
            &mut seen,
            "CapacityError::NonexistentLocalTime",
            error,
        );
    }
    {
        let error = CapacityError::ArithmeticOverflow;
        check(cases, &mut seen, "CapacityError::ArithmeticOverflow", error);
    }
    {
        let error = IdentityError::InvalidIdentifier;
        assert_copy(&error);
        check(cases, &mut seen, "IdentityError::InvalidIdentifier", error);
    }
    {
        let error = IdentityError::InvalidSourceKey;
        assert_copy(&error);
        check(cases, &mut seen, "IdentityError::InvalidSourceKey", error);
    }
    {
        let error = IdentityError::EmptyBatch;
        assert_copy(&error);
        check(cases, &mut seen, "IdentityError::EmptyBatch", error);
    }
    {
        let error = IdentityError::TimeAlreadySealed;
        assert_copy(&error);
        check(cases, &mut seen, "IdentityError::TimeAlreadySealed", error);
    }
    {
        let error = IdentityError::LateEvent;
        assert_copy(&error);
        check(cases, &mut seen, "IdentityError::LateEvent", error);
    }
    {
        let error = IdentityError::EventConflict;
        assert_copy(&error);
        check(cases, &mut seen, "IdentityError::EventConflict", error);
    }
    {
        let error = IdentityError::IdCollision;
        assert_copy(&error);
        check(cases, &mut seen, "IdentityError::IdCollision", error);
    }
    {
        let error = IdentityError::UnknownPerson;
        assert_copy(&error);
        check(cases, &mut seen, "IdentityError::UnknownPerson", error);
    }
    {
        let error = IdentityError::UnknownEpisode;
        assert_copy(&error);
        check(cases, &mut seen, "IdentityError::UnknownEpisode", error);
    }
    {
        let error = IdentityError::PersonDeceased;
        assert_copy(&error);
        check(cases, &mut seen, "IdentityError::PersonDeceased", error);
    }
    {
        let error = IdentityError::OpenEpisodesAtDeath;
        assert_copy(&error);
        check(
            cases,
            &mut seen,
            "IdentityError::OpenEpisodesAtDeath",
            error,
        );
    }
    {
        let error = IdentityError::EpisodeAlreadyExited;
        assert_copy(&error);
        check(
            cases,
            &mut seen,
            "IdentityError::EpisodeAlreadyExited",
            error,
        );
    }
    {
        let error = IdentityError::InvalidRelationship;
        assert_copy(&error);
        check(
            cases,
            &mut seen,
            "IdentityError::InvalidRelationship",
            error,
        );
    }
    {
        let error = IdentityError::RelationshipExists;
        assert_copy(&error);
        check(cases, &mut seen, "IdentityError::RelationshipExists", error);
    }
    {
        let error = IdentityError::UnknownRelationship;
        assert_copy(&error);
        check(
            cases,
            &mut seen,
            "IdentityError::UnknownRelationship",
            error,
        );
    }
    {
        let error = IdentityError::BirthOrdinalExists;
        assert_copy(&error);
        check(cases, &mut seen, "IdentityError::BirthOrdinalExists", error);
    }
    {
        let error = IdentityError::InvalidCheckpoint;
        assert_copy(&error);
        check(cases, &mut seen, "IdentityError::InvalidCheckpoint", error);
    }
    {
        let error = BirthMessageError::InvalidIdentifier;
        assert_copy(&error);
        check(
            cases,
            &mut seen,
            "BirthMessageError::InvalidIdentifier",
            error,
        );
    }
    {
        let error = BirthMessageError::EmptyInfantSet;
        assert_copy(&error);
        check(cases, &mut seen, "BirthMessageError::EmptyInfantSet", error);
    }
    {
        let error = BirthMessageError::DuplicateInfantOrdinal;
        assert_copy(&error);
        check(
            cases,
            &mut seen,
            "BirthMessageError::DuplicateInfantOrdinal",
            error,
        );
    }
    {
        let error = BirthMessageError::DuplicateEventKey;
        assert_copy(&error);
        check(
            cases,
            &mut seen,
            "BirthMessageError::DuplicateEventKey",
            error,
        );
    }
    {
        let error = BirthMessageError::BirthEventAlreadyRegistered;
        assert_copy(&error);
        check(
            cases,
            &mut seen,
            "BirthMessageError::BirthEventAlreadyRegistered",
            error,
        );
    }
    {
        let error = BirthMessageError::MessagePayloadConflict;
        assert_copy(&error);
        check(
            cases,
            &mut seen,
            "BirthMessageError::MessagePayloadConflict",
            error,
        );
    }
    {
        let error = BirthMessageError::UnknownMother;
        assert_copy(&error);
        check(cases, &mut seen, "BirthMessageError::UnknownMother", error);
    }
    {
        let error = BirthMessageError::Identity(IdentityError::InvalidIdentifier);
        assert_copy(&error);
        check(
            cases,
            &mut seen,
            "BirthMessageError::Identity(InvalidIdentifier)",
            error,
        );
    }
    {
        let error = BirthMessageError::Identity(IdentityError::InvalidSourceKey);
        assert_copy(&error);
        check(
            cases,
            &mut seen,
            "BirthMessageError::Identity(InvalidSourceKey)",
            error,
        );
    }
    {
        let error = BirthMessageError::Identity(IdentityError::EmptyBatch);
        assert_copy(&error);
        check(
            cases,
            &mut seen,
            "BirthMessageError::Identity(EmptyBatch)",
            error,
        );
    }
    {
        let error = BirthMessageError::Identity(IdentityError::TimeAlreadySealed);
        assert_copy(&error);
        check(
            cases,
            &mut seen,
            "BirthMessageError::Identity(TimeAlreadySealed)",
            error,
        );
    }
    {
        let error = BirthMessageError::Identity(IdentityError::LateEvent);
        assert_copy(&error);
        check(
            cases,
            &mut seen,
            "BirthMessageError::Identity(LateEvent)",
            error,
        );
    }
    {
        let error = BirthMessageError::Identity(IdentityError::EventConflict);
        assert_copy(&error);
        check(
            cases,
            &mut seen,
            "BirthMessageError::Identity(EventConflict)",
            error,
        );
    }
    {
        let error = BirthMessageError::Identity(IdentityError::IdCollision);
        assert_copy(&error);
        check(
            cases,
            &mut seen,
            "BirthMessageError::Identity(IdCollision)",
            error,
        );
    }
    {
        let error = BirthMessageError::Identity(IdentityError::UnknownPerson);
        assert_copy(&error);
        check(
            cases,
            &mut seen,
            "BirthMessageError::Identity(UnknownPerson)",
            error,
        );
    }
    {
        let error = BirthMessageError::Identity(IdentityError::UnknownEpisode);
        assert_copy(&error);
        check(
            cases,
            &mut seen,
            "BirthMessageError::Identity(UnknownEpisode)",
            error,
        );
    }
    {
        let error = BirthMessageError::Identity(IdentityError::PersonDeceased);
        assert_copy(&error);
        check(
            cases,
            &mut seen,
            "BirthMessageError::Identity(PersonDeceased)",
            error,
        );
    }
    {
        let error = BirthMessageError::Identity(IdentityError::OpenEpisodesAtDeath);
        assert_copy(&error);
        check(
            cases,
            &mut seen,
            "BirthMessageError::Identity(OpenEpisodesAtDeath)",
            error,
        );
    }
    {
        let error = BirthMessageError::Identity(IdentityError::EpisodeAlreadyExited);
        assert_copy(&error);
        check(
            cases,
            &mut seen,
            "BirthMessageError::Identity(EpisodeAlreadyExited)",
            error,
        );
    }
    {
        let error = BirthMessageError::Identity(IdentityError::InvalidRelationship);
        assert_copy(&error);
        check(
            cases,
            &mut seen,
            "BirthMessageError::Identity(InvalidRelationship)",
            error,
        );
    }
    {
        let error = BirthMessageError::Identity(IdentityError::RelationshipExists);
        assert_copy(&error);
        check(
            cases,
            &mut seen,
            "BirthMessageError::Identity(RelationshipExists)",
            error,
        );
    }
    {
        let error = BirthMessageError::Identity(IdentityError::UnknownRelationship);
        assert_copy(&error);
        check(
            cases,
            &mut seen,
            "BirthMessageError::Identity(UnknownRelationship)",
            error,
        );
    }
    {
        let error = BirthMessageError::Identity(IdentityError::BirthOrdinalExists);
        assert_copy(&error);
        check(
            cases,
            &mut seen,
            "BirthMessageError::Identity(BirthOrdinalExists)",
            error,
        );
    }
    {
        let error = BirthMessageError::Identity(IdentityError::InvalidCheckpoint);
        assert_copy(&error);
        check(
            cases,
            &mut seen,
            "BirthMessageError::Identity(InvalidCheckpoint)",
            error,
        );
    }
    {
        let error = BirthMessageError::InvalidCheckpoint;
        assert_copy(&error);
        check(
            cases,
            &mut seen,
            "BirthMessageError::InvalidCheckpoint",
            error,
        );
    }
    {
        let error = TransferError::UnsupportedEnvelopeVersion;
        assert_copy(&error);
        check(
            cases,
            &mut seen,
            "TransferError::UnsupportedEnvelopeVersion",
            error,
        );
    }
    {
        let error = TransferError::InvalidIdentifier;
        assert_copy(&error);
        check(cases, &mut seen, "TransferError::InvalidIdentifier", error);
    }
    {
        let error = TransferError::InvalidRequest;
        assert_copy(&error);
        check(cases, &mut seen, "TransferError::InvalidRequest", error);
    }
    {
        let error = TransferError::UnknownOperation;
        assert_copy(&error);
        check(cases, &mut seen, "TransferError::UnknownOperation", error);
    }
    {
        let error = TransferError::UnknownPerson;
        assert_copy(&error);
        check(cases, &mut seen, "TransferError::UnknownPerson", error);
    }
    {
        let error = TransferError::InvalidTransition;
        assert_copy(&error);
        check(cases, &mut seen, "TransferError::InvalidTransition", error);
    }
    {
        let error = TransferError::RevisionOverflow;
        assert_copy(&error);
        check(cases, &mut seen, "TransferError::RevisionOverflow", error);
    }
    {
        let error = TransferError::RevisionGapOverflow;
        assert_copy(&error);
        check(
            cases,
            &mut seen,
            "TransferError::RevisionGapOverflow",
            error,
        );
    }
    {
        let error = TransferError::MessageConflict;
        assert_copy(&error);
        check(cases, &mut seen, "TransferError::MessageConflict", error);
    }
    {
        let error = TransferError::TooManyPendingMessages;
        assert_copy(&error);
        check(
            cases,
            &mut seen,
            "TransferError::TooManyPendingMessages",
            error,
        );
    }
    {
        let error = TransferError::ResourceConflict;
        assert_copy(&error);
        check(cases, &mut seen, "TransferError::ResourceConflict", error);
    }
    {
        let error = TransferError::TimeRegression;
        assert_copy(&error);
        check(cases, &mut seen, "TransferError::TimeRegression", error);
    }
    {
        let error = TransferError::SequenceOverflow;
        assert_copy(&error);
        check(cases, &mut seen, "TransferError::SequenceOverflow", error);
    }
    {
        let error = TransferError::InvalidCheckpoint;
        assert_copy(&error);
        check(cases, &mut seen, "TransferError::InvalidCheckpoint", error);
    }
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
