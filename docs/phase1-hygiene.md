# Phase 1 — behavior-preserving extraction and measurement

Completed 2026-09-12. This is completion of the hygiene phase, NOT approval or
repair of the current flight. No trajectory equation, world dimension, render
algorithm, terrain detail, or production control behavior was retuned. Phase 2
is next; the proposed Earth-sized world and new speed function remain uninstalled.

## Code ownership and dependencies

The previous 11,495-line production/audit source is now a 2,909-line entrypoint,
renderer/terrain and terminal implementation, with the following responsibilities
extracted into real modules. Imports into those modules are explicit.

| Module | Owns | Important retained dependencies |
|---|---|---|
| [units.rs](../src/units.rs) | Existing physical dimensions and conversions | No camera or clock; the oversized production scale is unchanged. |
| [world.rs](../src/world.rs) | Fixed universe/field definitions, stable object identities, positions and spatial visibility rules | Units and shared math; no new camera-relative placement. |
| [opening.rs](../src/opening.rs) | Existing opening/approach equations, original trajectory cache and initialization | Shared math, terrain landing frame, and generation of the later motion cache. `legacy_trajectory_state` is still PRODUCTION code. |
| [route.rs](../src/route.rs) | Capture/spiral geometry, arc lookups and orbital frame geometry | Fixed world, math and existing speed/arc cache. The regional-frame dependence on the old phase law remains explicit. |
| [speed.rs](../src/speed.rs) | Existing braking envelope, curvature cap, arc/speed-limit cache and distance integration | Route geometry. Also contains the rejected later angular/altitude clocks pending phase 3; it is NOT yet one scalar speed law. |
| [motion.rs](../src/motion.rs) | Cached world-curve generation and the actual position/velocity/acceleration evaluator | Opening, route, speed and existing terrain. Owns `WORLD_CURVE` and the arrival-time state. |
| [camera.rs](../src/camera.rs) | Camera framing, near plane and shared projection/clipping | Actual motion, fixed geometry and terrain clearance. No replacement camera. |
| [math.rs](../src/math.rs) | Vector types and unchanged numerical/interpolation helpers | Existing sine table and square-root implementation. |
| [audit_legacy.rs](../src/audit_legacy.rs) | Historical compatibility equations and particle/projection witnesses | Audit-only; excluded from normal builds. |
| `audit_motion/world/surface/atmosphere/hardening/controls.rs` | Existing gate families | Read the production modules; test code is no longer interleaved with them. |
| [audit_trace.rs](../src/audit_trace.rs) | Read-only actual-camera trace and independent diagnostic dispatch | Audit-only. No new motion equation, cache mutation or renderer. |

The current runtime still follows this dependency chain:

```text
initialization: opening::generate_trajectory
                   -> motion::generate_world_curve
                   -> existing route + speed laws and caches

playback time -> camera::flow_camera
                   -> motion::trajectory_state
                      -> opening equations / WORLD_CURVE sampling
                   -> one production renderer + fixed world + shared terrain
```

The route/speed cache coupling, mutable initialization caches, late clocks and
legacy opening dependencies have NOT been hidden or replaced. Removing those
competing motion authorities belongs to phase 3, after phase 2's coupled fit.
This extraction supplies clear ownership and measurements without pretending
that moving functions into files solves their mathematical defects.

## Exact preservation checks

- Before/after traces match **268,801 camera states, all 25 fields**, across
  **0–140 seconds at 1,920 Hz**, bit-for-bit. Fields include actual camera
  position, trajectory velocity/acceleration, camera axes, AGL, reported speed,
  near plane, orbit progress/rate and roll.
- **30 normal-build RGB checkpoint digests match exactly**, spanning 0.001–140s:
  opening, wormhole exit, circular/square grid, capture, orbital descent and
  low flight. RGB was hashed in RAM; no images were saved, displayed or used
  as assets. This is sampled output equality, not a claim to have compared
  every possible frame. The existing wormhole gate also passes its 97 exact
  opening/shutter compatibility comparisons.
- **490 original declarations retain their tokens**, ignoring only whitespace,
  comments and module visibility. One additional entrypoint declaration has a
  documented audit-only dispatch change. Both hashes/reason are recorded in
  [the extraction witness](../tests/phase1_extraction.json).
- Ten tests verify fail-through reporting, unequal-time position differentiation,
  changed constants/equations/types/feature guards, source parsing and relocated
  source spans. The module-aware reader reproduces the actual pre-extraction
  raw hashes of all ten protected source spans. The older approved SHA256
  values have NOT been updated; their existing failure remains a failing gate.
