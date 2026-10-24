//! Clinical pathways, bounded progression, policy clocks, and decoupled draw ownership (CC-04, CC-06, CC-07).

pub mod adapter;
pub mod clock;
pub mod error;
pub mod progression;
pub mod stage;
pub mod streams;

pub use adapter::{
    ClinicalProgressionLog, ConfigurablePathwayAdapter, PatientPathwayOutcome, PathwayPatientSpec,
    StageExecutionRecord,
};
pub use clock::{PolicyClock, ScheduledEvent};
pub use error::{PathwayError, PathwayErrorKind};
pub use progression::{CompetingRiskModel, HazardRate};
pub use stage::{StageGraph, StageId, StageNode};
pub use streams::{
    PathwayRngStreams, SubsystemStream, STREAM_DOMAIN_PROGRESSION, STREAM_DOMAIN_ROUTING,
    STREAM_DOMAIN_SERVICE,
};
