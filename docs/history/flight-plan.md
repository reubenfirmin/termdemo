# Historical flight-plan checkpoints — archived 2026-09-12

This is a preserved record, NOT the active plan or a set of current approvals.
Use [the current plan](../../PLAN.md) and [phase 1 status](../phase1-hygiene.md).
Old phase numbers, locks, deadlines and acceptance statements below are historical.

## Historical checkpoints — superseded by Current directive above

The following records explain the dirty worktree and prior tests. Their
"binding", "accepted", "freeze", "next" and timing statements are historical
where they conflict with the Current directive. In particular, do not resume
the old Phase 3T power-32 proposal or treat the 27,000-mile planet, 2.4-unit
orbit radius, 18-unit exit speed or 137-second scripted endpoint as newly
approved constraints. Their status must be resolved in the scale study.

### Previous working state — 2026-09-12

**Wormhole/circular-grid correction:** the opening starfield, acceleration
and streaking rush already carry the camera through the wormhole around
4–5 seconds. There is NO second post-wormhole streak-building or acceleration
phase. The distant ring structure is already visible on emergence; its fixed
spokes continue assembling as we approach and traverse the cylinder. Grid-line
passage, with independently sized grid spacing, supplies the sense of speed.
This supersedes all older prose requiring streak populations to accumulate
through 7–11 seconds or into the circular grid.

Implemented in this correction: stars retain their fixed identities and
positions, but render as points after emergence. Their 32-ms motion-blur
exposure is exact through the approved first four seconds; a C2 shutter
function reduces it to zero over the next 16 world units (finishing before
5 seconds on the current flight). This is an explicit rendering/exposure
choice, NOT a physical braking law. It does not move, hide, or organize stars,
change their later square-grid visibility, or alter the fixed ring/spoke
catalogues. The camera path and speed are untouched. No imagery is used.

`make audit-wormhole` checks 97 exact in-memory opening-frame comparisons,
zero actual star-streak draw calls across 5–21 seconds at 30 Hz, retained
point stars/ring edges/spokes, fixed progressive spoke connectivity, aspect
ratios, and nonincreasing actual approach speed through 28 seconds. A negative
witness renders the old full exposure at 11 seconds and must detect streaks.
The new check is independent of the still-failing orbital gates below.

**Existing Phase 3T work remains partial.** This focused correction does not
install the proposed single braking function or resolve the descent timing
decision described below.

**Phase 3T — explicit speed function and independent grid scale. PARTIAL;
the active braking curve is NOT fixed.** The user has authorized reopening
post-wormhole motion, ring density and structure length together. The intended
implementation is `position(t) = path(integral(v(t)))`, with path parameterized
by arc distance. Curvature is an acceptance constraint, not a second speed
controller. Vertical descent must be included in that same total speed. The
approved first four seconds of camera/star motion remain unchanged. No imagery
or replacement scene pipeline is authorized.

Completed in source in this pass:

- Ring spacing is no longer the old 17:16 progression throughout the mature
  tunnel. It has its own 1.5-world-unit regular pitch. A positive exponential
  excess, with its constant/linear/quadratic/cubic terms removed, gives a C3
  progression into that exactly regular grid. Stellar catalogue spacing,
  star identities, world frame, planet, tunnel width and region boundaries
  are unchanged. Near the first physical rings the gap is now 3.287 world
  units rather than about 18: approximately 5.474 crossings/s at speed 18.
  There are 197 physical rings. Binary range lookup agrees with the fixed
  coordinate function. `make audit-grid-density` passes; measured render
  times at 4/8/11/19/21/26 s were 6.1–12.2 ms. No geometry was removed to
  obtain those timings. First-four-second star/camera functions are untouched;
  newly denser distant grid pixels at the emergence boundary are not claimed
  to match the previous grid image.
- `src/flight_speed.rs` contains an AUDIT-ONLY proposed governing function:
  `v(t) = 133.31040139731104 + (3400613843.4782605-133.31040139731104)
  / (1+((t-4)/21.970931282356474)^32)` in m/s, for t >= 4.
  It supplies analytic speed, acceleration and jerk, without position,
  curvature, orbital state, runtime fitting or speed caps. The four unit
  tests and the 1,920-Hz full-function audit pass. It reaches 600 m/s at 40 s;
  peak fractional braking is 1.2674/s at 28.459 s, and both absolute and
  fractional braking relax monotonically after 30 s. It is NOT connected
  to the camera and has not been accepted as the final function. The active
  rejected rational-envelope/curvature-limit implementation is unchanged.

**Decision pending before complete path integration:** the old ground-arrival
schedule is inconsistent with this proposed gentle tail. From 40 to 58 s the
function covers only 2,941 m of total path, but the altitude alone must fall
9,124 m. Even a purely vertical trajectory would not reach the ground until
about 104.38 s with this function; a useful forward flight takes longer.
Do NOT report that shortening grid spacing fixes this independent descent
constraint. Do NOT accelerate during descent to hide it. The user has been
asked whether ground arrival may move later, retaining 600 m/s at 9,144 m
by 40 s. No answer received yet. A substantially longer descent or retaining
higher speed through descent requires an explicit choice; do not silently
relax the old 15–20-second descent or 133-m/s cruise targets.

Remaining Phase 3T work, in order:

1. Resolve the descent timing/speed tradeoff and finalize/publish the single
   function. The current scalar proposal is not a complete feasible flight.
2. Build one arc-length representation of the capture, spiral and regional
   geometry. Preserve opening camera/star samples exactly. Remove active
   curvature speed overrides and time-driven regional angular/altitude laws;
   derive actual world velocity from the same scalar function throughout.
3. Derive fixed post-wormhole region extents and planet approach placement
   from the distance integral; do not fit another braking exponent to the old
   oversized path. Region times may only be revised explicitly. Grid pitch,
   tunnel width and total axial extent stay separate parameters.
4. Validate actual position derivatives against v(t), the full braking tail,
   containment, curvature, visible descent, Sun recurrence, line-passage
   cadence and performance. Only then revise authorized source/fingerprint
   locks and build the full replacement for live review.

New targets: `make audit-speed-function` (standalone scalar tests plus `b`
audit selector) and `make audit-grid-density` (`d`). The scalar test is NOT
included in `audit-flight` as a substitute for actual-camera checks. The grid
gate is included. Old active-braking, Phase 5 framing, and Phase 6/7 performance
failures remain open; an all-green or completed-flight claim is incorrect.

The normal demo has now been rebuilt with the density change only. Space
pause/resume and paused seeking pass both the numerical and real-terminal
checks. The old source-level unused-function warning exposed by the new binary
lookup has been fixed. Phase 0 motion and Phase 1 precision pass. No new speed
function has been installed, and no old motion fingerprint has been re-blessed.
The earlier checkpoint's statement that the normal binary was not rebuilt
describes that earlier pass, not the current grid-only build.

**Unfinished candidate — added descending revolutions and recurring Sun.**
The latest request supersedes the orbital radius/timing freeze below, but NOT
the approved opening, downward capture geometry, fixed world, or smooth-motion
requirements. The Sun must remain a fixed world direction. No imagery, visual
baselines, timed Sun reveal, camera cut, or replacement surface is authorized.

Diagnosis: the previous flight crossed the curved-limb range from about 57 to
3 pixels of sag in roughly 31.4–31.7 seconds. Its third nominal revolution had
nearly exhausted its angular travel by 34 seconds. The old tests counted phase
crossings and endpoint altitude, not time spent seeing changing curvature.
The fixed Sun did cross the earlier view, but for brief windows (about 29.1
and 30.6 seconds), so endpoint-only Sun tests missed the composition problem.

Source currently contains an **UNACCEPTED five-revolution candidate**:
one logarithmic-clearance spiral, the same capture curve, and a closer-to-limb
view during intermediate-altitude orbits. The 12-degree aircraft view, world
Sun direction, terrain, stars, tunnel and playback controls are untouched in
this pass. Aircraft arrival remains at 40 seconds for now. The normal demo
binary has deliberately NOT been rebuilt; `make build` would build this
unfinished candidate. Only the separate audit binary has been rebuilt.

`make audit-descent` checks actual projected core-sphere curvature and bright,
depth-unoccluded Sun pixels in full rendered frames held only in memory:

- Five-revolution candidate: 1.850 seconds and 1.863 actual revolutions in the
  50-to-3-pixel curved-limb range, versus the old approximately 0.3 seconds.
- Before 34 seconds, the first four revolutions contain respectively
  14/11/6/6 Sun-visible frames at 60-Hz sampling, each with at least 100 bright
  unoccluded Sun pixels. This is a recurrence measurement, not visual approval.
- A four-revolution trial gave only 1.450 seconds in that curvature range and
  a shorter third Sun transit; it was not retained.

**Do not claim completion or relax the existing braking gates.** The current
40-second fit fails them: only 38.3% of logarithmic slowdown occurs by 35 s
(minimum 55%); 33.2% occurs during 36–38 s (maximum 25%); worst 100-ms speed
loss is 25.4% (maximum 25%); worst two-second share is 36.9% (maximum 35%).
The old 38/39-second arrival-speed windows and first-orbit speed ratio fail
too. `=` and `^` therefore remain red. The integral reaches the geometric
endpoint exactly; an explicit feasibility guard now rejects an unreachable
fit instead of silently jumping to the endpoint. `#` passes its narrower
checks, but its output no longer falsely claims atmosphere entry at orbit two.

Next decision requested: may the added orbits move aircraft-speed arrival to
about 43–44 seconds, or must it stay at 40? No answer received yet. Do not
extend the deadline without authorization. A longer arrival window would
also move the subsequent 18-second descent; audit checkpoints must follow
the actual physical event, not blindly keep old timestamps. If 40 remains
mandatory, continue solving the spiral and speed envelope together; the
present candidate is not a successful solution.

New audit selectors: `:` prints the orbital/altitude/curvature/Sun profile
and braking-distribution metrics; `v` runs the descent/Sun gate. No source
locks or frozen-flight fingerprints were re-blessed for this candidate.
Before completion: pass motion continuity, volume and braking checks; then
explicitly update only authorized orbital locks and the flight fingerprint,
register `v` in the aggregate flight audit, rebuild production, verify Space
pause again, and obtain live review. The older Phase 5 framing and Phase 6/7
53-second performance failures below remain separate open issues.

**Playback inspection control:** Space now pauses/resumes the exact displayed
timestamp instead of skipping five seconds. Paused wall time is excluded on
resume; arrow/checkpoint seeking stays paused and renders the requested view.
Unchanged paused frames are not repeatedly rendered or transmitted. The
numerical and terminal-loop checks in `make audit-controls` pass, including
Escape cleanup. This changes playback controls only; the flight, framing and
Phases 5/6 acceptance issues below are unchanged.

**Active work — Phases 5 and 6, authorized together after Phase 3S.**
The user still rejects the speed decrease and has explicitly authorized
terrain/atmosphere work before resolving it. Motion approval is deferred,
not passed. Freeze the current path, timing, speed law, camera orientation,
opening, stars and tunnel. A fingerprint of every stored flight-state value
must remain exact while the surface is improved. Preserve the existing
navigable valley floor so terrain edits cannot silently retime the flight.
Use procedural geometry/materials only; no bitmap imagery or visual baselines.
Detailed work packages and gates are listed under Phases 5 and 6 below.

**Implementation checkpoint — Phases 5A–5C and 6A–6C implemented; 5D/6D
remain open, not accepted.** The active geography now uses nonperiodic,
domain-warped continental/basin/ridge bands and permanent tributaries. The
inherited navigable floor is retained exactly. The canyon narrows downstream
and continues beyond the flown section; the surface search now intersects
walls above the reference floor instead of overlooking them. Materials use
terrain-face lighting, filtered rivers and city blocks. No image assets were
created or used.

The fixed world sun now has exposed, depth-occluded sky samples at all eight
tested high-pass/descent/canyon poses. A shared exponential-density shell is
integrated only to visible geometry; clouds occupy a persistent 2-km shell.
Short terrain rays no longer inherit far-side atmospheric haze. Stars and
grid remain extinguished at airplane altitude. These are changes to world
geometry/materials, not a new camera or a screen-space sun.

The flight-state fingerprint remains **1120209875 / 1356210471** (high/low
32-bit words), covering every stored position, velocity, acceleration, phase
and phase-rate value through 140 seconds. Camera/orientation functions and the
star/tunnel definitions are untouched. This preserves the current motion; it
does not imply that its braking is approved.

Outstanding acceptance issues:

- The frozen camera yields **66.1%** terrain coverage at 40, 41 and 42 seconds,
  below the unchanged 70–85% composition target. A camera-framing adjustment
  requires the user's choice; do not silently change the camera or weaken the
  gate. The material/landmark gates pass (22–23 coarse color bins and all 48
  tracked landmarks).
- Surface-intersection work at the **53-second descent pose is 51–59 ms**,
  over the unchanged 33.3-ms maximum. Most other tested frames are 21–31 ms,
  but this is NOT a performance pass. The stable four-pixel lattice and exact
  boundary-pixel fallback remain; missing-ray boundary cells still reevaluate
  too much terrain. Finish conservative spatial acceleration/reuse without
  removing visible geometry. Exact sample memoization is checked bit-for-bit
  against uncached evaluation; cold-cache timings remain mandatory.
- Live appearance, scale progression and braking are unapproved. Do not use
  the numerical checks as a substitute for the user's uninterrupted review.

The numerical flight target now runs Phase 6 as well and continues through
individual failures to report the whole suite. Phase 5's framing and Phase
6/7's expanded performance gates remain red; no thresholds were lowered.
The old 122-second wall and fixed-sun failures described in historical
checkpoints below have been fixed, not skipped or retimed.

