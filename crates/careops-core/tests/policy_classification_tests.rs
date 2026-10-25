use careops_core::pathway::{
    ClassificationHistory, DecisionPolicy, DispatchPriority, InterruptionEligibility,
    ObservedPolicyState, PolicyClockState, PolicyDecision, ServiceTargetStatus, UrgencyCategory,
    VersionedPolicyClock, VersionedUrgencyAssessment,
};
use careops_core::profile::JurisdictionProfile;

#[test]
fn assessment_captures_profile_version_and_provenance_and_history_is_append_only() {
    let mut profile = JurisdictionProfile::default();
    profile.provenance.source_sha256 = Some("source-hash".into());
    let first = VersionedUrgencyAssessment::from_profile(&profile, 2, 4).unwrap();
    assert_eq!(first.scheme_version, "2026.1");
    assert_eq!(first.source_sha256.as_deref(), Some("source-hash"));
    assert!(VersionedUrgencyAssessment::from_profile(&profile, 99, 5).is_err());
    let mut history = ClassificationHistory::default();
    history.reassess(first.clone()).unwrap();
    let mut revised = profile.clone();
    revised.urgency_scheme.version = "2027.1".into();
    history
        .reassess(VersionedUrgencyAssessment::from_profile(&revised, 3, 8).unwrap())
        .unwrap();
    assert_eq!(history.assessments().len(), 2);
    assert_eq!(history.assessments()[0], first);
    assert_eq!(history.assessments()[1].scheme_version, "2027.1");
    assert!(
        history
            .reassess(VersionedUrgencyAssessment::from_profile(&profile, 4, 7).unwrap())
            .is_err()
    );
}

#[test]
fn policy_clock_preserves_elapsed_across_pause_and_reassessment() {
    let mut clock = VersionedPolicyClock::new("wait", "v1", 10, true);
    assert_eq!(clock.state(), PolicyClockState::NotStarted);
    clock.start(5).unwrap();
    clock.pause(9).unwrap();
    assert_eq!(clock.elapsed_ticks(), 4);
    clock.advance_to(15).unwrap();
    assert_eq!(clock.elapsed_ticks(), 4);
    clock.resume(16).unwrap();
    clock.reassess(20, "v2", 6).unwrap();
    assert_eq!(clock.elapsed_ticks(), 8);
    assert_eq!(clock.policy_version(), "v2");
    assert_eq!(clock.events()[0].target_duration_ticks, 10);
    assert_eq!(
        clock
            .events()
            .iter()
            .find(|e| e.kind == careops_core::pathway::PolicyClockEventKind::Reassessed)
            .unwrap()
            .target_duration_ticks,
        6
    );
    assert!(clock.is_breached());
    clock.stop(21).unwrap();
    assert_eq!(clock.state(), PolicyClockState::Stopped);
    assert!(clock.start(22).is_err());
    assert!(clock.reassess(22, "v3", 20).is_err());
}

#[test]
fn non_pausable_policy_clock_rejects_pause_without_changing_elapsed_state() {
    let mut clock = VersionedPolicyClock::new("departure", "v1", 240, false);
    clock.start(1).unwrap();
    assert!(clock.pause(10).is_err());
    assert_eq!(clock.state(), PolicyClockState::Running);
    assert_eq!(clock.elapsed_ticks(), 0);
    clock.advance_to(10).unwrap();
    assert_eq!(clock.elapsed_ticks(), 9);
}

#[test]
fn zero_target_and_default_urgency_profile_remain_compatible() {
    let assessment =
        VersionedUrgencyAssessment::from_profile(&JurisdictionProfile::default(), 1, 0).unwrap();
    assert_eq!(assessment.scheme_id, "ATS");
    let mut clock = VersionedPolicyClock::new("immediate", "v1", 0, false);
    clock.start(0).unwrap();
    assert!(!clock.is_breached());
    clock.advance_to(1).unwrap();
    assert!(clock.is_breached());
}

struct IndependentDimensionsPolicy;

impl DecisionPolicy for IndependentDimensionsPolicy {
    fn decide(&self, observed: &ObservedPolicyState) -> PolicyDecision {
        PolicyDecision {
            urgency: UrgencyCategory(observed.urgency.category),
            service_target: ServiceTargetStatus::Breached,
            dispatch_priority: Some(DispatchPriority("local-rule-7".into())),
            interruption: InterruptionEligibility::NotEligible,
        }
    }
}

#[test]
fn observed_policy_can_set_priority_and_interruption_independently_of_urgency() {
    let urgency =
        VersionedUrgencyAssessment::from_profile(&JurisdictionProfile::default(), 1, 5).unwrap();
    let observed = ObservedPolicyState {
        observed_at_tick: 8,
        urgency,
        elapsed_wait_ticks: 3,
    };
    let decision = IndependentDimensionsPolicy.decide(&observed);
    assert_eq!(decision.urgency, UrgencyCategory(1));
    assert_eq!(decision.service_target, ServiceTargetStatus::Breached);
    assert_eq!(
        decision.dispatch_priority,
        Some(DispatchPriority("local-rule-7".into()))
    );
    assert_eq!(decision.interruption, InterruptionEligibility::NotEligible);
}
