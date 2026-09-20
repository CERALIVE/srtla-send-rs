//! ADR-003 bind-map sidecar.
//!
//! Only the **telemetry-facing projection** is present so far: the operating
//! -mode tokens and the record types the ADR-001 snapshot publishes at its top
//! level. The sidecar reader, coherence retry, and link resolution land with the
//! rest of ADR-003; [`report`] is written against the tokens rather than against
//! a resolver so the document model has a stable shape to serialize in the
//! meantime.

pub mod report;
pub mod status;

pub use report::{BindMapReport, BindMapStatusRecord, CollisionRecord, DispositionRecord};
pub use status::{BindMapDisposition, BindMapStatus, CollisionGroup, DegradedReason};
