//! Clinical pathways, bounded progression, policy clocks, and decoupled draw ownership (CC-04, CC-06, CC-07).

pub mod adapter;
pub mod classification;
pub mod clock;
pub mod error;
pub mod progression;
pub mod stage;
pub mod streams;

pub use adapter::{
    ClinicalProgressionLog, ConfigurablePathwayAdapter, PathwayPatientSpec, PatientPathwayOutcome,
    StageExecutionRecord,
};
pub use classification::{
    ClassificationHistory, DecisionPolicy, DispatchPriority, InterruptionEligibility,
    ObservedPolicyState, PolicyDecision, ServiceTargetStatus, UrgencyCategory,
    VersionedUrgencyAssessment,
};
pub use clock::{
    PolicyClock, PolicyClockEvent, PolicyClockEventKind, PolicyClockState, ScheduledEvent,
    VersionedPolicyClock,
};
pub use error::{PathwayError, PathwayErrorKind};
pub use progression::{CompetingRiskModel, HazardRate};
pub use stage::{StageGraph, StageId, StageNode};
pub use streams::{
    PathwayRngStreams, SubsystemStream, STREAM_DOMAIN_PROGRESSION, STREAM_DOMAIN_ROUTING,
    STREAM_DOMAIN_SERVICE,
};
