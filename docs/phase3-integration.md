# Phase 3 — implementation and automated verification complete

The user approved retaining the original nearby stars as **unblurred points**
after emergence. There is no large exit detour and no new speed specification.
The revised motion and world are connected in the source and the normal
executable. The user rejected the separate preview workflow: use `make build`
and `make run`, with no alternate preview target. Known test failures remain
reported; the normal executable is not held on an older version because of them.

## Latest pass — 2026-09-13

The normal build contains the following corrections. `make audit-phase3-live`
passes **all 25 gates**, including the unchanged 33.333-ms elapsed-frame target:
the worst audited frame was **25.680 ms**. A complete 2,311-frame moving sequence
through 77 s peaked at **27.779 ms**, with no over-budget frames. These are measured
results on this host, not an unconditional real-time guarantee. Detailed-planet
and final in-motion visual acceptance remain phase 4/5 work. The late-opening
grid permission is recorded narrowly, as described below.
No motion coefficients, route, keyframe timing, stellar positions, grid pitch,
planet size, terrain heights, or surface mesh density were changed in this pass.

- **Spoke clipping:** clip against the viewport in homogeneous double precision,
  before perspective division. Near-plane-only clipping could send the hidden
  end of a spoke millions of pixels away, losing precision in the rasterizer.
  The audit now checks 12,003 camera samples across off-axis near-plane passages
  at three aspect ratios, checking viewport bounds, line position, reciprocal depth,
  reversed endpoints and continuity. The world endpoints remain fixed.
- **Terrain lighting:** removed the extra per-triangle lighting path. It used
  the old native origin and a different inverse lens; it also multiplied colors
  AFTER haze/clouds had been added, lighting the atmosphere in triangular patches.
  Surface vertices and boundary pixels now use normals differentiated from the
  existing displaced terrain before atmosphere is applied. This changes lighting,
  not the terrain or its sampling density. Normal, ocean and uniform-haze raster
  tests cover the replacement. Paused normal-build frames were inspected.
- **Exact caches:** store fixed ring/spoke vertices and procedural-noise corner
  values; shortcut only exact zero/one LOD plateaus. Cached geometry is compared
  to the original evaluator, including other layouts and out-of-cache objects.
  Noise collisions/negative keys and LOD threshold values are also checked.
  The noise-cache/LOD change alone was byte-identical in 80 normal frames across
  the flight. That comparison does **not** claim the separate lighting correction
  leaves planetary colors unchanged.
- **Actual rendering evidence:** normal and instrumented RGB agree exactly at
  eight late-flight checkpoints. The Sun is visibly drawn during orbit 1 at 29 s
  and orbit 2 at 35 s, not merely returned by a projection calculation.
  There are 258 forward crossings of fixed grid planes from 7.6 to 26 s; cadence
  decreases from 27.249 to 6.007 crossings/s without reversals. The initial cadence
  exceeds 15 Hz, so 30-fps temporal aliasing remains a visual-review risk; a plane
  crossing count alone does not approve the apparent motion.
- **Pixel-exact parallel rendering:** up to three persistent Linux child workers
  plus the parent claim eight-row tiles from one atomic work queue. This replaces
  fixed stripe assignments so a worker that finishes can take the next tile.
  They run the same renderer with the same time/aspect and original four-pixel
  surface lattice. The production field renderer records one bounded, shared
  command snapshot per frame; ordered row bins avoid scanning every command for
  every tile. The immutable background/star input and disjoint completed RGB/depth
  output occupy separate shared storage. The parent collects output only after
  every worker finishes. No snapshot survives as world geometry or predicts the
  next camera. Terrain caches and raster scratch remain private to each process.
  All 45 serial/parallel RGB **and depth** comparisons pass at 15 times and three
  aspect ratios, including the protected opening, grid, orbits and low flight.
  Exact row/tile ownership, reaping and serial recovery after actually terminating
  an audit-owned worker are tested. Resource/channel failures fall back to the
  same serial renderer; the normal build uses this implementation too.
  Additional runs restricted to one and two CPUs verify serial operation and
  two-process recovery. All 80 normal-build RGB samples across 0–77 s match the
  saved pre-worker normal executable exactly. A further 80-frame comparison of
  the final queued renderer against this pass's saved fixed-worker executable
  also matches exactly. Scheduling changes do not alter the previously corrected
  terrain lighting or any visible frame content.

`make profile-phase3-render` uses the normal build and the existing audit
instrumentation, recording wall time and per-stage CPU time in
`target/phase3-audit/profile.json`. Wall time includes preparation, dispatch,
worker waiting and RGB/depth assembly. CPU sums rendering work in ALL processes,
including command preparation; it is not the parent CPU masquerading as total
frame cost. World-table startup and terminal encoding/transmission are outside
these render measurements. Repeated frames do not establish sustained terminal FPS.