Resume order: resolve the high-pass framing choice (leave it unchanged unless
authorized), finish conservative acceleration of missing-ray boundary cells
at 53 seconds without dropping surface samples, then rerun `make audit-flight`
and obtain live review. The source/build checks pass without warnings; the
aggregate audit deliberately exits nonzero for the open gates. Do not start
Phase 7 cleanup as though Phases 5/6 had been accepted.

**Current checkpoint — Phase 3S: stretch the braking curve backward, without another hard-braking window.**
Phase 3R is rejected: it added a small early reduction but retained a separate
late brake, leaving the perceptible slowdown concentrated near 36–38 seconds.
The latest instruction is to spread the decline, not move a strong braking
event earlier. The downward spatial curve remains approved and unchanged.

There is now one time-based logarithmic speed envelope from **25.658 to 40 s**
for both capture and orbit. Its integral determines distance along the same
geometric path. The envelope is fitted to that path's remaining length and
the approximately 600 m/s endpoint; it does not change the world scale,
shorten the curve or reset the position at an orbital boundary.
A smooth, curvature-derived speed limit anticipates tighter bends and
prevents a later acceleration when curvature falls again. There is no
separate first/second/third-orbit braking clock.

The spatial capture, orbital radius/altitude functions, frame rotation and
regional track retain their source locks. Opening position and basis through
20.5 seconds remain canonical. The incoming speed function is exact before
the authorized braking onset. Tunnel, gravity well, star objects, surface and
renderer are untouched.

| Observation | Demo time / value |
|---|---:|
| Braking envelope begins | 25.658 s |
| Orbit entry | approximately 28.641 s |
| Orbit one complete | 30.016 s; duration 1.375 s |
| Orbit two complete | 31.708 s, inside atmosphere |
| Speed at 35 / 36 / 37 s | 292,811 / 27,147 / 3,975 m/s |
| Speed at 38 / 39 / 40 s | 1,144 / 658 / 600 m/s |
| Orbit three complete | 40 s, approximately 9,144 m AGL |
| Low flight, 20 m AGL | 58 s |

These observations are not another set of timed steering controls.
The new distribution gate checks actual camera velocity: at least 55% of
the total logarithmic slowdown must have occurred by 35 s; 36–38 s may
contain at most 25%; no sliding two-second window may contain more than 35%.
No 100 ms interval may lose over 25% of speed. The old nominal 1/2/3-second
early-taper checks are removed because they passed the rejected behavior.

The numerical representation of the same curve is refined to 960 Hz through
the fast maneuver, with 120 Hz retained for regional flight. Orbital velocity
and acceleration use spatial derivatives and the same distance law; this
removes interpolation-induced speed increases without loosening tolerance.
The monotonic-speed audit checks knots AND midpoints at 1,920 Hz against a
running minimum, with the existing 10 ppm + 1e-8-world/s allowance. Capture
arc inversion uses finer long-interval quadrature; the geometric curve is
unchanged. All integration/refinement happens at initialization, not during
rendering. Sampling density is not a new flight segment or camera handoff.

Validation remains numerical/source/depth-based, not visual approval.
At the end of Phase 3S the 122-second wall and fixed-sun gates were failing.
The subsequent Phases 5/6 pass above fixes those failures; framing and expanded
performance gates remain unresolved. Do not claim an all-green audit suite.

Next gate: user live review of the stretched braking curve. Historical
checkpoint text below is superseded where it conflicts with this checkpoint.

## Previous checkpoint — Phase 3Q (historical)

**Phase 3Q: fast-jet speed at airline height by 40 seconds.** The latest
direction advances the aircraft-speed deadline from the previous 58.5-second
implementation to **no later than 40 seconds**, clarified as roughly Concorde/
F-15-scale forward speed (target 600 m/s) at 30,000 feet AGL. The first attempt
at a 133 m/s endpoint is rejected: easing absolute speed across the huge
space-to-aircraft ratio looked like an abrupt stop near 38 seconds. The new
law eases log speed, distributing the fractional slowdown over time, with
explicit checks of the arrival window rather than just its endpoint.
It retains the contained fast
orbit established in Phase 3P. The user explicitly
accepts an approximately 1.2-second first orbit at the retained incoming
speed, with braking around the planet. This resolves the previous speed/turn
question: do not introduce early braking or enlarge the tunnel.

Phase 3O is rejected. It replaced a roof excursion with a sideways excursion
and called any below-floor position contained. The corrected volume regression
rejected that path at 22.967 seconds before implementation changed.

The replacement has radius 2.4 world units, within the fixed tunnel's 2.53-unit
half-width. It travels over and under the planet, not in a wide horizontal
circle. Its approach is one monotone axial graph, matching position, tangent,
curvature and third derivative at the orbit; no lateral escape or dip-then-rise
correction. Entry inclination is 20 degrees. The first two revolutions retain
that plane. Regional orientation changes gradually during the slower third
revolution, after the primary brake, not during the fast swing. Phase 3Q
replaces independent normal/tangent blending with one normalized quaternion
rotation between those same entry and regional frames. The old blend produced
a high-curvature kink when its axes partly cancelled; accelerating the old
third-orbit schedule exposed it in the existing steering-rate test. This is a
correction to the third orbit's geometric reorientation, not a camera-attitude
patch. The approach and first two orbit planes are unchanged.

Third-orbit braking now controls world-distance speed, not angular speed.
Its C2 arc-length map is built once from the geometric curve, and the integral
of the smooth speed law determines duration and phase. Angular rate may change
as the curve bends, but actual camera speed must decrease throughout braking.
Neither this arc map nor the existing trajectory table is another world or a
new renderer. The incoming cruise remains full through the far side of orbit
one; its subsequent braking endpoint is retuned continuously to finish the
revolutions before the deadline.

The approved 0–20.5-second position and camera basis, incoming speed law, fixed
planet/grid coordinates, stars, ring geometry and gravity well are unchanged.
The below-floor exception is localized to a planet-centred ball of radius
2.53 and cannot override the side or roof bounds. Neither the world nor the
camera is shifted to fit the view.

| Observed milestone | Demo time |
|---|---:|
| Orbit entry | approximately 28.59 s |
| Far side of orbit one / braking begins | approximately 29.16 s |
| Orbit one complete | 29.733 s |
| Orbit two complete, inside atmosphere | 31.441 s |
| Arrival slowdown at 30,000 feet AGL | 38 s: 1,203 m/s; 39 s: 634 m/s |
| Orbit three complete, 30,000 feet AGL and fast-jet speed | 39.679 s |
| Explicit speed-and-altitude deadline | 40 s: approximately 600 m/s, 9,144 m AGL |
| Sustained high pass | 41 s: approximately 600 m/s, 9,144 m AGL |
| Low flight, 20 metres AGL | 57.679 s |
| Continuing canyon flight endpoint | 137 s |

The first orbit lasts about 1.141 seconds, orbit two about 1.708 seconds, and
orbit three about 8.238 seconds, still more than twice orbit two. There is no
phase plateau or position reset.
After two seconds at aircraft height, the same track descends over sixteen
seconds into the canyon. Speed stays about 600 m/s during the high pass, then
eases down toward the existing 133 m/s canyon cruise throughout that descent.
The regional phase is the integral of this continuous speed law, not a reset
at either height. This keeps the longer remaining flight inside the same
existing canyon through 137 seconds; terrain is not moved or extended.

The new deadline regression first failed the previous implementation at
40 seconds with an actual velocity of 2,392,047 m/s. It now checks world-space
camera velocity at 40 seconds (550–650 m/s), simultaneous 9,000–9,300 m AGL,
the 38–41-second arrival window, aircraft-height completion by 40,
and monotone braking at 240 Hz through orbit completion, allowing only
sine-table/finite-difference numerical error against a running minimum (no
accumulating allowance). The third-orbit brake must not lose more than 30%
of actual speed in any 100 ms; this rejects the old last-frame speed collapse.
Independent quadrature
checks the arc-length map between its knots. The older 20–30-second duration
requirement for orbit three is superseded by the explicit 40-second deadline;
the requirement for orbit three to last more than twice orbit two remains.

Validation is numerical/source/depth-based, not visual approval. The strict
volume oracle checks the actual rounded-to-square cross-section, unconditional
side and roof bounds, and the localized underpass. Negative witnesses reject
the previous sideways-under-floor loophole, early floor exits, excessive
depth and circular-corner escapes. The production path is sampled at 120 Hz
through 137 seconds, with dense additional capture/orbit geometry samples.

Fast turning is now explicitly authorized, but discontinuities are not.
Capture tests require monotone inward approach, no upward counter-turn, and
curvature no greater than the joined orbit. Position, tangent and curvature
must match at the join. Additional samples straddle every interpolation knot
and midpoint for acceleration, jerk and direction-rate spikes. Observed
maxima: 75.88 world units/s², 1,016 world units/s³ and 6.356 rad/s. Bounds scale
with the authorized circular orbit: 2*v²/r acceleration, 4*v³/r² jerk and
1.25*v/r direction rate. Camera angular acceleration is limited to 2*(v/r)².
These replace the incompatible slow-orbit rate cap, not the continuity tests.

The background parallax audit now compares views separated by orbital phase,
not a half-second (nearly opposite hemispheres at this speed), and removes
camera rotation from the parallax measurement. The small-globe centre must
remain visible; once the globe fills the view, geometric horizon/sky and
surface-coverage checks replace the obsolete requirement to keep the planet's
centre on screen.

Run `make audit-flight` for the strict volume, motion, source, depth,
refinement and determinism/performance checks without stored image baselines.
Phase 3Q numerical deadline, continuity and containment checks pass, together
with direct Phases 0–4 and 7, release build/check and `git diff --check`. The full
`make audit-flight` run is **not green**: Phase 5's fixed 122-second witness
finds zero rendered wall-depth samples on either side after this
retiming. Keep that failure visible; it has not been fixed or waived.
Phase 6 remains separately incomplete: its old atmospheric-entry grid witness
is not exposed along this path, and fixed-sun aircraft composition was already
unresolved. Neither is hidden by a claim that every phase passes.

Latest live feedback: the motion still feels unnatural; the user asks to
"pull the curve back" because there is room. Clarification is pending on
whether this means starting the downward spatial turn farther before the
planet or starting braking earlier. Do not silently change the approved
incoming trajectory or the previously specified far-side braking onset.
The 40-second fast-jet speed/airline-height target and strict tunnel bounds
remain binding. No further production path change has been made for this
ambiguous request yet.

Next gate: resolve that direction and correct the live motion. Do not
treat numerical success as visual acceptance. Preserve the dirty workspace.
Older status narratives are historical and their contradictory geometry,
timestamps and acceptance claims are superseded by this checkpoint.

The active user direction has five parts:

1. Zigzag artifacts were reported along the lower planet and later on the
   globe during braking, together with a movement glitch near 35 seconds.
   Local fades or attitude blends are not the final remedy; consolidation must
   remove the duplicate guide geometry and separately evaluated motion that
   made those defects possible.
2. The accepted downward curve stays fixed, with braking beginning smoothly
   at about 25.658 seconds, 3–4 seconds earlier than the previous far-side
   onset. Starting the camera bank cannot independently alter flight velocity.
   The same speed law continues through approach and orbit. The descent reaches
   approximately 30,000 feet AGL by completion of orbit three, with actual
   fast-jet speed (approximately 600 m/s) reached no later than 40 seconds.
3. At 30,000 feet the demo must sustain an airplane-scale, predominantly
   downward view of a detailed fractal continent. Surface motion must be slow
   enough to read. Atmospheric optical depth must have removed the background
   grid and stars from the sky; only the persistent world-space sun remains
   visible there.
4. The approach trajectory is now almost perfect, but the implementation must
   be consolidated into one geometric model. A visually smooth join between
   separately evaluated paths or representations is not sufficient.
5. The starfield, streak region, logarithmic-ring region, circular tunnel,
   squared tunnel, bottom-plate gravity well, and mesh beyond the planet are
   spatial parts of one world-anchored field. They are not time-selected scenes,
   a camera-following transport, or a finite sliding window of recycled rings.

The previous post-capture schedule is rejected. It delayed meaningful descent
until after four turns, then spent almost four more turns approaching the
valley. That produced long intervals which alternated between repetitive and
visually imperceptible. Preserve the accepted capture and first revolution,
then redistribute the remaining 137-second timeline toward one obvious
braking/descent maneuver, a sustained high-altitude landscape pass, and the
later valley approach.

The last session traced the lower-edge problem to two separate concerns. The
52-second rendered silhouette already matched the canonical sphere at its
upper edge, but that audit did not inspect the lower edge. It now checks both
edges. More importantly, the near plane is perpendicular to camera forward
while sphere intersections are measured along oblique rays. At a wide field
of view a ray can therefore continue through solid terrain after its near
surface has crossed the near plane. Treating that ray as a miss exposes a
serrated strip of background. The resulting edit in
`sample_surface_vertex` assigns such a solid ray to a point just beyond the
camera near plane; the independent canonical-core audit was corrected to test
the far intersection against that plane. This is solid-surface clipping, not
a mask, fade, or alternate renderer.