- Normal/release and audit builds complete without warnings; `git diff --check`
  passes. No stored image or old flight fingerprint was regenerated or blessed.

The full trace SHA256, before and after, is:

```
0c6e5038fb4159ad765d06a2ab55dfd5725a205d6edb8529ef201303a23f4a92
```

The normal executable hashes differ after extraction, as expected from code
organization/build output. Equality claims above concern the measured states
and RGB bytes, not equality of executable files.

## Current measurements and failures

[Complete whole-second speed/altitude report](phase1-camera-motion.md), with
all 268,801 samples available in the generated
[CSV](../target/flight-audit/camera-motion.csv). Speed is independently
differentiated from camera positions at their actual f32 playback timestamps,
then compared with the reported velocity; no candidate speed envelope is used.

The trace exposes a **94.5271% speed loss in one second ending at 36.5927s**,
and **16.7297% reacceleration in one second ending at 46.2719s**. Both remain
unchanged. Worst post-exit position/velocity finite-difference residual is
0.02441%; the coarser 120-Hz stencil gives 2.93855%. These are measured numerical
residuals, not a waiver of the intended single-function/continuity requirements.

The final independent suite runs **17 gates: 10 pass, 7 fail**. It exits nonzero.
The final performance run was not concurrent with the render comparison.

| Failed gate | Result and status |
|---|---|
| Historical source locks | Orbital radius/altitude raw-source fingerprint already differed before extraction; expected and observed hashes are unchanged. |
| Flight requirements | Existing arrival does not maintain the required fast-jet slowdown at airline height; exit 249. |
| World/camera catalogue | Existing `last_ring > GRID_RINGS` ordinal assertion fails; now named explicitly instead of reporting only object counts. No geometry or assertion was changed. |
| Historical flight fingerprint | Existing landscape flight fingerprint mismatch remains a separate failure; exit 225. |
| Terrain refinement | Newly reachable beyond the fingerprint gate: 66.1% coverage at 40/41/42s versus the existing 70–85% target; exit 229. |
| Atmosphere/performance | Newly reachable beyond the fingerprint gate: combined frame time 52.606 ms at 53s versus 33.333 ms; exit 227. |
| Hardening/performance | Existing 53s checkpoint takes 51.007 ms versus 33.333 ms; exit 232. Timing varies by run; no quality reduction or threshold change was made. |

Passing gates: extraction tokens, playback state, legacy motion, precision,
motion derivatives, surface geometry, orbital descent, grid density, wormhole
and terminal playback/cleanup. Old gate names/messages refer to historical
assertions, not approval of the latest flight specification. Each gate still
stops on its first unsafe/failed assertion; the runner guarantees that the
OTHER independent gates execute and reports them all.

The original full-run attempt had six failing gates; refinement and atmosphere
were each stopped by the same old flight fingerprint before exercising their
own checks. That fingerprint is now reported once as an independent failing
gate, while terrain and atmosphere checks execute unchanged. The original
direct `&` and `*` selectors retain their fingerprint-first behavior.

Additional planned work is still absent: Earth-sized rescaling, the one global
braking function, the shortened partial-third-orbit descent, the richer fractal
planet and manual spacecraft control. None is implied by phase 1 completion.

## Reproduction and evidence locations

```sh
make check
make audit-harness
make audit-extraction
make audit-flight  # currently fails, and still runs every independent gate
make audit-trace   # actual camera-state.bin, camera-motion.csv/.md, motion-summary.json
```

The machine-readable suite results are in
[gates.json](../target/flight-audit/gates.json). Baseline binaries, original
source and before/after traces from this pass are retained under
`/tmp/termdemo-phase1.Lijj5X/`; these are temporary evidence, not runtime inputs.
The source witnesses and report remain in the repository if temporary files
are later cleared.

```sh
python3 scripts/flight_audit.py compare /tmp/termdemo-phase1.Lijj5X/before/camera-state.bin target/flight-audit/camera-state.bin
python3 scripts/flight_audit.py compare-render /tmp/termdemo-phase1.Lijj5X/demo-before target/x86_64-unknown-linux-gnu/release/termdemo
```

The superseded plan/README narratives are preserved in [history](history/flight-plan.md).
Only [PLAN.md](../PLAN.md) governs the next phase. Stop here after phase 1;
phase 2 is the next task, not an implicit continuation into flight tuning.