`node scripts/profile-phase3-render.cjs --sequence` profiles successive 30-Hz
samples across 0–77 s instead of repeating selected frozen views. It retains
every sample and exceedance, binary hashes and observed host metadata in
`target/phase3-audit/profile-sequence.json`. The final sequence's median was
12.120 ms, p95 21.122 ms and maximum 27.779 ms. `--serial` selects the same
kernel without workers in the existing audit binary, not another app build.

`make profile-phase3-playback` runs the normal executable through a PTY for the
full journey, without seeking. It checks the existing timestamp counter reaches
77 s monotonically and normal exit restores terminal state. Its report counts
completed protocol transmissions, **not actual Kitty display/compositor FPS**;
it does not constitute visual approval.
The final normal executable completed 4,248 transmissions over 76.992 s between
first and last frames: 55.162 transmissions/s on average, p95 gap 25.393 ms and
maximum gap 32.926 ms. The final counter was 77 s and terminal cleanup passed.
The live audit, moving-frame sequence and PTY playback all used normal executable
SHA-256 `b4b2526ccbc7ff26db2eefbe3aab12c765cf33887af7c283ff65ee3ca320bbab`.

## What is implemented

- `flight_speedlaw.rs`: the reviewed single Bernstein-integral exponential
  speed function. Coefficients are constants; there is no runtime fit, curvature
  cap, orbital speed override or independent descent clock.
- `flight_route.rs`: one analytic spatial curve, with differentiated radial
  capture/descent and the compact off-centre weave. Arc inversion interpolates
  the scalar metric, not camera poses. Production positions are `P(s(t))`.
- `flight.rs`: one fixed registration of the metre world into native coordinates.
  It does not recalculate world placement from the selected camera or route.
  Planet orientation is also fixed independently of the selected route.
- `world.rs`: Earth radius 6,371 km; independent 2,000-km axial grid pitch;
  192 circumferential lanes; 6,000-km circular radius; square width 18,000 km
  and height 15,603.605 km; floor-attached 6,371-km gravity depression.
  The catalogue is fixed, with 4,096 additional equal-area distant stars.
- `approved_opening.rs`: only the frozen original 0–4-second arithmetic and
  coordinates. The old later controllers are archived under
  `docs/history/phase1-code/`, not compiled beneath the new speed function.
- All illumination now reads the same world-space Sun. The old extra legacy
  Sun-coordinate transform was removed. Rays invert the actual projection
  basis, including the original fixed horizontal calibration.

The original terrain and renderer remain; no imagery, new surface renderer,
coarser mesh, or reduced detail was introduced. Diagnostic screenshots were
viewed in `/tmp`; they are not assets or replacement image baselines.

## Measured actual flight

| Event | Actual time | Speed | Terrain clearance |
|---|---:|---:|---:|
| Wormhole emergence; rings visible | 5 s | 66,965,185 m/s | 681,591.8 km |
| Circular entry; incomplete spokes | 7.6 s | 54,578,142 m/s | 524,259.2 km |
| Square evolution begins | 23 s | 15,550,665 m/s | 51,530.7 km |
| Fully squared grid | 26 s | 11,907,394 m/s | 11,696.9 km |
| Orbit 1 | 28 s | 9,949,757 m/s | 320.224 km |
| Orbit 2 | 33.097328 s | approximately 6,609,000 m/s | approximately 115.5 km |
| Partial orbit 3 begins | 41.373521 s | approximately 3,148,000 m/s | approximately 40.0 km |
| Airline height | 57 s | 7,144.77 m/s | 9,144 m |
| Canyon arrival | 77 s | 600 m/s | 30 m |

These speeds are fitted outputs, not newly imposed fixed specifications.
The final state is 2.31393 revolutions beyond orbit-1 entry: no third-lap completion.

## Evidence and remaining visual-review risks

Run `make audit-phase3-live`. It builds the normal executable and the existing
instrumented `target/phase0-audit/` executable, then writes the diagnostic report
to `target/phase3-audit/result.json`. That report directory is not an alternate
application build. Reference hashes are never regenerated by this command.
The actual camera is sampled at 1,920 Hz through 77 s, including every radial
and lateral component. The independent JavaScript model uses a more refined
arc table than the production code.

Passing checks include:

- 7,681 protected-opening camera samples: position, velocity, acceleration,
  forward, right and down are **byte-identical** to the original executable.
- All 16,254 original star objects and the original 4-s camera projection
  header are byte-identical.
- Total-speed function relative error below 1.8e-14; independent differentiation
  of actual positions differs from velocity by at most 6.72e-7 relative.
  Independent differentiation of velocity differs from acceleration by at
  most 7.82e-7 relative. The acceleration check uses a five-point stencil on
  the actual nonuniform f32 timestamps; a three-point stencil has appreciable
  truncation error at the quadratic onset immediately after 4 s.
- Independently computed route positions differ by less than 0.00051 m.
- Minimum roof clearance 8,015.522 km; minimum side clearance 8,997.931 km;
  minimum circular clearance 4,690.941 km. Below-floor travel stays within
  a 2,308.776-km horizontal footprint around the planet.