The near-plane edit has now been rebuilt and validated. The expanded
52-second limb regression initially treated the bottom viewport crop as a
spherical limb; it now checks wire visibility only where the canonical upper
or lower limb is actually on screen. Near-plane-owned solid-interior samples
also suppress the false mesh accent produced by their temporary clip-plane
normal. Fill coverage remains exact: the rendered 52-second upper and lower
edges have zero offset from the canonical sphere and a maximum one-pixel
adjacent step. Subsequent diagnostic work traced the remaining artifact to
isolated cyan coarse-mesh accents at the bottom viewport crop.
Those accents now fade continuously through the existing projected-footprint
LOD as mountain detail resolves. The Phase 4 regression checks both the true
canonical limb and the separately cropped viewport edge at 52 seconds.

The prescribed validation sequence completed on 2026-09-05:

```text
cargo check --release
cargo build --release --features phase0-audit --target-dir target/phase0-audit
printf '\045' | target/phase0-audit/x86_64-unknown-linux-gnu/release/termdemo
make audit-phase0
make audit-phase7
git diff --check
make check
make size
```

The historical Phase 4 and Phase 7 audits passed before the trajectory retune,
but they are not acceptance results for the unified model. All 51 exact hashes
through the accepted first orbit remain unchanged. `git diff --check`,
warning-free `make check`, and `make size` also pass. The optimized executable
is 82,384 bytes, or 81,960 bytes with section headers stripped.

The 2026-09-05 consolidation pass was validated without generating or
inspecting frames. Warning-free release and audit-feature checks pass; direct
Phase 0–4 numerical audits pass; the rejected compatibility identifiers and
independent globe renderer are absent from `src/main.rs`; and `git diff
--check` passes. The current optimized executable is 70,136 bytes with a
1,658,996-byte static working set. Protected frame hashes were deliberately
not run or regenerated during this pass.

A post-consolidation regression first collapsed the initial star geometry,
then a frame interpolation introduced a visible squeeze at 27 seconds. The
attempted repair was architecturally wrong: it reconstructed field vertices in
the current camera basis, scaled them from current planet depth, and constrained
camera aim from projected pixel bounds. Those feedback loops made the planet
appear pinned to the wrong grid plate and violated the one-world requirement.

The camera, fixed planet, displaced surface, projection, depth pipeline, and
star-to-tunnel field now share one world model. The production renderer uses a
fixed longitudinal field coordinate and permanent logarithmic ring identities;
the old time-transported field and ring-age placement survive only as
audit-feature compatibility oracles and cannot be called by rendered code.

The required replacement is a world-anchored field manifold. A permanent
longitudinal coordinate `s` runs from the distant starfield through the streak,
logarithmic-ring, circular-tunnel, and squared-tunnel regions, across the fixed
planetary well, and onward beyond the planet. An angular/lane coordinate
`theta` identifies position around the cross-section. `field_point(s, theta)`
must be independent of demo time and camera position. The camera moves through
this structure; the horizon is its projected vanishing point, not a moving
world origin. The tunnel is cylindrical in world space and appears conical only
through perspective. The bottom plate is one part of that same tunnel surface
and is displaced by a localized field centred on the fixed planet.

Phase 3A is complete in production code. `FieldLayout` owns a fixed `f64`
orthonormal frame, tunnel radius, planet coordinate, post-planet extent, and
fixed deterministic spatial boundaries for all six field regions. `field_point(layout,
s, theta)` has no time, camera, projected-depth, or ring-age input; its circular
region is a physical cylinder, its later cross-section is a true square, and
its exact bottom cardinal point meets the fixed planet through the localized
well. The active renderer now consumes this field directly.

The first consolidation pass exposed a gap the old Phase 3 test could not see:
it sampled star positions at zero seconds and then jumped to the late approach,
so the camera could move away from the field and leave no projected stars at
four and seven seconds while both sampled endpoints still passed. The approach
camera moves from its initial world state with monotonically decreasing planet
distance. Phase 3 samples
the complete 0–11-second star interval every half second and requires projected
and visible counts, spatial coverage, and raster-length streaks after seven
seconds. A mere projected midpoint no longer counts as a surviving streak.

A second regression proved those distribution checks were still insufficient:
an approximation had removed the canonical minimum exit-speed normalization
and replaced it with an organization-weighted radial scale. The original
`flyby_segment` arithmetic order, constants, squared radial law, and
`exit_scale = max(0.65 / |motion|, 1)` are preserved in audit-only compatibility
oracles locked by independent source digests. The Phase 3C proof established
that the old 120 Hz endpoints cannot all belong to fixed world identities, so
they are no longer production equations. The replacement instead protects the
approved key events and spaces: stars at the opening, streaks at 4 seconds,
logarithmic rings at 11 seconds, the circular tunnel at mesh maturity, and the
round-to-square transition at 21 seconds. Those events are fixed locations on
one field axis, reached by one continuously evaluated camera trajectory.

The approach camera begins on the centre of the horizontal field rather than
the planet centreline, then follows compact C2 lateral and vertical maneuvers
inside the same world-space trajectory before returning exactly to its capture
state. The planet remains fixed below the field axis at the centre of the
bottom-plate gravity well. This is world-space camera motion, not a projected
framing correction. Camera altitude and the near plane are measured from the
actual world position to the canonical displaced surface.

Do not generate, inspect, compare, or tune from screenshots, contact sheets,
rendered stills, image-generation tools, or image-editing tools. Existing frame
hashes may remain as automated change detectors, but they cannot define motion,
prove geometric continuity, or justify compatibility geometry. Visual testing
belongs exclusively to the user's live review unless the user explicitly
reverses this instruction.

Performance is resolved for the current automated gate. With release
`opt-level = "z"`, the 97-second frame consumed roughly 129–140 ms of process
CPU time. Temporary component profiling attributed approximately 67 ms to the
star/grid background, 12 ms to atmosphere, and 52 ms to the surface. The
terrain sampler now reuses spherical map coordinates and release optimization
is level 3. Higher optimization caused this `no_std` binary to require `memcpy`
and `memset`, so small x86-64 implementations were added to the existing
startup assembly. Phase 4 and the isolated Phase 7 all-checkpoint hardening
gate pass the 33.3-ms maximum. Phase 6 remains open on its sun-composition
requirement, not on field geometry.

The fixed universe catalogue is now culled only by geometry and physical
atmospheric extinction. Once the camera reaches the optically opaque 30,000-foot
regime, the renderer skips projection of the extraterrestrial catalogue because
its continuous visibility is exactly zero; it does not test demo time or a scene
label. This restores the 33.3-ms Phase 7 gate while matching the requirement
that the grid and stars have disappeared and only the fixed sun remains in the
sky. Future performance work must preserve that rule and use frustum, depth,
surface coverage, or optical depth rather than timeline-based hiding.

Current implementation state after geometric consolidation:

- the stored 51-frame Phase 0 baseline currently reports mismatches at every
  checkpoint. A clean build of committed `HEAD` reproduces the current 0.5-s
  hash rather than the stored hash, proving that the Phase 3A/3B audit-only
  additions did not cause this discrepancy. The baseline remains unmodified
  and unblessed while its earlier source/toolchain provenance is resolved;
- the revised Phase 1 precision and Phase 2 trajectory audits pass on one
  continuously integrated trajectory state;
- Phase 3 now uses a fixed universe catalogue and the single displaced surface
  in Phase 4 remains implemented; production field geometry has no time,
  trajectory, ring-age, or camera-placement dependency;
- every required performance checkpoint is below the 33.3-ms hard maximum;
- `git diff --check`, release checks, the source gate, and direct Phase 0–4 and
  Phase 7 audits pass without warnings; Phase 5 now passes destination-depth
  and canyon-track checks. Phase 6 remains red on fixed-sun exposure;
- the separate guide globe and screen-derived planet proxy have been deleted;
  unresolved and polygon coverage now sample the same displaced sphere, and
  the artifact-producing surface line accents are absent;
- direct Phase 0–5, Phase 7 and the new flight-requirements gate encode the
  revised flight; terrain/city visual acceptance and sun composition remain
  open;
- the orbit-relative braking profile and sustained 30,000-foot camera pass are
  useful behavioral targets; the user reports that the approach is almost
  perfect, so consolidation must preserve that character without preserving
  the rejected path seam;
- continuous field extinction by the actual 30,000-foot altitude is
  implemented; detailed high-pass terrain art and a reliably exposed fixed sun
  remain open.

## Immediate next implementation order

1. Completed on 2026-09-05: the protected baseline now contains 51 exact
   frames through the current first-orbit completion at 34.97835 seconds, plus
   513 orbit-one motion samples.
2. Trajectory half completed on 2026-09-05: `trajectory_state(time)` now
   evolves one radius/phase/meridian state from time zero. The old post-orbit
   path, boundary reconstruction, tolerance guard, and alternate camera frame
   are deleted. Direct numerical audits cover state derivatives, orbit timing,
   braking, displaced-surface AGL, deterministic seeking, and endpoint motion.
3. Completed on 2026-09-05: the legacy planet proxy, separate wire globe,
   geometry-time offset, compatibility world translation, and dual surface
   ownership are removed. The planet is fixed at the canonical origin and all
   surface coverage uses the same displaced-sphere sampler.
4. Completed on 2026-09-05: Phase 3–4 direct audits now test state identity,
   canonical placement, shared bounds, one intersection source, aspect behavior,
   and displaced-surface AGL without generating or inspecting frames.
5. Completed on 2026-09-06, Phase 3A: define the permanent field manifold
   `field_point(s, theta)`, its fixed axis, physical tunnel radius, planet
   coordinate, and spatial boundaries for stars, streaks, logarithmic rings,
   circular tunnel, squared tunnel, and post-planet continuation.
6. Completed on 2026-09-06, Phase 3B: fixed signed
   ring/chunk identities now span the whole field with geometric axial spacing;
   stars and mesh edges share the same permanent ring/lane topology, and a
   physical streak candidate projects one fixed identity across an exposure
   interval.
7. Completed on 2026-09-06, Phase 3C: multi-time camera-ray proofs show that
   the canonical star centres and ring-age mesh cannot be fixed world points.
   The exact star-centre witness requires a moving, approach-synchronized
   world trajectory; some old streak endpoints cannot be two observations of
   that particle at 0.0001-pixel precision. The fixed mesh is likewise
   mathematically incompatible with the old ring-age projection. These
   findings explicitly forbid using exact-output compatibility as a reason to
   retain the co-advected field.
8. Structurally completed but visually rejected on 2026-09-08, Phase 3D: the
   renderer switched atomically to fixed field coordinates, but retained a
   spatial star-to-ring organization law inherited from the old staged model.
9. Implemented on 2026-09-08, Phase 3E: stars now form an independent permanent
   3D catalogue; rings and rails are separate permanent objects beginning at
   their fixed physical boundary. Production has no star-to-ring position law,
   inverse visibility partition, or renderer handoff. Current gate: live review
   of the continuous 0–28-second traversal before Phase 5.
10. Implemented on 2026-09-09, Phase 3L: preserve the exact approved opening
   through 20.5 seconds, then traverse one C2 world-space capture centerline
   into a radius-15 planet-centred orbit. Carry residual arc distance exactly
   across the polynomial-to-circle join; hold full speed through phase 512;
   brake and reduce radius as functions of orbital phase; arrive at 30,000 feet
   at the former 58.325-second target. Phase 3P supersedes its orbital plane,
   radius, cadence and containment. Current gate: live review of Phase 3P.
11. Once motion and unified geometry are accepted, retune Phase 5 terrain LOD
   and art for the long high-altitude continent view, with tracked persistent
   landmarks.
12. Implemented on 2026-09-06: the fixed field's visibility is derived from
   physical atmospheric optical depth and reaches zero by 30,000 feet, while
   the persistent world-space sun remains separate. Phase 7 determinism and
   all-checkpoint performance gates pass with this extinction in place.

## Single-model consolidation gate

“One world” means shared data and equations, not merely similar outputs or a
smooth crossfade. Before Phase 5, the rendered program must satisfy all of the
following structural requirements:

- One `trajectory_state(time)` law produces position, velocity, acceleration,
  forward, down, and roll for the entire 137-second path. It evolves one state
  under continuous braking and steering controls. It may not select an orbit
  position formula before a timestamp and a reconstructed descent formula
  afterward, accept a stored boundary position, or estimate new initial
  conditions from finite differences.
- Orbit, braking, high pass, and valley arrival are observations of that one
  state evolution. Orbit crossings and actual altitude observations constrain
  motion, but they are not scene joins,
  position resets, coordinate changes, or ownership boundaries.
- Grid, well, planet, atmosphere, terrain, clouds, and valley have canonical
  world positions at the same requested time. Remove `geometry_time` and
  `world_shift` as compatibility motion. Camera-relative origin subtraction is
  allowed only as a precision operation on those same-time world positions.
- One fixed field coordinate system owns the starfield, streaks, logarithmic
  rings, circular tunnel, squared tunnel, gravity-well plate, and mesh beyond
  the planet. A stable `(s, theta)` identity maps to the same world position at
  every requested time unless that identity is explicitly a moving particle
  with a world-space velocity law.
- Spatial character is derived from `s`, not demo time: connectivity, ring
  density, circular-to-square deformation, color organization, and proximity
  to the planet may vary along the field, but the camera crossing a timestamp
  may not relocate or replace the structure.
- The field axis, tunnel radius, region boundaries, ring coordinates, and
  planet coordinate are fixed world data. Visible-range selection may cull
  stable identities but may not recycle a finite ring window into new world
  positions.
- The camera is never an input to `field_point`. Camera position is used only
  after world placement for subtraction, clipping, projection, motion-blur
  measurement, and visibility. The horizon point is a projection result, not
  a field origin translated with the camera.
