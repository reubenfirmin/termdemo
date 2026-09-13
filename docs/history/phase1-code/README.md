# Archived phase-1 controllers and audits

These files preserve the previous implementation for investigation. They are
not Rust modules of the active executable. The corresponding original motion,
source-hash and rendering assertions are historical evidence, not acceptance
criteria for the resized phase-3 world. Their expected results were not updated.

Active flight: `src/flight.rs`, `flight_route.rs`, `flight_speedlaw.rs` and the
protected `approved_opening.rs`. Active checks: `make audit-phase3-live`.
The original phase-1 extraction comparison is expected to fail after this
intentional motion rewrite; it must not be advertised as passing.