- No capture-altitude rebound; actual minimum sampled AGL 29.99999995 m.
  This is dense sampling, **not formal interval collision proof**.
- Rendered rings at 5 s; assembling spokes through circular entry; zero star
  streaks from 5 s onward; local stars hidden by the fully squared grid.
- Projection/ray agreement at three aspect ratios and working Space pause,
  resume without catch-up, seeking while paused and quit.

**Resolved — narrowly approved late-opening grid revision.**
The old grid already contributes pixels by the 3.985-s checkpoint. Thus the
new grid changes the final approximately 0.02 s of the protected opening,
despite preserving every original star and camera sample exactly. At 4 s,
645 RGB bytes differ inside x=152–167, y=92–107. This was isolated in a
temporary copy of the old renderer with **only its grid drawing disabled**:
its full RGB then matches the new opening exactly at 3.99 and 4 s.
The user's subsequent instruction to finish phase 3 was taken as approval for
ONLY that change. Original hashes remain in `tests/phase3-opening-reference.json`.
The separate `tests/phase3-opening-grid-revision.json` records five revised exact
full-frame hashes and original hashes outside the inclusive x=152–167, y=92–107
rectangle. Both must match: this is not a blanket mask over unchecked pixels.
Frames through 3.98 s still require their original full-frame hash. Stars and
all native camera samples remain exactly protected. No renderer switch was added.

**Resolved in the final measurements — elapsed frame time.** Earlier runs peaked
at 53.272 ms and 46.247 ms, and remain historical failures rather than waived
samples. Before any new production changes, the same saved binaries became
substantially faster. An old/new/old comparison also showed ordinary run-to-run
variation: the old renderer at 65 s measured 22.881 and 22.393 ms around a queued
renderer run of 19.531 ms. Do not attribute the entire earlier 53-to-26-ms
difference to code changes. The exact external cause was not established; no
governor, hardware or system settings were changed.

The unchanged final gate includes all 93 timed samples at three aspect ratios;
its maximum was 25.680 ms at 36 s / aspect 1. The full moving-frame sequence also
passed, without normalizing CPU speed, removing outliers or lowering quality.
The retained maximum is four processes. Earlier eight-worker and scanline-bound
experiments were not retained. Recheck actual elapsed time after phase-4 changes;
passing on this host is not a universal 30-fps guarantee.

Late-flight cost remains dominated by the unchanged terrain-ray searches and
surface shading. Shared tile-boundary vertices can be evaluated by more than
one worker; this duplicates computation, not geometry or pixels. Any further
acceleration must retain the surface lattice, exact boundary rays, lighting and
all spokes. Do not drop detail, skip unproven silhouette cells, change the flight,
normalize measured times to a hypothetical CPU, or declare the performance gate
passed from a lower parent-process CPU time.

The post-haze triangular lighting patches are removed. The current terrain still
lacks the detailed nonrepeating geography needed for the intended descent;
that and final cross-scale visual acceptance remain phase 4 work.
Peak forward-camera angular rate is about 83.88 degrees/s;
numerical continuity is **not** passenger-comfort or visual approval. Uninterrupted
terminal playback has not been visually approved in this pass.

## Endpoint and resume instructions

Playback **pauses at 77 s**. It does not pretend the ship physically
brakes to zero: the boundary velocity/acceleration remain available. Manual
flight has not been implemented; phase 5 must inherit that complete state,
including angular motion. There is no placeholder scheduled orbit after arrival.

1. Keep the resolved opening question closed: preserve both the original baseline
   and the explicitly bounded revision, together with exact stars/camera checks.
2. Proceed to phase 4's detailed procedural geography on this established flight.
   Do not retune the route, speed, scale or approved opening as part of terrain work.
3. Retain the strict frame-time and exact execution-parity gates as detail grows.
   Do not repeat rejected scanline-bound or eight-worker experiments as new fixes.
4. Complete temporal-aliasing, descent readability and uninterrupted visual
   review alongside the detailed planet. Fixed-plane cadence, Sun witnesses and
   PTY transmission timing do not establish comfort or exclude every in-motion
   artifact. Phase-3 implementation/checks are complete; final visual acceptance
   is not being inferred from those checks or from stills.

The saved **pre-integration** executable had SHA-256
`e5a23b0119fa4898189fd8422ecf29db00b2cbfcd3e4bbb8579efe8fb39bdc30`.
This is historical evidence, not the current executable's hash. Use the normal
`make build` / `make run` workflow for current work, as explicitly directed by
the user. The separate preview build target has been removed.

The original phase-1 whole-source locks are intentionally obsolete after this
rewrite and were not regenerated. Legacy audit selectors now fail explicitly,
instead of falling through to a normal frame and returning a false pass.
The fourteen audit-harness unit tests test the reader/reporting algorithms using
fixtures; they do not claim unchanged phase-1 source. The 12 offline numerical
tests and warning-free normal/audit compile checks pass. `rustfmt` is not installed.