- One canonical displaced-sphere function owns surface position, normal,
  material coordinates, relief, and AGL. Every fill, line, depth sample,
  atmosphere limit, collision/altitude query, and LOD sample consumes that
  function or vertices produced directly from it.
- Remove the screen-derived planet proxy and the separate wire globe.
  Projected bounds must be derived from the canonical sphere and camera. Any
  remaining line accent must be an edge or isoline of the same persistent
  surface vertices, not an independently sampled latitude/longitude sphere.
- Different raster primitives are permitted only as coverage strategies for
  the same surface samples. They may not carry separate positions, depths,
  silhouettes, visibility clocks, or geometry. Resolution changes derive from
  projected footprint and may not select another geometric representation.
- Source audits must reject post-orbit position branches, boundary-state
  arguments, duplicate planet centers/radii, screen-to-world planet lifting,
  and alternate surface ownership. Numerical audits must sample the one state
  directly and require derivative agreement near floating-point precision,
  with bounded jerk rather than a loose discontinuity allowance.

The current approach and first orbit remain the behavioral target. Exact frame
hashes are diagnostic, not permission to retain a second geometric model. If
consolidation necessarily changes protected pixels, keep the old hashes as a
visible failing reference and do not replace them until the user has approved
the unified live sequence.

## Non-negotiable invariants

1. The entire demonstration is one world observed by one continuously moving
   camera. There are no scenes.
2. Every visible object belongs to that world. Nothing may be screen-locked,
   introduced midway along the flight path, moved independently to preserve a
   composition, or removed before it physically leaves the view.
3. The star field, streaks, threshold rings, radial mesh, squared tunnel,
   gravity well, planet, continents, terrain, mountains, and valley have one
   canonical world-space identity. The starfield, streak, logarithmic-ring,
   circular-tunnel, squared-tunnel, and post-planet areas occupy named ranges
   of one fixed longitudinal coordinate. Apparent stages arise from camera
   motion, deformation, resolution, illumination, and atmosphere—not
   replacement geometry or a blended coordinate system.
4. One continuously evaluated state law owns camera position, velocity,
   acceleration, forward direction, roll, and the apparent motion of nearby
   geometry. Timeline targets may influence continuous controls but may not
   divide the position or attitude into separately initialized paths.
5. The direction of orbital travel never reverses. Braking changes speed, not
   direction.
6. The planet remains a sphere at every distance and every terminal aspect
   ratio. Terminal compensation is applied after projection and never deforms
   world geometry.
7. The planet is anchored halfway into the bottom grid at the center of a
   pronounced deformation of that same grid. Planet, well, and grid share one
   world coordinate system and projection; neither the planet nor the field is
   translated to follow the camera.
8. The grid disappears during atmospheric entry only through physical
   occlusion and atmospheric optical depth. It may not be hidden by a wipe,
   rectangle, disk, opacity timeline, or scene replacement.
9. Surface detail becomes resolvable continuously. Continents, ranges, and the
   valley must be visible at an appropriate earlier scale before the camera
   reaches them.
10. Escape always exits. Navigation and fast-forward controls must not alter
    the trajectory or leave terminal state unrestored.
11. The first frame and the last frame use one world, state evaluator,
    camera-space clipping path, projection, depth convention, and canonical
    surface source. Points, lines, and filled polygons may use specialized
    raster primitives only when they consume that same geometry and depth;
    time may never select a second model or owner.
12. The accepted approach and the latest smooth, braking first orbit are the
    behavioral and aesthetic target. Replacement equations must reproduce signed-off motion
    exactly; similarity, endpoint agreement, and aggregate visibility are not
    acceptable substitutes. Preserve canonical laws through explicit
    equivalence tests while expressing their coordinates in the single world.
    Never retain a second geometry solely to satisfy old pixel hashes. Any
    necessary protected-interval change remains unblessed until the user
    approves the unified live motion; baseline changes may not be made silently.
13. Do not use still imagery, screenshots, contact sheets, or generated images
    to design, diagnose, or approve motion and geometry. Development evidence
    is source structure, numerical state/derivative audits, deterministic
    seeking, performance measurements, and the user's live observations.
14. A mesh point's world position may not depend on demo time, camera position,
    projected depth, or ring age. Moving star particles are permitted only when
    their motion is an explicit world-space trajectory within the same field.

## Physical scale

The working planet has a diameter of 27,000 miles and a radius of 13,500 miles.
Its canonical render radius is `0.115`.

| Quantity | Physical value | Planet-radius ratio | Canonical value |
|---|---:|---:|---:|
| Planet radius | 13,500 mi | `1.0` | `0.115` |
| Airplane-pass altitude | 30,000 ft / 5.6818 mi | `0.000420874` | `0.0000484005` |
| Airplane-pass center distance | 13,505.6818 mi | `1.000420874` | `0.1150484005` |
| Final camera altitude AGL | 20 m / 0.012427 mi | `0.0000009205` | `0.0000001059` |

The airplane-pass-to-valley altitude ratio is approximately 457:1. At 30,000
feet the camera is already deep inside the atmospheric shell and the planet
reads as a broad landscape rather than a distant globe. At 20 metres, the
visible surface is locally almost flat even though every point remains on the
same sphere.

Trajectory and terrain placement must use `f64` through camera-relative
subtraction. Only the resulting local vectors should be converted to `f32` for
projection and rasterization. At the current world-coordinate magnitude, a
20-metre canonical altitude is too close to `f32` precision for stable direct
subtraction.

## Accepted sequence and current boundary

The opening and world-space sequence remain protected. The latest motion
revisions and unresolved orbital requirements are recorded in Working state
above; older orbit timings below are not fresh approval.

1. The camera first sees the colored-star range of the fixed field, with
   occasional attached sparkles. Preserve the approved opening density/motion.
2. Acceleration and outward star streaks belong to this opening wormhole rush,
   completed around 4–5 seconds. They must not rebuild afterward. Stars become
   points on emergence; they do not become or replace grid spokes.
3. The distant fixed ring structure is already visible on emergence. The camera
   approaches at high speed and gradually decelerates; it does not accumulate
   speed to enter the grid. Far rings converge at the horizon vanishing point
   and expand under perspective without changing world identity or shading.
4. Fixed radial connections assemble progressively along the same field
   coordinate, independently of the star exposure. Preserve this assembly.
5. Connectivity becomes the clean circular tunnel without replacing the field.
6. The same cylindrical tunnel continuously squares along `s`; perspective,
   not conical world geometry, produces the apparent cone.
7. The planet is first visible by 27 seconds on the bottom grid, halfway inside its
   gravity well.
8. Broad off-axis curvature begins from the complete approved 20.5-second
   world state and carries the same velocity into high orbit. There is no
   special movement or force change at 28 seconds.
9. Demo time and trajectory time remain strictly 1:1. The incoming velocity is
   redirected from radial approach into tangential motion without replacing
   the camera or renderer.
10. The first complete revolution after capture retains a fast, legible swing.
    Cruise speed holds through the far side near 29.16 seconds; the revolution
    completes near 29.73
    seconds as an observed milestone in one state evolution, not a point at
    which another path is initialized.
11. Orbit two is the visible braking and descent maneuver. Speed and altitude
    fall strongly but continuously, orbital direction never reverses, and the
    camera pitches progressively toward the continent.
12. By completion of the substantially longer orbit three, the camera is at
    30,000 feet displaced-surface AGL and at no more than one quarter of its
    first-orbit completion speed. There are no further global holding orbits.

Use the existing protected hashes only to detect changes while the model is
being consolidated. Motion evidence comes from dense samples of the one camera
state from 28 seconds through the full path. That state follows a regional
ground track across the same continent, holds the airplane-scale interval, and
finally bends toward the existing valley. Orbit-to-regional motion is a change
in the continuously applied forces and curvature, not a new path evaluation.

## Target orbit-to-valley timeline

Orbit-relative requirements are binding; the measured times at the current
checkpoint are observations of the curve, not scene switches. Preserve the
137-second endpoint.

- Capture: preserve the exact incoming speed law through 28 seconds; enter
  below 30 degrees; never cross the roof or side walls. Only a bounded
  under-planet region permits passing below the floor.
- Orbit one: full space speed through the far side, then smooth braking on
  its back half. Keep significant space above the horizon.
- Orbit two: remove the remaining space speed and enter the atmosphere by
  completion. Retain upper sky while the planet grows.
- Orbit three: substantially longer (numerically, more than twice orbit two),
  ending at 30,000 feet AGL with readable aircraft-scale ground motion,
  continent features and procedural daytime/nighttime settlements.
- Start of orbit four: continue the regional ground track; atmospheric
  extinction removes grid and stars. No fourth full holding orbit.
- Reach 20 metres AGL 15–20 seconds later (currently 18 seconds). See the
  canyon ahead during descent and continue between its walls to the endpoint.

No milestone may initialize another camera, reset velocity or position, hide
a bad join, or clamp phase until a chosen timestamp.

## One camera and one trajectory

`FlowCamera` is a value returned by the one trajectory-state evaluator for the
entire demo, including straight flight before the planet. It is not itself a
collection of timeline-specific camera formulas. Continuous control fields
curve the same evolving state at capture, braking, entry, and valley approach;
no camera or path is created, activated, blended, reconstructed, or handed off.
The legacy `CameraPose`, `journey_position`, and `journey_camera` path must not
drive any rendered element after migration.

At every point in the descent, camera position has the form:

```text
planet_center
  + surface_normal * (planet_radius + terrain_height + altitude_agl)
```

The surface normal follows one spherical ground track generated by the same
state. Orbit two and orbit three retain same-direction global curvature while
continuous forces descend and brake the camera. After orbit three, curvature
opens into a regional path across the
selected continent and then bends toward the existing valley coordinates; it
does not interpolate between control positions, reconstruct state at a named
time, or reverse angular direction.

Camera forward is derived from the position derivative. Planet-relative down
is the surface normal projected perpendicular to forward. Any artistic pitch
or bank is a smooth offset from this physical flight frame, with matching
derivatives at its endpoints; it is never an independent look-at animation.

The first orbit enters at 20 degrees in an over-and-under plane. Both initial
orbits retain significant sky above the horizon as altitude and speed fall. By orbit three the surface occupies
roughly 70–85 percent of the frame, with a small atmospheric sky band retained
for the sun. Avoid alternating between planet-hidden and planet-dominant
compositions; the continent remains continuously readable through the braking
maneuver.

The in-atmosphere attitude reaches a high-oblique aircraft view at completion
of orbit three. Keep readable ground motion and a sky band, continuing into
the valley over the following 15–20 seconds. The valley appears below and
ahead before its walls surround the camera; no independent look-at path.

## One planetary surface

There is one procedural spherical surface:

```text
surface(normal) = planet_center
  + normal * (planet_radius + terrain_height(normal))
```

The current terrain sampling and landing map must be expressed through that
single height function and in the physical unit system above. The old
1,000-unit planet cannot remain as a second rendered planet.

Distance changes tessellation and color detail, not identity or renderer:

- First sight uses a coarse spherical polygon mesh, with restrained wire
  accents drawn from the same vertices.
- The visible continental patch subdivides from existing parent cells.
- Regional and mountain wavelengths become active only as their projected
  footprint becomes resolvable.
- Child vertices geomorph from their parent surface so subdivision cannot pop.
- The same displaced cells become the valley floor and walls.
- Curvature remains in the equations at 20 metres even when it is no longer
  visually obvious.

Terrain detail must be band-limited by projected footprint. Higher-frequency
fractal octaves resolve progressively from existing low-frequency landforms;
they do not appear as a new texture or replacement landscape.

The accepted visual target is a colorful 1990s *Populous*-like landscape, not
photorealism. Low-poly filled cells, readable height tiers, oceans, continents,
mountain chains, valleys, and selective mesh lines are desirable. Geometry
must be interesting and visibly fractal across scales, but surface fidelity is
subordinate to smooth continuous flight and a stable frame rate. Surface color
comes from persistent world-space properties such as elevation, slope, biome,
water, and sun direction; it is not a screen-space effect or a scene palette
swap.

The 30,000-foot pass is now the primary surface-art checkpoint rather than a
brief transition. It must show nested continental structure, coast or drainage
organization where applicable, multiple terrain biomes, mountain chains, and
smaller relief inherited from those parents. The image may remain deliberately
low-poly, but it may not read as a flat color field, blurred noise, or a generic
texture. Track several persistent landmarks across the pass and keep their
screen motion slow enough that their shapes can be followed for multiple
seconds.

## Visibility and depth

Grid and planetary geometry need a shared depth test. A compact fixed-point or
inverse-depth buffer is preferred if it preserves executable size and
precision. All planet, terrain, grid, cloud, and atmospheric samples use the
same camera-space depth convention.

That depth convention is active from the start of the demo. An empty or
trivially cleared early depth buffer is still the same buffer and pipeline; it
must not be introduced at 28 seconds as a new rendering mode.

The planet physically occludes grid lines behind it. Grid lines in front remain
visible. Near-plane clipping must preserve crossing segments instead of
dismissing an entire line when one endpoint is behind the camera.

There are no screen-space black-hole masks after the camera passes the clean
threshold. The clean background may be black, but it cannot be an independent
shape that moves or occludes world geometry.

## Accepted rendering architecture

Every frame follows one pipeline:

```text
canonical world geometry and continuous material flow
  -> one FlowCamera transform
  -> shared camera-space clipping and culling
  -> one aspect-correct projection
  -> point, line, or triangle raster primitive
  -> one depth convention
  -> restrained world-space lighting and atmosphere
```

