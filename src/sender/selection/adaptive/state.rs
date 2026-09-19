// Keep the sole-carrier state-machine interface byte-unchanged while the loop
// owns the mode-agnostic scheduler state, including probe pacing and targets.
pub type AdaptiveState = super::super::SchedulerShared;
