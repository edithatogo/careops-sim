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
    {
        let error = ArrivalError::InvalidVersion;
        check(cases, &mut seen, "ArrivalError::InvalidVersion", error);
    }
    {
        let error = ArrivalError::InvalidProvenance;
        check(cases, &mut seen, "ArrivalError::InvalidProvenance", error);
    }
    {
        let error = ArrivalError::InvalidHorizon;
        check(cases, &mut seen, "ArrivalError::InvalidHorizon", error);
    }
    {
        let error = ArrivalError::LimitExceeded;
        check(cases, &mut seen, "ArrivalError::LimitExceeded", error);
    }
    {
        let error = ArrivalError::InvalidMode;
        check(cases, &mut seen, "ArrivalError::InvalidMode", error);
    }
    {
        let error = ArrivalError::DuplicateMode;
        check(cases, &mut seen, "ArrivalError::DuplicateMode", error);
    }
    {
        let error = ArrivalError::InvalidBin;
        check(cases, &mut seen, "ArrivalError::InvalidBin", error);
    }
    {
        let error = ArrivalError::OverlappingBins;
        check(cases, &mut seen, "ArrivalError::OverlappingBins", error);
    }
    {
        let error = ArrivalError::CountMismatch;
        check(cases, &mut seen, "ArrivalError::CountMismatch", error);
    }
    {
        let error = ArrivalError::ArithmeticOverflow;
        check(cases, &mut seen, "ArrivalError::ArithmeticOverflow", error);
    }
    {
        let error = ArrivalError::SamplingExhausted;
        check(cases, &mut seen, "ArrivalError::SamplingExhausted", error);
    }
    {
        let error = EdIdentityError::InvalidSourceKey;
        assert_copy(&error);
        check(cases, &mut seen, "EdIdentityError::InvalidSourceKey", error);
    }
    {
        let error = EdIdentityError::SourceKeyCollision;
        assert_copy(&error);
        check(
            cases,
            &mut seen,
            "EdIdentityError::SourceKeyCollision",
            error,
        );
    }
    {
        let error = EdIdentityError::AdmissionKeyConflict;
        assert_copy(&error);
        check(
            cases,
            &mut seen,
            "EdIdentityError::AdmissionKeyConflict",
            error,
        );
    }
    {
        let error = EdIdentityError::IdCollision;
        assert_copy(&error);
        check(cases, &mut seen, "EdIdentityError::IdCollision", error);
    }
    {
        let error = EdIdentityError::Ledger(IdentityError::InvalidIdentifier);
        assert_copy(&error);
        check(
            cases,
            &mut seen,
            "EdIdentityError::Ledger(InvalidIdentifier)",
            error,
        );
    }
    {
        let error = EdIdentityError::Ledger(IdentityError::InvalidSourceKey);
        assert_copy(&error);
        check(
            cases,
            &mut seen,
            "EdIdentityError::Ledger(InvalidSourceKey)",
            error,
        );
    }
    {
        let error = EdIdentityError::Ledger(IdentityError::EmptyBatch);
        assert_copy(&error);
        check(
            cases,
            &mut seen,
            "EdIdentityError::Ledger(EmptyBatch)",
            error,
        );
    }
    {
        let error = EdIdentityError::Ledger(IdentityError::TimeAlreadySealed);
        assert_copy(&error);
        check(
            cases,
            &mut seen,
            "EdIdentityError::Ledger(TimeAlreadySealed)",
            error,
        );
    }
    {
        let error = EdIdentityError::Ledger(IdentityError::LateEvent);
        assert_copy(&error);
        check(
            cases,
            &mut seen,
            "EdIdentityError::Ledger(LateEvent)",
            error,
        );
    }
    {
        let error = EdIdentityError::Ledger(IdentityError::EventConflict);
        assert_copy(&error);
        check(
            cases,
            &mut seen,
            "EdIdentityError::Ledger(EventConflict)",
            error,
        );
    }
    {
        let error = EdIdentityError::Ledger(IdentityError::IdCollision);
        assert_copy(&error);
        check(
            cases,
            &mut seen,
            "EdIdentityError::Ledger(IdCollision)",
            error,
        );
    }
    {
        let error = EdIdentityError::Ledger(IdentityError::UnknownPerson);
        assert_copy(&error);
        check(
            cases,
            &mut seen,
            "EdIdentityError::Ledger(UnknownPerson)",
            error,
        );
    }
    {
        let error = EdIdentityError::Ledger(IdentityError::UnknownEpisode);
        assert_copy(&error);
        check(
            cases,
            &mut seen,
            "EdIdentityError::Ledger(UnknownEpisode)",
            error,
        );
    }
    {
        let error = EdIdentityError::Ledger(IdentityError::PersonDeceased);
        assert_copy(&error);
        check(
            cases,
            &mut seen,
            "EdIdentityError::Ledger(PersonDeceased)",
            error,
        );
    }
    {
        let error = EdIdentityError::Ledger(IdentityError::OpenEpisodesAtDeath);
        assert_copy(&error);
        check(
            cases,
            &mut seen,
            "EdIdentityError::Ledger(OpenEpisodesAtDeath)",
            error,
        );
    }
    {
        let error = EdIdentityError::Ledger(IdentityError::EpisodeAlreadyExited);
        assert_copy(&error);
        check(
            cases,
            &mut seen,
            "EdIdentityError::Ledger(EpisodeAlreadyExited)",
            error,
        );
    }
    {
        let error = EdIdentityError::Ledger(IdentityError::InvalidRelationship);
        assert_copy(&error);
        check(
            cases,
            &mut seen,
            "EdIdentityError::Ledger(InvalidRelationship)",
            error,
        );
    }
    {
        let error = EdIdentityError::Ledger(IdentityError::RelationshipExists);
        assert_copy(&error);
        check(
            cases,
            &mut seen,
            "EdIdentityError::Ledger(RelationshipExists)",
            error,
        );
    }
    {
        let error = EdIdentityError::Ledger(IdentityError::UnknownRelationship);
        assert_copy(&error);
        check(
            cases,
            &mut seen,
            "EdIdentityError::Ledger(UnknownRelationship)",
            error,
        );
    }
    {
        let error = EdIdentityError::Ledger(IdentityError::BirthOrdinalExists);
        assert_copy(&error);
        check(
            cases,
            &mut seen,
            "EdIdentityError::Ledger(BirthOrdinalExists)",
            error,
        );
    }
    {
        let error = EdIdentityError::Ledger(IdentityError::InvalidCheckpoint);
        assert_copy(&error);
        check(
            cases,
            &mut seen,
            "EdIdentityError::Ledger(InvalidCheckpoint)",
            error,
        );
    }
    {
        let error = OperationalError::Ineligible;
        assert_copy(&error);
        check(cases, &mut seen, "OperationalError::Ineligible", error);
    }
    {
        let error = OperationalError::Unavailable;
        assert_copy(&error);
        check(cases, &mut seen, "OperationalError::Unavailable", error);
    }
    {
        let error = OperationalError::CapacityInUse;
        assert_copy(&error);
        check(cases, &mut seen, "OperationalError::CapacityInUse", error);
    }
    {
        let error = OperationalError::NotAssigned;
        assert_copy(&error);
        check(cases, &mut seen, "OperationalError::NotAssigned", error);
    }
    {
        let error = OperationalError::Accounting;
        assert_copy(&error);
        check(cases, &mut seen, "OperationalError::Accounting", error);
    }
    {
        let error = OperationalError::SpaceUnavailable;
        assert_copy(&error);
        check(
            cases,
            &mut seen,
            "OperationalError::SpaceUnavailable",
            error,
        );
    }
    {
        let error = OperationalError::ReservedForOther;
        assert_copy(&error);
        check(
            cases,
            &mut seen,
            "OperationalError::ReservedForOther",
            error,
        );
    }
    {
        let error = OperationalError::CleaningInProgress;
        assert_copy(&error);
        check(
            cases,
            &mut seen,
            "OperationalError::CleaningInProgress",
            error,
        );
    }
    {
        let error = OperationalError::DiagnosticBlocked;
        assert_copy(&error);
        check(
            cases,
            &mut seen,
            "OperationalError::DiagnosticBlocked",
            error,
        );
    }
    {
        let error = OperationalError::BoardingRetained;
        assert_copy(&error);
        check(
            cases,
            &mut seen,
            "OperationalError::BoardingRetained",
            error,
        );
    }
    {
        let error = OperationalError::AtomicMultiResourceUnsupported;
        assert_copy(&error);
        check(
            cases,
            &mut seen,
            "OperationalError::AtomicMultiResourceUnsupported",
            error,
        );
    }
    {
        let error = OperationalError::UnsupportedMultiRole;
        assert_copy(&error);
        check(
            cases,
            &mut seen,
            "OperationalError::UnsupportedMultiRole",
            error,
        );
    }
    {
        let error = OperationalError::RouteUnreachable;
        assert_copy(&error);
        check(
            cases,
            &mut seen,
            "OperationalError::RouteUnreachable",
            error,
        );
    }
    {
        let error = OperationalError::EdgeInaccessible;
        assert_copy(&error);
        check(
            cases,
            &mut seen,
            "OperationalError::EdgeInaccessible",
            error,
        );
    }
    {
        let error = BoundaryError::IncompatibleOfferDestination;
        assert_copy(&error);
        check(
            cases,
            &mut seen,
            "BoundaryError::IncompatibleOfferDestination",
            error,
        );
    }
    {
        let error = BoundaryError::ImmediateExpiryDefaultProhibited;
        assert_copy(&error);
        check(
            cases,
            &mut seen,
            "BoundaryError::ImmediateExpiryDefaultProhibited",
            error,
        );
    }
    {
        let error = BoundaryError::PrematureBayReleaseBeforePhysicalTransfer;
        assert_copy(&error);
        check(
            cases,
            &mut seen,
            "BoundaryError::PrematureBayReleaseBeforePhysicalTransfer",
            error,
        );
    }
    {
        let error = BoundaryError::ExcessiveBoarderStaffReservation;
        assert_copy(&error);
        check(
            cases,
            &mut seen,
            "BoundaryError::ExcessiveBoarderStaffReservation",
            error,
        );
    }
    {
        let error = BoundaryError::AmbulanceOffloadBlockedBySaturation;
        assert_copy(&error);
        check(
            cases,
            &mut seen,
            "BoundaryError::AmbulanceOffloadBlockedBySaturation",
            error,
        );
    }
    {
        let error = BoundaryError::ReceiverStaffUnavailable;
        assert_copy(&error);
        check(
            cases,
            &mut seen,
            "BoundaryError::ReceiverStaffUnavailable",
            error,
        );
    }
    {
        let error = BoundaryError::ProhibitedCausalWardPolicyClaim;
        assert_copy(&error);
        check(
            cases,
            &mut seen,
            "BoundaryError::ProhibitedCausalWardPolicyClaim",
            error,
        );
    }
    {
        let error = BoundaryError::UnsupportedImplicitDeteriorationHazard;
        assert_copy(&error);
        check(
            cases,
            &mut seen,
            "BoundaryError::UnsupportedImplicitDeteriorationHazard",
            error,
        );
    }
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