Different primitive types and levels of detail are implementation details
inside this pipeline. They may be selected by topology, projected footprint,
or a fixed work budget. They may not be selected because the clock crossed a
scene time. In particular, 28 seconds must contain no planet-renderer switch,
grid-projection switch, depth activation, or `Option<FlowCamera>` branch. The
only event there is the smooth change in the derivative of the existing flight
path as it bends toward orbit.

The planet and terrain use a fixed-capacity polygon mesh on one stable screen
lattice. A coarse sphere is already present at first visibility. The lattice's
world-space footprint contracts continuously as the camera approaches, while
terrain child bands geomorph from their persistent parents according to that
footprint. Fill, resolved wire accents, occlusion, and terrain placement all
use the same surface sampler and depth convention. Terminal aspect
compensation is applied only after projection, so the sphere cannot become an
ellipse in portrait terminals.

Terrain bands and colors are deterministic in persistent spherical
coordinates. Expensive fractal data is generated or cached once, then sampled
through a band-limited hierarchy. The frame loop must not ray trace every
screen pixel, repeatedly intersect displaced surfaces, or reevaluate several
noise hierarchies per pixel.

The reference performance gate is the headless 320x200 render computation,
excluding terminal transmission and one-time startup generation. It measures
process CPU time so unrelated host scheduling contention does not inflate the
renderer's compute cost; interactive playback deadlines continue to use the
monotonic wall clock:

| Requirement | Budget |
|---|---:|
| Target frame time | at most 16.7 ms (60 fps) |
| Hard maximum at any timeline checkpoint | 33.3 ms (30 fps) |
| Surface work | fixed vertex and triangle budgets |
| Terrain generation | cached, mipmapped, or incrementally reused |

Performance is a design constraint, not a final polish pass. If an effect
breaks the hard maximum, reduce detail density, projected subdivision, or
shading complexity without changing geometry identity, camera motion, or the
accepted appearance through the first orbit.

## Atmosphere, plasma, clouds, and sun

The atmosphere is a spherical shell around the same planet. Its optical depth
depends continuously on camera altitude and ray path length.

- Above the atmosphere, the existing grid and planet remain fully visible.
- Atmospheric color accumulates over those pixels rather than replacing them.
- Grid visibility decreases only according to optical depth along its ray.
- By 30,000 feet, transmission for background grid and star rays through the
  visible sky must be below one display-code value; neither may remain visibly
  patterned. This threshold is altitude/optical-depth driven, not time driven.
- Entry plasma depends on atmospheric density and actual flight speed.
- Plasma begins faintly, strengthens through denser air, and remains attached
  to the camera/flow field rather than becoming a scene overlay with its own
  motion.
- Clouds use the same spherical coordinates as terrain and retain their world
  positions during approach.
- The sun is projected through the unified camera and keeps one world-space
  direction from the grid through the valley.
- At the airplane pass, the atmospheric sky may contain scattering and the
  persistent sun, but no other visible background geometry. Clouds remain
  attached to the planetary surface and may appear below or within the
  landscape view rather than behaving as sky-locked decoration.

## Legacy code migration

The following existing routines describe useful visual work but are not an
accepted continuation because they use the older camera or scale:

- `journey_position`
- `journey_camera`
- `render_intro_surface`
- the full-screen ray-intersection path in `render_flow_surface`
- `render_world_surface`
- `fill_atmospheric_sky`
- `draw_world_sun`
- the atmospheric intensity currently derived from legacy camera altitude
- every `flow.is_none()` or equivalent time-selected projection branch
- `FLOW_POST_ORBIT_START`, `flow_first_orbit_boundary`,
  `flow_first_orbit_phase_state`, `flow_descent_relative`, and every boundary
  tolerance or argument used to initialize a later path
- `geometry_time` and compatibility `world_shift`
- `grid_planet_view` and `projected_planet_view` when used as planet geometry
  or bounds rather than reporting only
- the independently sampled `render_wire_planet` globe

Their terrain, color, lighting, cloud, and plasma formulas may be migrated into
the unified system. Their camera positions, planet radius, visibility gates,
implicit scene timing, and separate raster assumptions must not be retained.

## Implementation phases

Phases are dependency-ordered and gated. A later phase must not compensate for
an unresolved failure in an earlier one. Each phase ends with source and
numerical checks and, where motion or appearance changes, the user's live
review. The agent does not create still-image evidence or award its own visual
approval.

### Phase 0: Protect the accepted journey

**Goal:** retain a diagnostic record of the accepted star-to-grid sequence and
the former capture baseline while the geometry is consolidated.

**Status:** complete. `make audit-phase0` protects 51 exact 320x200 frames
through the accepted first-orbit boundary at 34.97835 seconds, samples the
27–29 second camera and grid boundary at
0.01-second intervals, checks the one-camera source contract, and exercises
motion, aspect ratio, input, and terminal cleanup. The control test fragments
the Right Arrow escape sequence across delayed reads, so a buffered-sequence
happy path cannot conceal accidental exit behavior. Frame hashes live in
`tests/phase0_frames.sha256`; recorded motion constraints live in
`tests/phase0_motion.txt`. The former orbit is additionally sampled 513 times
from capture through its exact one-turn milestone. That baseline remains a
diagnostic record; the user's later direction now requires gradual speed loss
through this revolution. It may not force retention of a second model and may
not be regenerated without the user's live approval.

Work:

- Retain dense numerical state samples throughout the approved sequence, with
  extra samples around 27 seconds, 28 seconds, and the completion of orbit one.
- Record camera position, velocity, forward, down, planet center distance,
  orbital phase, representative projected grid vertices, and primitive/depth
  pipeline identity.
- Add a headless audit mode that reports or validates these quantities without
  opening a terminal window.
- Preserve the existing input controls and terminal cleanup behavior.

Exit criteria:

- Every protected hash either remains exact or stays reported as failing and
  unchanged until the user approves a necessary single-model difference.
- Pre-planet projection samples and grid morphology do not change.
- The 28-second camera boundary remains position- and orientation-continuous.
- No renderer, projection, or depth-system identity changes at 28 seconds.
- The existing swing and first revolution remain same-direction and preserve
  their approved motion and composition through approximately 34.978 seconds.
- Escape, seeking, and fast-forward pass headless or terminal-safe checks.

### Phase 1: Establish physical units and precision

**Goal:** make 20-metre altitude numerically stable without changing the
accepted visible trajectory.

**Status:** the `f64` precision foundation is complete; trajectory-specific
consumers must be rechecked after consolidation. `make audit-phase1` checks the
centralized 27,000-mile planet scale, the 30,000-foot airplane control,
physical/canonical round trips, and recovery of a positive 20-metre local
offset at several surface orientations after far-origin placement. Recorded
tolerances live in `tests/phase1_precision.txt`.

The accepted capture and first-orbit state, planet-surface vertices, grid
vertices, clipping, and camera-relative projection must all cross one shared
`f64` world-space boundary. Phase 2 consumes that precision in one state law
rather than initializing a continuation at an orbit milestone.

Work:

- Define conversions between miles, metres, planet radii, and canonical render
  units in one place.
- Introduce double-precision planet, camera, velocity, and terrain-placement
  calculations.
- Subtract world positions in double precision before converting local vectors
  to `f32` for projection.
- Keep terminal aspect correction exclusively in projected screen space.

Exit criteria:

- A 20-metre height difference remains positive and stable at every sampled
  point on the planet.
- Round trips between physical and canonical units stay within the documented
  tolerance.
- Accepted state and projection samples remain within documented numerical
  tolerances; any protected hash change follows the Phase 0 approval rule.
- No new renderer or alternate coordinate system is introduced.

### Phase 2: Build the continuous orbit-to-valley trajectory

**Goal:** preserve the accepted approach, smoothly shed incoming velocity
through the fast first orbit, then fly the same camera through decisive braking
and descent to a sustained 30,000-foot
continent pass before reaching the 20-metre valley endpoint. There is no
scene, stop, reversal, or replacement path.

**Status:** implemented with Phase 3P corrections; live review pending.
The fixed 120-Hz world curve yields position, velocity and acceleration at
arbitrary seek times. Capture traverses one spatial curve by arc length;
orbital radius, plane and speed change continuously with phase. Third-orbit
completion is integrated, not snapped to a timestamp. Phase 2 checks
derivatives, seeking, braking, atmosphere by orbit two, the longer third
orbit, 30,000 feet AGL, 18 seconds to low flight and nonzero endpoint motion.

The regional normal uses the canonical planet/world basis, and the same
trajectory continues above displaced terrain into the existing canyon.

Work:

- Replace `flow_relative_position`, `flow_descent_relative`, their boundary
  argument, and the post-orbit selection branch with one
  `trajectory_state(time)` evaluator through the 137-second endpoint.
- Evolve radius, unwrapped angular position, angular velocity, meridional
  steering, and their derivatives from one initial state under smooth global
  control envelopes. Do not reconstruct state at orbit one or any later time.
- Produce two same-direction braking revolutions followed by a regional
  continental track from that evolution; do not retain unused holding turns.
- Steer the track continuously toward the existing continent and valley
  coordinates without reversing orbital direction.
- Tune the continuous radial control so altitude decreases monotonically into
  a near-level 30,000-foot pass and later resumes descending without a plateau
  branch, spline knot, or reset.
- Derive camera forward from path velocity and planet-relative down from the
  surface normal; express artistic pitch and bank only as smooth offsets from
  that frame.
- Add headless orbit-crossing checks based on unwrapped angular phase rather
  than hard-coded guesses about which timestamp represents an orbit.
- Measure surface landmark displacement throughout the airplane pass and add
  upper and lower bounds after visual tuning: motion must remain perceptible
  without preventing terrain inspection.
- Judge trajectory through direct state, derivative, curvature, viewport, and
  deterministic-seek samples. Do not use still rendered frames. Any placeholder
  geometry must consume the same canonical surface source established in Phase
  4; it may not justify a temporary representation.

Exit criteria:

- Position, velocity, acceleration, jerk, forward, and down are outputs of one
  evaluator and remain continuous without a special first-orbit comparison or
  tolerance exception.
- Orbit one preserves its accepted speed and framing.
- Orbit two makes both braking and descent visually obvious.
- Orbit three lasts more than twice orbit two and completes at 30,000 feet
  displaced-surface AGL with nonzero aircraft-scale ground speed.
- Altitude decreases monotonically to the airplane pass, remains approximately
  level during the pass, and later decreases monotonically to 20 metres.
- Forward ground speed remains nonzero through the endpoint.
- The ground-track angular direction never changes sign.
- The camera sees planet, braking descent, continent-dominant airplane pass,
  valley from above, and valley entry in that order using placeholder geometry.
- Source contains no `FLOW_POST_ORBIT_START`, boundary-state reconstruction,
  post-orbit position selection, or separately blended camera frame.
- User approves the live motion before surface-detail work begins.

### Phase 3: Unify the camera and render pipeline across the timeline

**Goal:** make every frame use the same state evaluator, camera, world clock,
projection, clipping, and depth system while preserving the approved approach
behavior.

**Status:** Phase 3A–3C are complete and the Phase 3D production switch was
implemented on 2026-09-06. The one `FlowCamera`, fixed planet, displaced
surface, clipping, projection, and depth pipeline are retained. Production
field placement is camera- and time-independent; the camera crosses ordered
starfield, streak, logarithmic-ring, circular-tunnel, squaring-tunnel, and
squared-tunnel ranges. Automated structural, event, build, and performance
checks pass; live review of the changed field pixels remains the final Phase 3D
acceptance gate.

Work:

#### Phase 3A: Define the anchored field manifold

**Status:** complete in production code. The fixed layout and camera-free
`field_point(layout, s, theta)` pass structural numerical checks and are used
by the renderer.

- Define a fixed orthonormal field frame and a permanent longitudinal
  coordinate `s`; define `theta` or lane coordinates around its cross-section.
- Define `field_point(s, theta)` without time, camera, projected pixels, or
  ring-age inputs.
- Fix the physical tunnel radius, far-field extent, post-planet extent,
  `s_planet`, and the spatial boundaries of the starfield, streak,
  logarithmic-ring, circular-tunnel, and squared-tunnel regions.
- Define circular-to-square deformation, connectivity, color organization, and
  detail as continuous functions of `s`.
- Define the bottom plate as part of the same cross-section and apply the
  localized planetary displacement around the fixed `(s_planet, theta_bottom)`.

#### Phase 3B: Give stars, rings, and mesh stable identities

**Status:** complete in audit-only code. Signed ring ordinals round-trip through
fixed 16-ring chunk identities and map to immutable axial coordinates with a
17:16 geometric spacing ratio on both sides of the planet. Stable ring/lane
edges provide one shared topology and star identities reference those same
edges. A candidate physical streak projects the same fixed point from two
camera poses. None of this code drives the renderer yet.

- Map every star and ring identifier to stable field coordinates. Generate an
  effectively unbounded or sufficiently extended procedural field using stable
  integer chunk/ring identities.
- Replace `ring_state` recycling as a world-position source. Visible-range
  selection may choose fixed rings near the frustum, but selection must not
  change their coordinates or reuse an identity at another location.
- Encode logarithmic ring spacing in fixed axial coordinates so rings converge
  toward the projected horizon and continue through both sides of the planet.
- Derive streak endpoints from the projected displacement of the same world
  particle over a defined exposure interval. If the canonical star motion
  requires actual particle motion, encode it explicitly as a world-space
  trajectory independent of the camera.
- Retain one mesh topology across the circular and squared regions; change its
  cross-section continuously with `s` rather than replacing geometry.

#### Phase 3C: Prove canonical compatibility before rendering it

