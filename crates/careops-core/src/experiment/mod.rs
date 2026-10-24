//! Experiment lifecycle, deterministic seed allocation, cancellation, and execution abstractions.

pub mod cancellation;
pub mod error;
pub mod io;
pub mod runner;
pub mod seed;

pub use cancellation::CancellationToken;
pub use error::{ExperimentError, ExperimentErrorKind};
pub use io::atomic_write_file;
pub use runner::{
    current_epoch_ms, run_generic_batch, resume_generic_batch, BatchProgress, GenericBatchCheckpoint,
    GenericBatchRunDocument, IndexedReplication, PartialBatchMarker,
};
pub use seed::allocate_replication_seeds;
