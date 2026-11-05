// Apache-2.0. Frozen pre-migration public-error oracle; no expected values from derives.
use careops_core::identity::IdentityError;
use careops_ed::{
    arrivals::ArrivalError,
    disposition::{composite_fixture::BoundaryError, operations::OperationalError},
    identity::EdIdentityError,
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
    serde_json::from_str(include_str!("fixtures/r57_error_ed.json")).unwrap()
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
    assert_copy(&EdIdentityError::InvalidSourceKey);
    assert_copy(&OperationalError::Ineligible);
    assert_copy(&BoundaryError::IncompatibleOfferDestination);
    case!("ArrivalError::InvalidVersion", ArrivalError::InvalidVersion);
    case!(
        "ArrivalError::InvalidProvenance",
        ArrivalError::InvalidProvenance
    );
    case!("ArrivalError::InvalidHorizon", ArrivalError::InvalidHorizon);
    case!("ArrivalError::LimitExceeded", ArrivalError::LimitExceeded);
    case!("ArrivalError::InvalidMode", ArrivalError::InvalidMode);
    case!("ArrivalError::DuplicateMode", ArrivalError::DuplicateMode);
    case!("ArrivalError::InvalidBin", ArrivalError::InvalidBin);
    case!(
        "ArrivalError::OverlappingBins",
        ArrivalError::OverlappingBins
    );
    case!("ArrivalError::CountMismatch", ArrivalError::CountMismatch);
    case!(
        "ArrivalError::ArithmeticOverflow",
        ArrivalError::ArithmeticOverflow
    );
    case!(
        "ArrivalError::SamplingExhausted",
        ArrivalError::SamplingExhausted
    );
    case!(
        "EdIdentityError::InvalidSourceKey",
        EdIdentityError::InvalidSourceKey
    );
    case!(
        "EdIdentityError::SourceKeyCollision",
        EdIdentityError::SourceKeyCollision
    );
    case!(
        "EdIdentityError::AdmissionKeyConflict",
        EdIdentityError::AdmissionKeyConflict
    );
    case!("EdIdentityError::IdCollision", EdIdentityError::IdCollision);
    case!(
        "EdIdentityError::Ledger(InvalidIdentifier)",
        EdIdentityError::Ledger(IdentityError::InvalidIdentifier)
    );
    case!(
        "EdIdentityError::Ledger(InvalidSourceKey)",
        EdIdentityError::Ledger(IdentityError::InvalidSourceKey)
    );
    case!(
        "EdIdentityError::Ledger(EmptyBatch)",
        EdIdentityError::Ledger(IdentityError::EmptyBatch)
    );
    case!(
        "EdIdentityError::Ledger(TimeAlreadySealed)",
        EdIdentityError::Ledger(IdentityError::TimeAlreadySealed)
    );
    case!(
        "EdIdentityError::Ledger(LateEvent)",
        EdIdentityError::Ledger(IdentityError::LateEvent)
    );
    case!(
        "EdIdentityError::Ledger(EventConflict)",
        EdIdentityError::Ledger(IdentityError::EventConflict)
    );
    case!(
        "EdIdentityError::Ledger(IdCollision)",
        EdIdentityError::Ledger(IdentityError::IdCollision)
    );
    case!(
        "EdIdentityError::Ledger(UnknownPerson)",
        EdIdentityError::Ledger(IdentityError::UnknownPerson)
    );
    case!(
        "EdIdentityError::Ledger(UnknownEpisode)",
        EdIdentityError::Ledger(IdentityError::UnknownEpisode)
    );
    case!(
        "EdIdentityError::Ledger(PersonDeceased)",
        EdIdentityError::Ledger(IdentityError::PersonDeceased)
    );
    case!(
        "EdIdentityError::Ledger(OpenEpisodesAtDeath)",
        EdIdentityError::Ledger(IdentityError::OpenEpisodesAtDeath)
    );
    case!(
        "EdIdentityError::Ledger(EpisodeAlreadyExited)",
        EdIdentityError::Ledger(IdentityError::EpisodeAlreadyExited)
    );
    case!(
        "EdIdentityError::Ledger(InvalidRelationship)",
        EdIdentityError::Ledger(IdentityError::InvalidRelationship)
    );
    case!(
        "EdIdentityError::Ledger(RelationshipExists)",
        EdIdentityError::Ledger(IdentityError::RelationshipExists)
    );
    case!(
        "EdIdentityError::Ledger(UnknownRelationship)",
        EdIdentityError::Ledger(IdentityError::UnknownRelationship)
    );
    case!(
        "EdIdentityError::Ledger(BirthOrdinalExists)",
        EdIdentityError::Ledger(IdentityError::BirthOrdinalExists)
    );
    case!(
        "EdIdentityError::Ledger(InvalidCheckpoint)",
        EdIdentityError::Ledger(IdentityError::InvalidCheckpoint)
    );
    case!("OperationalError::Ineligible", OperationalError::Ineligible);
    case!(
        "OperationalError::Unavailable",
        OperationalError::Unavailable
    );
    case!(
        "OperationalError::CapacityInUse",
        OperationalError::CapacityInUse
    );
    case!(
        "OperationalError::NotAssigned",
        OperationalError::NotAssigned
    );
    case!("OperationalError::Accounting", OperationalError::Accounting);
    case!(
        "OperationalError::SpaceUnavailable",
        OperationalError::SpaceUnavailable
    );
    case!(
        "OperationalError::ReservedForOther",
        OperationalError::ReservedForOther
    );
    case!(
        "OperationalError::CleaningInProgress",
        OperationalError::CleaningInProgress
    );
    case!(
        "OperationalError::DiagnosticBlocked",
        OperationalError::DiagnosticBlocked
    );
    case!(
        "OperationalError::BoardingRetained",
        OperationalError::BoardingRetained
    );
    case!(
        "OperationalError::AtomicMultiResourceUnsupported",
        OperationalError::AtomicMultiResourceUnsupported
    );
    case!(
        "OperationalError::UnsupportedMultiRole",
        OperationalError::UnsupportedMultiRole
    );
    case!(
        "OperationalError::RouteUnreachable",
        OperationalError::RouteUnreachable
    );
    case!(
        "OperationalError::EdgeInaccessible",
        OperationalError::EdgeInaccessible
    );
    case!(
        "BoundaryError::IncompatibleOfferDestination",
        BoundaryError::IncompatibleOfferDestination
    );
    case!(
        "BoundaryError::ImmediateExpiryDefaultProhibited",
        BoundaryError::ImmediateExpiryDefaultProhibited
    );
    case!(
        "BoundaryError::PrematureBayReleaseBeforePhysicalTransfer",
        BoundaryError::PrematureBayReleaseBeforePhysicalTransfer
    );
    case!(
        "BoundaryError::ExcessiveBoarderStaffReservation",
        BoundaryError::ExcessiveBoarderStaffReservation
    );
    case!(
        "BoundaryError::AmbulanceOffloadBlockedBySaturation",
        BoundaryError::AmbulanceOffloadBlockedBySaturation
    );
    case!(
        "BoundaryError::ReceiverStaffUnavailable",
        BoundaryError::ReceiverStaffUnavailable
    );
    case!(
        "BoundaryError::ProhibitedCausalWardPolicyClaim",
        BoundaryError::ProhibitedCausalWardPolicyClaim
    );
    case!(
        "BoundaryError::UnsupportedImplicitDeteriorationHazard",
        BoundaryError::UnsupportedImplicitDeteriorationHazard
    );
    assert_eq!(seen.len(), 54);
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
    let key = "ArrivalError::InvalidVersion";
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
    let error = EdIdentityError::from(IdentityError::UnknownPerson);
    assert_eq!(error, EdIdentityError::Ledger(IdentityError::UnknownPerson));
    assert!(error.source().is_none());
    assert_eq!(
        error.to_string(),
        "identity command references an unknown person"
    );
}