**Status:** complete in audit-only code. Three-time camera-ray reconstruction
accepts a fixed candidate only when it matches the first and last observations
within 0.0001 pixels, then tests the middle observation. It proves that the
canonical star centres and canonical forming-mesh vertices are not fixed world
points. Before the astronomical rescale, a moving-world witness reproduced the
canonical star centres; at light-year coordinates its tiny local offsets are
explicitly precision-limited, further disqualifying it as production geometry.
Its approach-synchronized motion is evidence of the old co-advected model, not
an acceptable implementation of the anchored field. The exposure audit
also identifies old streak endpoints that cannot be two observations of the
same particle at the required precision. Therefore Phase 3D must preserve the
approved camera and starfield character but necessarily change protected
mesh/streak pixels; those differences remain unblessed until live approval.

- Preserve the locked signed-off star and mesh equations as independent
  audit-only oracles.
- For fixed candidates, intersect or minimize the canonical camera rays across
  multiple timestamps and require one consistent world coordinate per identity.
  Do not accept a visually similar fit.
- Where a fixed identity cannot mathematically reproduce the oracle, determine
  the exact world-space particle trajectory required; do not introduce
  screen-relative correction or camera-following transport.
- Densely compare projected world results with the canonical starfield,
  star-to-ring formation, approach, gravity-well encounter, capture, and first
  orbit. Retain the 0.0001-pixel equivalence target wherever the canonical law
  is representable at that precision.
- Add structural audits proving time-invariant mesh coordinates, fixed region
  boundaries, logarithmic ring spacing, continuous round-to-square shape,
  horizon convergence, extent beyond the planet, and camera-independent field
  placement.

#### Phase 3D: Replace the provisional field atomically

**Status:** structurally implemented, visually rejected, and superseded by
Phase 3E. It successfully removed field transport and ring-age placement, but
its star coordinates still converged toward mesh vertices and its inverse
star/ring visibility weights constituted a disguised handoff.

#### Phase 3E: Populate the fixed universe without a handoff

**Status:** structurally implemented, then superseded by Phase 3F. It removed
camera-driven placement, but stars still borrowed `FieldRingIdentity`, the
renderer retained a hard star/ring spatial cutoff, and rail emergence was
still encoded as opacity.

**Architecture correction — universe catalogue:** the field is no longer
constructed from the current camera trajectory. Its `f64` basis, origin,
extent, and physical coordinates are fixed universe constants. The current
flight's crossings at 4, 11, 20.063, and 21 seconds are audit observations,
not world-construction inputs. Replacing the camera path must not alter any
star, ring, rail, planet, atmosphere, or sun identity.

The production field consists of stable physical object identities:

- `StarObjectIdentity(cell, object)` resolves to one fixed point independent of
  every mesh lane and vertex.
- `RingObjectIdentity(ring)` resolves to one permanent closed loop, tessellated
  only for projection.
- `RailObjectIdentity(first_ring, lane)` resolves to one permanent segment
  joining the exact shared vertices of adjacent rings.

There are more than twelve thousand fixed stars, six thousand fixed rails, and
ninety fixed rings. Stars occupy their own permanent 3D volume. Rail
connectivity and the
circular-to-square deformation remain fixed spatial properties of the mesh.
Production rendering has no `FieldRegion` branch, density switch,
particle-placement switch, inverse star/ring visibility partition, or
particle-to-mesh handoff. The old region names remain only as audit labels
describing what the current camera happens to see.

The camera-dependent procedural star backdrop has been removed. Empty space
is a camera-independent vacuum color, and every visible star is the projection
of a physical star object. Streaks are projections of the same fixed star from
two poses of whichever camera path is supplied; neither world geometry nor
star material reads the current flight's milestones.

The opening camera remains four light-years upstream of the ring/tunnel
entrance, whose physical radius projects below 0.0001 pixel there. The approved
0–4-second camera law remains source-locked and joins the subsequent canonical
trajectory with continuous position, velocity, and acceleration. This lock
applies to the camera law only: the former procedural background, layered star
placement, supplemental population, post-4 speed support, and renderer-driven
blur taper were rejected and are superseded by the universe catalogue above.

Mesh rendering has a generic physical-distance envelope: it is absent beyond
26 world units and fades to full opacity by 18, without reading demo time. The
fixed stellar volume widens at astronomical distance and narrows toward its
physical endpoint. Each star has one independent hashed axial and transverse
coordinate. No star reads `field_point`, approaches a mesh vertex, changes
position, or changes visibility because the camera entered another named area.

Motion blur is purely observational. The application samples the chosen camera
path at the beginning and end of a 0.032-second exposure and passes both poses
to the field renderer. The renderer neither accepts demo time nor calls the
camera-path function. Each streak connects projections of one fixed star from
those two poses, with no timeline, region, or organization multiplier applied
to its length. A replacement flight path therefore requires only replacement
camera poses; it does not reconstruct, transport, or reclassify world objects.

Direct audits require zero premature mesh visibility at 0 and 4 seconds, at
least 320 visible opening stars across 24 screen bins, at least 64 visible
motion-blurred stars, a four-light-year opening separation, a subpixel
uncropped grid, superluminal forward progress, an exact C2 camera join, and a
production renderer with no demo-time or camera-path dependency.

- Switch stars, streaks, rings, and mesh together to the proven anchored field;
  do not run two coordinate models or crossfade between them.
- Delete `flow_field_transport(time)`, time-derived mesh placement,
  camera-dependent field reconstruction, and obsolete finite-window ownership
  from production. Rejected equations may remain only as feature-gated,
  independently named audit oracles.
- Keep the established `FlowCamera`, fixed planet, displaced surface,
  camera-relative `f64` subtraction, clipping, projection, and depth convention.
- Re-run Phase 0–4 source and numerical gates, then stop for the user's live
  review before Phase 5 terrain art or Phase 6 atmospheric concealment.

Exit criteria:

- No rendered element at any time reads the legacy camera or old 1,000-unit
  planet position.
- The same field identity returns bit-identical world coordinates at every
  sampled time; geometry evaluation has no camera or demo-time parameter.
- Stars, streaks, logarithmic rings, circular tunnel, squared tunnel, planetary
  well, and post-planet mesh occupy ordered, continuous ranges of one `s` axis.
- Ring spacing is fixed and logarithmic, the tunnel is cylindrical in world
  space, and only perspective makes it appear conical.
- The field spans from the projected horizon through and beyond the fixed
  planet, and it does not follow the camera into orbit.
- Protected-frame changes remain reported and unblessed until the user accepts
  any necessary unified-model difference in live playback.
- The 28-second frame uses the same projection, clipper, and depth path as its
  neighbors; only continuous camera and geometry parameters change.
- Seeking to any demo time produces the same camera state as continuous
  playback.
- The sun, planet, and grid do not jump at the former camera boundary.
- Seeking and continuous playback return bit-identical state and geometry at
  the same timestamp without history or boundary initialization.
- `flow_field_transport`, ring-age world placement, screen-relative field
  placement, and camera-derived mesh coordinates are absent from rendered code.
- The build has no newly suppressed dead-code or unused-item warnings.

#### Phase 3F: Separate spatial catalogues and overlap them physically

**Status:** implemented; awaiting the user's live review of 0–28 seconds. The
camera law and all world coordinates outside catalogue organization are
unchanged.

Stars now use a dedicated unsigned `StarCellIdentity` with a 33/32 logarithmic
spacing law and fixed hashed positions. Star source and identity code cannot
reference `FieldRingIdentity`, ring spacing, mesh vertices, lanes, or samples.
The original star volume extends eight world units into the fixed ring volume.
The renderer independently queries both catalogues there; it has no `s`
boundary, region branch, inverse weight, or crossfade selecting one
representation. Phase 3M retains every original point and extends the star
catalogue forward without moving this original volume.

Every rail lane has a permanent hashed starting-ring identity in the physical
formation volume. `rail_object_exists` is topology, not an opacity curve, and
all existing rail objects render with the same physical distance visibility as
the rings they join. The circular-to-square deformation remains a property of
`field_point(s, theta)` and is not involved in star placement.

The production numerical audit samples every 0.5 seconds from 0 through 28. It
requires at least 32 visible fixed field objects, eight occupied screen bins,
and nonzero projected motion after the opening instant at every sample. It also
requires at least 64 actual stars in the original eight-unit star/ring overlap
and at least four distinct permanent rail starting rings. This supplements the key
event checks rather than treating those timestamps as implementation segments.

The fixed logarithmic-ring distance table is evaluated at compile time using
the prior iterative addition and multiplication order. It therefore changes no
ring coordinate while removing the repeated per-vertex distance calculation
that had broken the opening/orbit-entry frame budget. The original bloom
raster remains unchanged.

#### Phase 3G: Remove the 21-second trajectory reversal

**Status:** superseded by the Phase 3H retiming below.

The former endpoint-constrained quintic reduced axial speed to zero at about
21.916 seconds, moved the camera backward, and then accelerated forward again.
The replacement is a monotone degree-18 polynomial progress density evaluated
continuously from 11 seconds through orbit entry. It preserves the position,
velocity, and acceleration at the 11-second join and the accepted orbit-entry
state at 28 seconds. Its axial speed remains positive and at least 2.5 world
units per second at every 120 Hz audit sample.

Because the camera now traverses the world monotonically, the fixed rail and
square-region coordinates were moved to the new physical locations crossed at
their established observation times. No renderer offset, time-selected scene,
or object transport was added. The circular-tunnel visibility audit observes
the physical ring one second before its boundary crossing, when its projected
radius fits the viewport, rather than requiring it to fit at the exact crossing
instant.

#### Phase 3H: Expose the first ring, retain streak exits, and accelerate the approach head

**Status:** historical; speed superseded by Phase 3J/3T. Streak-exit clipping
is retained for the opening wormhole rush only; it is not a requirement to
keep streaking stars in the post-wormhole ring/circular grid.

The initial four seconds and its exact position, velocity, and acceleration at
the jump completion remain unchanged. The continuous monotone approach law now
runs from that 4-second state to the same accepted orbit-entry state at 28
seconds. Its degree-30 progress density increases the early and middle
ring-field traversal speed without reducing the orbit-entry tail speed. The
120 Hz audit requires strictly forward axial progress, a minimum 3.3 world
units per second throughout 4–28 seconds, and exact endpoint position,
velocity, and zero acceleration at both protected states.

The rings remain permanent objects at fixed `s` coordinates; they were not
moved onto the camera or made time-dependent. The physical mesh visibility
envelope now fades over 28–52 world units. The field is still outside that
envelope during the astronomical rush, but several nested rings are already
inside it when `starflight_displacement` reaches its exact zero state. At that
jump-exit state the audit requires at least four visible ring identities, three
with substantial visibility, 128 visible circumferential edges, and six
visible connecting rails. This verifies a readable distant structure rather
than the technical projection of one later ring fragment.

Each streak remains one fixed star projected at the two shutter endpoint poses.
When only part of that projective shutter line lies in front of the near plane
or inside the viewport, homogeneous half-space clipping retains exactly that
visible interval. Star-cell culling admits a cell visible at either exposure
endpoint. The numerical audit finds real catalogue stars with one endpoint
behind the camera and requires at least eight finite, screen-intersecting
partial exposures. There is no time-selected streak module, visibility handoff,
or replacement motion law.

#### Phase 3I: Extend the square tunnel, steer off-axis, and separate exterior stars

**Status:** implemented; awaiting live review of 20–42 seconds.

The planet and accepted orbit-entry state remain at 28 seconds. The field now
reaches its fully square cross-section at the exact fixed `s` coordinate crossed
at 26 seconds, leaving two complete seconds of flight through the fully squared
tunnel before capture. This changes the fixed field shape, not the camera clock,
and the square remains fully resolved through the planetary well and the mesh
beyond it.

From 20.5 seconds to orbit entry, the single trajectory evaluator adds a compact
C2 right-left maneuver and a smaller vertical maneuver. Both are analytic sums
of smootherstep states and return to exactly zero position, velocity, and
acceleration at 28 seconds. The camera follows the resulting velocity through
24 seconds and maintains a continuously orthonormal frame; Phase 3K then adds
the smooth physical-planet look offset. The motion is steering through the grid
rather than screen-space sliding. The original approach before 20.5 seconds
and the accepted capture position state are unchanged.

The dense near-star catalogue now has a fixed spatial extinction range across
the eight-world-unit star/ring overlap and is exactly absent by orbit entry; it
cannot reappear as a block when the orbit looks back toward the approach. A
second catalogue contains 512 sparse stars at fixed world positions 64–192
world units from the planet. Their finite, varied depths create real camera
parallax. Their opacity is derived from the camera's geometric relation to the
horizontal grid enclosure: zero while inside it and smoothly visible outside
it. Atmospheric extinction still removes them at 30,000 feet. Neither catalogue
uses a scene timestamp, camera-relative star position, or renderer handoff.

#### Phase 3J: Decelerate continuously while preserving every field event

**Status:** approved opening law retained through 20.5 seconds; the former
20.5–28 off-axis maneuver was superseded by Phase 3L.

At the exact end of the astronomical jump, the approach begins at 18 world
units per second. Its analytic C2 speed law remains protected: the active
centerline decelerates continuously to 13.25822914011477 world units per second
at 28 seconds. Exact world position, velocity, acceleration, and field-basis
camera are retained through 20.5 seconds. Phase 3L intentionally replaces the
former 20.5–28 world-position construction while preserving that speed law.

The greater distance travelled is represented by a longer fixed universe, not
by shortening the observations. Fixed `s` coordinates exactly equal the camera
coordinates through the established 4-second jump exit, 11-second full-ring
event, and mesh-maturity event. Because the replacement capture geometry begins
at 20.5 seconds, the 21-second square-transition and 26-second fully-square
timestamps are observations within one world unit of those unchanged physical
boundaries, not exact coordinate locks. The physical ring catalogue begins 64 world units beyond the
jump-exit coordinate so several rings are already visible on arrival, and the
distance-based mesh envelope fades from 80 to 180 world units. Thus the same
event times are reached at the higher approach speed while every ring and rail
retains a permanent world location.

The former pair of timed off-axis excursions is no longer active. Phase 3L
uses the single capture centerline for all transverse motion after 20.5
seconds. The source gate continues to lock the accepted opening and speed law.

#### Phase 3K: Historical rejected capture-bank pass

**Status:** rejected and no longer active. The timed look, roll, impact pulse,
and frame morph described below are retained only as historical rationale for
Phase 3L; source and audit gates now reject them in the active trajectory and
camera.

The planet-facing attitude is now one long C2 curve from 24 through 35 seconds.
It begins almost imperceptibly while the camera is still inside the square
tunnel, so keeping the planet in view no longer requires a late corrective
pitch. Its later downward framing is reached by another C2 blend extending
through 42 seconds. The explicit roll tangent rises with zero endpoint angular
velocity and acceleration from 27 to 31 seconds, remains steady through 35
seconds, and releases with the same C2 continuity through 42 seconds.

The bank is evaluated only in `flow_camera`; neither `trajectory_state` nor
`trajectory_acceleration` can read it. The physical lateral capture velocity
rises smoothly from 28 seconds to the far-side interval at 31.5 seconds, then
releases smoothly through 42 seconds rather than stopping at completion of
orbit one. The camera frame moves from the projected
fixed-field right vector to the physical planet-orbital right vector
`cross(toward_planet, forward)` over 29–32 seconds, before the fixed-reference
projection can approach a singular orientation. There is no normalized
down-vector cancellation or discontinuous fallback.

The braking control is spatial: its value is exactly zero until orbital phase
512, the geometric far side of orbit one, and reaches one only at phase 2048,
completion of orbit two. The 120-Hz numerical gate requires settled cruise
speed to remain within 98–110 percent through the far side, at least 70 percent
of far-side speed to remain at orbit-one completion, and no more than 20
percent to remain at orbit-two completion. It also rejects any per-frame speed
step above two percent.

The camera gate keeps the planet in the viewport throughout 28–35 seconds,
proves the roll value and its first two derivatives exactly at all four support
boundaries, and rejects frame jumps. Forward and down chord rates are bounded
at 1.5 and 2.5 per second respectively. The 100-ms acceleration check measures
only the tangential change in angular rate, excluding the ordinary centripetal
second derivative of a frame rotating smoothly at constant orbital speed; that
angular acceleration stays below 3.0 per second squared through the complete
bank release.

The planet flicker came from two raster passes sharing depth during the
unresolved-to-polygon transition. Complementary opacity did not guarantee
complementary coverage: the exact subpixel pass could write depth and reject
the polygon pass behind it. That transfer is deleted. One bounded polygon
lattice now owns the surface at every apparent size, while mixed boundary
cells continue to use exact pixel-centre rays against the same displaced
sphere. Conservative projected bounds only cull impossible cells and expand
nestedly as the planet grows; there is no opacity crossfade or second surface
pass.

#### Phase 3L: One geometric capture centerline

**Status:** implemented numerically; awaiting user live review.

The approved trajectory and fixed field-basis camera are evaluated exactly
through 20.5 seconds. Their complete world position, velocity, and normal
acceleration seed one C2 world curve: one quintic whose endpoint position,
tangent, and curvature meet the upstream intersection of the radius-15 orbit
while the complete curve remains in the squared tunnel. Its parameter has no
demo-time input. Arc distance advances along that geometry using the approved speed law
through 28 seconds and 13.25822914011477 thereafter. The unused fraction of the
integration substep at the circle join is carried into the orbit, so the join
cannot discard distance or create a one-frame slowdown.

The curve reaches a planet-centred orbit of radius 15 world units from above.
Its endpoint tangent and curvature equal the circular orbit tangent and
centripetal curvature. Orbit radius and speed then evolve from unwrapped
orbital phase: full speed through phase 512, the main brake through phase 2048,
and the 30,000-foot target at phase 3072 and 58.325 seconds. Camera attitude is an
orthonormal frame derived from the path tangent, planet direction, physical
distance, and the fixed field lateral axis projected onto the viewing plane.
There is no active timestamp bank, impact pulse, projected framing
feedback, or old-orbit orientation seed.

Numerical gates check the exact protected opening, curve join, monotone phase,
position derivatives, per-frame speed changes, camera-frame rates, planet
visibility, event-space continuity, deterministic seeking, and the fixed-world
field. Direct Phase 0–4 and isolated Phase 7 gates pass. Phase 5 terrain detail
and Phase 6 sun composition remain the next work only after live acceptance.

#### Phase 3M: Stabilize cylindrical rails and carry stars into the square grid

**Status:** implemented numerically; awaiting user live review.

Rail segments now use a closed near-plane projection rule. If one physical
ring endpoint is behind the camera and its adjacent endpoint is in front, the
shared rail is clipped to the near plane and remains drawable; the clipped
point is not immediately rejected by a second strict comparison. A dense
audit over the circular-tunnel interval exercises actual rail objects on both
sides of the plane and requires every partial segment to project finitely.

The original 569 star cells and all of their world positions are unchanged.
Another 113 fixed cells extend the same catalogue forward through the
circular tunnel. These are POINT stars after wormhole emergence, not a second
streak population (2026-09-12 correction). Star visibility remains full at the start of the colored
square deformation, then fades continuously across its first eight world
units. The extension is spatial catalogue geometry, not a timed scene or a
camera-relative replacement.

#### Phase 3N: Keep capture inside the squared approach

**Status: rejected; superseded by Phase 3O.** This constrained the approach
polynomial but left the vertical radius-15 orbit free to cross the roof. Its
58.325-second table-tick completion also fails the current orbital cadence.
Do not restore its claimed acceptance.

#### Phase 3O: Whole-orbit containment, horizon, well and descent cadence

**Status: rejected.** The radius-15 horizontal orbit was still far outside
the tunnel. Its below-floor exception incorrectly bypassed lateral limits.
Do not restore that curve or its permissive tests.

#### Phase 3P: Contained fast over-and-under maneuver

**Status: implemented; numerical verification and live review gate above.**

- The user accepts a roughly 1.2-second first orbit. Preserve the incoming
  speed law and brake on the back half of orbit one and through orbit two.
- Radius 2.4 fits the unchanged 2.53-unit half-width. No lateral exits, no roof
  exits, and no below-floor travel outside the localized planet region.
- Use one monotone axial graph to enter the over/under orbit at 20 degrees.
  Match position, tangent, curvature and third derivative. No independently
  timed lateral/downward moves, counter-turns or overshooting control polygon.
- Keep the two fast revolutions in one plane; reorient gradually toward the
  regional track during the slower third orbit.
- Treat large continuous angular rates separately from impulses. Scale the
  rate bounds to the authorized fast orbit; check acceleration and jerk at
  every interpolation knot and midpoint, plus monotonic capture curvature.
- Keep atmosphere by orbit two, aircraft altitude by orbit three, and
  15–20 seconds from there to low canyon flight.
- The strict volume regression remains unchanged by the new implementation.
  `make audit-flight` runs it first; no saved images or visual hashes approve
  the result.

### Phase 4: Establish one efficient depth-tested polygon pipeline

**Goal:** replace both the screen-space intro planet and the full-screen
ray-surface prototype with one bounded polygon surface that works from first
sighting through the valley.

**Status:** completed and stabilized through Phase 3L. The screen-derived proxy,
independently sampled wire globe, separate wire-depth path, and dual
subpixel/polygon ownership are deleted. One bounded lattice calls the canonical
displaced-surface sampler at every scale, with exact rays in boundary cells;
the artifact-producing mesh-edge accents are removed; conservative bounds use
the maximum displaced radius; and camera AGL is measured against the same
relief source. The direct audit is numerical and does not render a frame.

Retain the proven bounded lattice, camera-relative `f64` subtraction, compact
depth buffer, canonical near-plane clipping, exact boundary-cell coverage, and
displaced-terrain sampling where they can consume the single surface source.
Projected bounds, tiny coverage, polygon vertices, optional accents, grid
occlusion, atmosphere limits, and camera AGL must all derive from that source.
No independently sampled ellipse, latitude/longitude sphere, screen lift, or
second depth convention may survive for the sake of matching old pixels.

Work:

- Express the existing terrain source as displacement of the canonical sphere.
- Use the same surface-normal coordinates for ocean, continents, mountains,
  valley, clouds, and landing target.
- Build a fixed-capacity sphere/terrain vertex and triangle pipeline using the
  shared camera, near-plane clipping, culling, projection, and compact depth
  buffer.
- Draw polygon fill and optional wire accents from the same vertices and
  triangles; do not maintain a second wire globe or ray surface.
- Delete `grid_planet_view` and `projected_planet_view` as geometric sources.
  Replace them with bounds projected from the canonical sphere/surface.
- Delete `render_wire_planet`. If an early globe guide remains desirable, derive
  it from persistent isolines or edges on the canonical surface mesh and pass
  it through the same clipping and depth ownership.
- Make unresolved coverage an adaptive raster strategy over the same canonical
  surface hits, not a separately positioned or separately bounded planet.
- Cull back-facing, off-frustum, subpixel, and fully occluded patches before
  rasterization where correctness permits.
- Make the planet physically occlude rear grid lines while preserving grid
  lines that are genuinely in front.
- Instrument computation time and fixed mesh bounds at representative timeline
  checkpoints without producing or inspecting still images. Per-frame
  primitive counters remain useful follow-up instrumentation.
- Continue calculating final altitude above displaced terrain rather than the
  nominal sphere.

Exit criteria:

- No time branch replaces a screen ellipse, wire sphere, ray sphere, or plane
  with another surface renderer.
- A sampled terrain location has the same world position at orbital, regional,
  and valley distances.
- The planet remains round at every tested terminal aspect ratio.
- Grid visibility is fully explained by camera depth and planet occlusion.
- The final camera is 20 metres above the displaced surface.
- Protected-hash changes remain reported and unblessed pending live user
  approval; the 60-fps target is met where feasible, and no checkpoint exceeds
  the 33.3-ms hard maximum at 320x200.
- `git diff --check`, `make check`, and the Phase 0 through Phase 4 audits pass
  without warnings.

### Phase 5: Build the colorful fractal planet-to-valley surface

**Goal:** make the one polygon surface resolve into an interesting,
*Populous*-like colored world from planet to continent, landscape, mountains,
and valley without popping or appearing midway.

#### Authorized implementation pass: 5A–5D

1. **5A — Persistent non-repeating geography.** Replace the active repeated
   256-cell terrain motif with nonperiodic world-coordinate landforms. Coarse
   basins and massifs own their smaller ridge/residual bands. Use spherical
   coordinates for planetary identity and an unwrapped local chart for the
   existing destination watershed. Weight each wavelength by its actual
   projected footprint, not a timed scene switch. Retain continent ownership
   and the existing navigable floor; verify the exact flight fingerprint.
2. **5B — Drainage into the canyon.** Add tributary cuts and cliff structure
   as persistent residuals of the same watershed. Keep the floor's original
   position and elevation. Correct the shared surface-intersection search:
   a ray starting inside the wall-height shell must still find the nearer
   wall, rather than falling through to a reference floor sphere. Keep the
   122-second wall witness and its minimum sixteen owned pixels per side;
   add representative descent/low-flight samples and intersection residuals.
3. **5C — Readable surface materials.** Derive relief shading, elevation
   tiers, drainage/water colors and settlement detail from the same persistent
   terrain samples. Add footprint filtering to fine city blocks and terrain
   detail. No rotating texture, tiled continent, screen-space relief or
   replacement terrain renderer. Favor clear low-poly geography over realism.
4. **5D — Evidence and performance.** Test non-repetition at the old tile
   period, parent/child reconstruction, fixed landmark coordinates, resolved
   height/color variation during the 40–42-second pass, sky/surface coverage,
   visible canyon depth and temporal stability. Keep the 4-pixel polygon
   lattice and bounded per-vertex work. Verify exact flight-state identity,
   all earlier numerical gates and the 33.3-ms frame ceiling. Live aesthetic
   approval remains pending; these tests do not approve braking or appearance.

The previous dependency requiring motion approval first is deliberately
deferred for this pass. Phase 6 may proceed after Phase 5's numerical gates;
do not use atmosphere to conceal remaining terrain or motion defects.

**Status:** 5A–5C implemented in the authorized combined pass; 5D remains open
on framing, the 53-second compute budget and live appearance. The active
geography is no longer the old repeating 256-cell map. Domain-warped 2048/320
mile continental/basin terms and rotated 64/16/4/1/0.25-mile ridge bands are
filtered by projected footprint. The legacy 64/16/4/1 hierarchy survives only
inside the permanent river-floor support to preserve exact flight geometry.
Surface materials add drainage, biome/elevation tiers, triangle-normal sun
lighting and footprint-filtered settlements. Clouds are separate persistent
shell geometry, not painted into terrain. Audits check non-repetition,
parent/child reconstruction, fixed landmarks, subpixel geomorphing, high-pass
height/material variation and destination-valley depth ownership.
The overhead visibility gate now renders the surface, reads the actual depth
at the projected destination, reconstructs the owning surface normal, and
rejects nearer terrain or planetary-curvature occlusion. Merely projecting the
destination in front of the camera is not considered visibility. The camera's
production spherical-map coordinates are also sampled every quarter-second
from the overhead pass through the endpoint: the track must stay inside the
same generated valley excavation, approach its centreline at the named
controls, and finish on that centreline.
The inherited 88-mile excavation now serves only as the regional parent. A
persistent world-space funnel follows the same ground track and narrows into a
1.5-mile wall-to-wall valley at the final scale, with a half-mile floor and
terrain walls that rise by more than 0.1 mile on both sides. This detail is a
continuous residual of the same height function, not a late terrain object.
At 122 seconds the rendered depth buffer is reconstructed into persistent
spherical-map coordinates. Each half of the low-flight frame must contain at
least sixteen depth-owned samples from the nested walls; testing their source
height alone is insufficient.
The shared sampler brackets displaced-surface intersections inside a bounded
relief shell, including rays that miss the reference floor sphere but hit a
wall. Safeguarded secant/bisection refinement checks the actual surface
residual. Quarter-second track checks remain, and depth-owned walls are now
tested at 58, 90, 122 and 137 seconds without changing the old minimum of
sixteen pixels per side. Exact memoization preserves sample values, but the
boundary fallback still exceeds the compute budget at the descent checkpoint;
5D cannot be marked complete until that is resolved.

Work:

- Generate or cache a deterministic multiresolution fractal height and biome
  hierarchy once, in persistent spherical coordinates.
- The overhead continent may not be a tilted, repeated patch or a coarse motif
  that merely scales up. Its low-frequency height field must first read as a
  varied continental landscape: distinct massifs, broad basins, watersheds,
  escarpments, and drainage trunks with asymmetric placement and scale.
- Use coarse parents for oceans, continent silhouettes, and major relief;
  child residuals add ranges, mountain groups, drainage, and the destination
  valley without changing parent identity.
- Make the canyon system the resolved continuation of those drainage trunks.
  Successively lower-altitude bands add tributaries, gullies, cliffs, and wall
  structure inside the same basins, so the eventual flight corridor can be
  traced back to a feature already visible in the airplane-height landscape.
- Sample visible polygon patches on the stable screen lattice; their
  world-space footprint contracts continuously during approach while the
  triangle budget remains fixed.
- Geomorph child vertices from their parent surface positions.
- Band-limit fractal terrain octaves so each wavelength becomes visible only
  when resolvable.
- Preserve continent boundaries and large mountain chains as parents of later
  regional and valley detail.
- Reject obvious periodic repetition at the initial continent view. Repeated
  noise may supply residual roughness, but domain warping, octave rotation,
  basin masks, and nonperiodic feature placement must keep the large readable
  forms from looking like a tiled height map.
- Apply readable flat or restrained Gouraud color from persistent elevation,
  slope, water, biome, and sun values. Prefer strong stylized geography over
  expensive realism.
- Reuse the same mesh data for fills, wire accents, depth, shadows, atmosphere
  inputs, and camera AGL calculations.
- Ensure the destination valley is visible from above before the camera enters
  it.
- Treat the 30,000-foot pass as a primary art deliverable: frame most of the
  continent below the camera and expose nested coast/drainage, biome, range,
  mountain, and local-relief structure at the appropriate footprints.
- Track representative continental landmarks for several seconds during the
  pass and verify that their shapes remain attached and readable while moving.

Exit criteria:

- No LOD change moves an existing silhouette or terrain landmark by more than
  one pixel in a frame.
- Continental features remain spatially attached as they become regional
  terrain.
- Mountain ranges are visible before individual mountains dominate the view.
- The valley is identifiable from above before its walls surround the camera.
- The world is colorful and geographically interesting at every named scale;
  it does not read as a generic noise sphere or a flat height field.
- At 30,000 feet, terrain owns roughly 70–85 percent of the frame and shows
  multiple nested fractal scales without temporal popping or blurred noise.
- Landmark motion across the airplane pass is plainly perceptible but slow
  enough for a feature to remain recognizable over multiple seconds.
- No full-screen surface ray tracing or repeated per-pixel fractal evaluation
  remains in the rendered path.
- The 60-fps target is met where feasible, and no checkpoint exceeds the
  33.3-ms hard maximum at 320x200.
- User approves the scale progression before atmospheric effects are allowed
  to obscure it.

### Phase 6: Integrate atmosphere, plasma, clouds, and lighting

**Goal:** add entry effects on top of the continuous geometry without hiding a
motion or LOD defect and without breaking the frame budget.

#### Authorized implementation pass: 6A–6D

1. **6A — One fixed sun.** Select a permanent world-space direction that is
   above the regional horizon and within the existing high-pass/valley view.
   Do not steer the camera, animate the sun into view or pin it to the screen.
   Use that same direction for terrain illumination, day/night settlements,
   atmospheric tint and the depth-occluded sun disk. Preserve foreground depth.
2. **6B — Depth-correct atmosphere.** Integrate shell density only over the
   visible ray interval: to grid/surface depth, or the shell exit for sky.
   Short foreground rays must not inherit optical depth from the far side of
   the planet. Retain complete grid/star extinction by 30,000 feet, but keep
   nearby terrain legible. Test interval additivity/monotonicity, short-ray
   haze, the entry-density window and the actual 40–42-second hold.
3. **6C — Anchored clouds and entry effects.** Give clouds persistent
   spherical coordinates and restrained contrast/coverage. Filter unresolved
   cloud detail and city lights. Keep entry plasma driven by actual speed and
   density, with zero heat for a stopped camera and no independent event clock.
   Sky color comes from world-ray elevation and the fixed light, not screen
   row or a full-screen altitude wash.
4. **6D — Integrated verification.** Require exposed, depth-correct sun at
   high-pass, descent and low-flight checkpoints; unchanged geometric depth
   with atmospheric shading; no background-field depth at airplane altitude;
   deterministic seeking and bounded combined frame time. Add Phase 6 to the
   ordinary numerical flight audit, not a separately skipped failing gate.
   Report unresolved results explicitly and leave visual acceptance to the user.

**Status:** 6A–6C implemented; the physical/depth/lighting gates pass, but 6D
remains open on combined compute time and user review. The shared 100-km shell
uses eight-point quadrature of exponential density on the actual visible ray
interval. Solid-core/surface/grid depth clips that interval. Tests exercise
10-metre foreground haze, integral additivity, altitude-driven field
extinction, unchanged depth ownership and world-oriented sky color. The real
grid-depth witness is selected from the current physical entry-density window,
not an obsolete timestamp. Clouds occupy a fixed 2-km shell, with nonperiodic
coordinates and footprint filtering; cloud opacity remains at most 0.12.
Plasma retains actual speed/density and ground-track inputs.

The sun retains one constant world direction, shared by illumination and its
infinite-depth projected disk. All eight exposure witnesses pass at 40, 41,
42, 53, 55, 90, 122 and 137 seconds. It is not steered by camera or clock.
The expanded integrated frame-time audit covers physical entry, 32, 34, 38,
40–42, 53, 58, 65, 78, 90, 106, 122 and 137 seconds. The 53-second surface
cost fails the unchanged limit; the aggregate numerical audit now includes
this failure instead of omitting Phase 6.

Work:

- Model the atmosphere as a shell around the same sphere.
- Compute optical depth from ray length and altitude.
- Attenuate grid and surface light physically along those rays.
- Drive plasma intensity and shape from atmospheric density and actual flight
  speed.
- Anchor clouds in the same spherical coordinates as terrain.
- Use the one world-space sun direction through orbit, atmosphere, and valley.
- Bound atmospheric work by visible tiles or depth-buffer spans; do not add a
  second unconditional full-frame high-cost pass.
- Calibrate shell extinction so grid and star transmission through sky pixels
  is below one display-code value by 30,000 feet while the fixed sun remains
  visible through atmospheric tint.

Exit criteria:

- Disabling atmospheric shading reveals the identical underlying geometry and
  trajectory.
- Plasma has no independently animated position or scene-timed wipe.
- Grid and star loss during entry corresponds to measured optical depth, with
  both effectively absent from sky pixels throughout the 30,000-foot pass.
- Clouds and surface landmarks retain their relative world positions.
- The sun crosses no camera or coordinate-system discontinuity.
- The combined geometry, depth, and atmosphere frame stays below the 33.3-ms
  hard maximum at every 320x200 checkpoint.

### Phase 7: Remove legacy paths and harden the complete sequence

**Goal:** leave one understandable implementation whose structure enforces the
continuous-world contract.

**Status:** reopened. The previous Phase 7 result checked one camera and shared
projection plumbing but did not reject the post-orbit path change,
geometry-time offset, screen-derived planet proxy, separate wire sphere, or
dual surface ownership. Re-run hardening only after the single-model gate and
Phases 0–6 pass under the revised architecture.

Work:

- Remove superseded camera, planet, terrain, masking, and visibility code.
- Remove `flow_field_transport`, ring-age world placement, recycled world-ring
  identities, and any field function that reads camera state or demo time.
- Remove every boundary-state constructor, post-timestamp trajectory formula,
  screen-derived planet proxy, compatibility geometry clock, and independently
  sampled globe representation named by the consolidation gate.
- Consolidate duplicate scale and timing constants.
- Profile the stable surface lattice, depth buffer, and atmospheric pass.
- Tune detail and lighting without changing trajectory equations.
- Remove the rejected full-screen displaced-surface ray-intersection path and
  every time-selected renderer branch.
- Recheck all controls, terminal cleanup, numerical state determinism,
  computation time, executable size, and warning policy without producing or
  inspecting still imagery.

Exit criteria:

- One trajectory-state evaluator, camera, projection, depth convention,
  canonical field manifold, canonical world clock, and planetary surface owner
  remain in rendered code.
- No source comment or document describes a removed path swap, boundary path,
  compatibility clock, or alternate planet representation as active.
- All global acceptance checks below pass.
- User approves the uninterrupted full sequence.
- `git diff --check`, `make check`, and `make size` complete without warnings.

## Acceptance checks

The orbit-to-valley work is not complete until all of these pass:

- The capture swing has no velocity impulse when the camera-only bank begins;
  bank angular velocity and acceleration are zero at its boundaries, cruise
  holds through the far side of orbit one, and braking then completes smoothly
  across orbit two.
- Orbit two visibly and continuously reduces both altitude and speed.
- Orbit two finishes inside the 100-km atmosphere. Orbit three lasts more
  than twice as long, finishing at 30,000 feet AGL and aircraft ground speed.
- Entry stays below 30 degrees; the approach and all three orbits stay below
  the roof. The actual floor has a pronounced well.
- Settlements have daytime materials and night lights. Reach 20 metres AGL
  15–20 seconds after starting orbit four.
- The endpoint is 20 metres above the displaced terrain, not the nominal
  sphere.
- Displaced-surface AGL decreases monotonically from orbit one to orbit three,
  remains approximately level through the high pass, then decreases
  monotonically to 20 metres.
- Position, velocity, acceleration, and bounded jerk come from one state
  evaluator and remain continuous everywhere; named milestones are sampled
  observations rather than joins.
- Forward, down, and roll remain continuous and finite.
- Ground-track angular motion never changes sign.
- Protected hashes are never silently regenerated. Any mismatch caused by
  consolidation stays reported until the user approves the unified live result.
- Camera, projection, clipping, and depth-system identity stay constant across
  the entire timeline; 28 seconds is not a renderer boundary.
- Planet radius in physical screen pixels is aspect-independent within one
  raster pixel.
- No renderer uses `journey_camera` or the old 1,000-unit planet.
- No rendered path contains `FLOW_POST_ORBIT_START`, a reconstructed
  first-orbit boundary state, `geometry_time`, compatibility `world_shift`,
  screen-derived planet geometry, or an independently sampled wire globe.
- No mesh world-position function accepts demo time, ring age, projected depth,
  or camera state. No finite ring identity is recycled into multiple world
  locations.
- A stable field coordinate orders the starfield, streaks, logarithmic rings,
  circular tunnel, squared tunnel, fixed planetary well, and mesh beyond the
  planet without gaps or renderer boundaries.
- No time branch swaps screen-space planet, ray surface, wire globe, polygon
  sphere, or planar terrain renderers.
- The continent is identifiable before regional terrain, the destination
  valley is visible from above before entry, and mountain walls surround the
  camera only after that overhead approach.
- During the 30,000-foot pass, terrain occupies roughly 70–85 percent of the
  frame and persistent fractal landmarks remain recognizable over multiple
  seconds of clearly perceptible motion.
- Grid and star disappearance can be explained by depth or atmospheric optical
  depth at every sampled pixel; their sky transmission is below one display
  code value by 30,000 feet.
- The persistent world-space sun remains visible in the atmospheric sky during
  the airplane pass, subject only to physical foreground occlusion.
- Planet, terrain, fill, any line accents, atmosphere depth limits, and camera
  AGL calculations share the same canonical surface positions, normals, and
  displacement source.
- 320x200 render computation targets 16.7 ms and never exceeds 33.3 ms at any
  required timeline checkpoint, excluding one-time generation and terminal
  transmission; timing does not produce still-image evidence.
- Escape, seeking, and fast-forward remain functional at every checkpoint.
- `git diff --check`, `make check`, and `make size` pass without warnings.

Visual adjudication belongs to the user. Do not open the demo or a browser and
do not generate, inspect, compare, or edit screenshots, frames, contact sheets,
or other imagery unless the user explicitly reverses this instruction. If GUI
testing is later requested, use desktop 8 or 9 and close every launched window
when finished.
