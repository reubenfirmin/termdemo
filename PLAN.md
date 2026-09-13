# Continuous Flight Plan

Status: binding design specification for the unfinished single-world journey.

This document separates required behavior from legacy code that happens to
exist in the production modules. If implementation and this plan disagree, preserve the
accepted pre-planet sequence and the latest directed capture behavior, then correct the later
implementation. Do not disguise a
disagreement with a transition, fade, mask, or renderer replacement.

## Current directive — scale and single-function flight rebuild, 2026-09-12

**This section is the active implementation plan. The archived historical
checkpoints linked below are records of earlier work, NOT additional current
requirements or approvals.** In particular, the old curvature-limited brake,
the proposed power-32 scalar function, fixed absolute world coordinates,
old source fingerprints, and the timed regional descent are not solutions
to preserve. Phase 1 separates the code without changing runtime flight behavior.
Phase 1 is complete; preservation evidence, existing failures and the dependency
map are in [phase1-hygiene.md](docs/phase1-hygiene.md). The phase-2
[offline proposal](docs/phase2-study.md) and [complete evaluated grid](docs/phase2-flight-grid.md)
are now connected in the normal build, including the new dimensions and
the single distance-driven flight. See [integration, measurements and resume status](docs/phase3-integration.md).
The old competing controllers are archived, not active beneath the new function.
**Build instruction:** always use the normal `make build` / `make run` workflow.
The user rejected the extra preview output directory. Do not preserve an older
normal executable by compiling current work elsewhere. Audits use the normal
executable and the existing instrumented audit build; report failures without
waiving them or regenerating reference hashes. **Phase 3 implementation and
automated verification are complete. Final visual acceptance remains phase 4/5.**

**Latest phase-3 pass (2026-09-13):** all **25/25 gates pass**, including the
unchanged 33.33-ms render-time threshold (worst audit frame 25.68 ms). A full
2,311-frame sequence through 77 s peaked at 27.78 ms, with no over-budget frames.
The renderer now uses a shared work queue of eight-row screen tiles instead of
fixed per-worker stripes, with ordered grid-command bins and disjoint shared
RGB/depth output. This changes scheduling only: the original four-pixel surface
lattice, all boundary rays, stars, geometry, motion and detail remain unchanged.
Serial/parallel RGB and depth, real worker-failure recovery, one/two-CPU fallback,
normal-build preservation and terminal controls pass. Runtime variability was
also confirmed by testing identical saved executables before any new changes;
do not attribute the entire earlier 53-to-26-ms difference to code optimization.
The original opening reference and its narrowly approved grid revision remain
protected. Next is phase 4's detailed geography, not further flight retuning.
See the integration document for measurements and remaining visual-review risks.

**Foreground decision resolved:** the user's “yes” permits the preserved nearby
stars to remain as unblurred points after emergence. No large exit detour is
selected. No numerical speed or zero-foreground-motion requirement is added.

**Opening-grid question resolved narrowly:** the user's instruction to finish
phase 3 followed the specific question about changing ONLY the grid pixels in
the final approximately 0.02 s. This is recorded as that limited approval, not
as permission to alter the stars or camera. The original reference is retained;
`tests/phase3-opening-grid-revision.json` records the five revised full-frame
hashes AND original-outside-rectangle hashes. All earlier frames, 7,681 native
camera samples and 16,254 original star objects remain exact. There is no timed
renderer switch or masked test region in which arbitrary changes can pass.

**Latest clarification — there are NO fixed numerical speed specifications.**
Exit, orbital, airline-height and near-surface speeds are outputs of the coupled
geometry/timing calculation. In particular, 600 m/s at airline height and
133 m/s at the surface were study values, not binding constraints. Do not
declare the requested descent infeasible by holding either value fixed.
Preserve the Earth-sized planet, approximate event/orbit timing, continuous
front-loaded braking and readable descent. Select and report physical speeds
that satisfy those geometric and perceptual requirements together. The accepted
early table is a pacing baseline, not a list of immutable velocities.

**Latest acceptance — retain the front-loaded journey through orbit 3 entry,
then descend, do not complete orbit 3.** The user accepted the direction of
[the numerical grid](docs/front-loaded-flight-grid.md), with this explicit
exception: once the third revolution begins, rapidly lose altitude and total
speed together toward local continental flight and the canyons. Orbit 3 entry
is a descriptive milestone on the same continuous maneuver, not a switch to
another controller. There is NO required third-lap completion, fourth orbit,
or holding loop. The table's 107-second third lap and 148.657-second aircraft
arrival are superseded, not approved timings.

Retain the table through orbit 3 entry as the approximate planning baseline:
orbit 1 starts at 28 s, orbit 2 at 33 s, and orbit 3 at 41.419 s, with substantial
braking already accomplished in the grid. These are not measurements or an
approval of an unfitted incoming route. The candidate's exact coefficients
were fitted using third-lap completion; refit the ONE global speed function
and geometric descent together without that constraint. Preserve the accepted
early pacing and report any necessary deviations before implementation. Do
not simply truncate the old table, add a late brake, or claim that its remaining
tail now meets the new descent requirement. The new aircraft/surface times
are unresolved; the former 40-second deadline no longer fits this accepted
ordering and must not force another braking cliff.

Planetary fidelity is a prerequisite for accepting that final descent. Fixed,
recognizable continental landforms must resolve into mountain systems, valleys,
cities and the actual canyon geometry as altitude falls. Detail must come from
the same nonrepeating procedural/fractal world, not imagery, a tiled repeat,
newly introduced terrain, or a renderer change. Verify the perception of
descent and braking with these landmarks in uninterrupted playback as well as
paused frames; numerical motion alone cannot establish visual success.

**Latest planetary specification — distinct fractal systems, connected by
geography.** Continents have recursively detailed coastlines and a broad
interior-high/coastal-low elevation tendency. Mountain ranges, regional slopes,
rivers/lakes, and city placement/layout use distinct procedural constructions,
not one noise field recolored for every feature. Drainage follows the final
terrain's gradients and basins; settlements respond to that terrain and water.
All features retain world identity as their finer structure becomes resolvable.
The binding work packages and acceptance gates are in
[Later-phase planetary geography](#later-phase-planetary-geography) below.
This is planned work, not a claim that the current terrain satisfies it.

**Execution-order correction:** implement the unified flight in phase 3,
before the detailed procedural planet in phase 4. Phase 3 includes the basic
fixed-world dimensions required by the reviewed route/speed calculation, so
new motion is never connected to incompatible old distances. Use the existing
shared terrain/renderer and explicit relief-clearance bounds; do not wait for
the new fractal systems or substitute a temporary flat surface. Phase 4 then
develops and evaluates those systems on the established flight. Final visual
acceptance still requires their detail, but it does not block implementing
and checking the single motion authority first.

**Latest user decisions — Earth-sized planet, close five-second first orbit,
fast inward space approach that becomes a shallow orbital descent:** use a
6,371-km planet radius (12,742-km diameter). Raise the proposed speed as needed
to make the ACTUAL first revolution approximately five seconds, accounting
for the one braking function and changing orbital radius. This supersedes
the prior sub-100,000-m/s ceiling and 1,000,000-m/s exit study. Neither the old
world-unit speed nor the earlier 8.018-million-m/s constant-radius arithmetic
is an approved new exit speed.

**Latest braking/report correction:** substantial, smooth speed loss belongs
at the HEAD of the post-wormhole curve. The small, closely spaced physical grid
cells supply the visual impression of speed; retaining nearly full exit speed
through a long plateau is not required. Braking must start promptly and smoothly
after emergence and ease across the journey, not be delayed to a high-power
middle/end cliff to satisfy old distances or deadlines. The 209.4-million-m/s
power-10 diagnostic table was explicitly rejected for precisely that defect.
It is not the next proposal and must not supply new geometry/event distances.

The next numerical TABLE must include explicit milestone labels: orbit 1 entry,
the second-orbit pass, orbit 3 beginning, and the other scene/flight events
listed below. These are observations along one continuous path, never motion
segments or instructions to change speed, position, framing or the renderer.

We arrive from space, not from an already shallow orbit. The inward velocity
component may initially remove altitude very quickly. The ONE geometric route
then bends continuously toward tangential motion; once established in close
orbit, its descent angle is shallow and the remaining altitude loss is more
gradual. Do not prescribe a particular entry longitude, a fixed 20-degree
entry frame, a blanket under-30-degree incoming approach, or an arbitrary
1,000-to-500-km first-lap altitude profile. Choose the entry point, inclination
and altitude from the complete smooth, tunnel-contained approach. A high
descent rate at space speed does not itself require a near-vertical dive.

**Historical measurements before phase 3, not the current runtime:** entry orbit
was about 453,415 km from the planet center: 10.435 old planet diameters, with
about 431,689 km altitude. The old planet radius was 21,726.144 km; total speeds
at 28 and 32 seconds were approximately 2.414 billion and 144.638 million m/s.
Phase 3 has now replaced that scale and flight with the Earth-sized world and
measured close orbit documented in the integration report above.

The exploratory 14-million-m/s function and 1,000-to-500-km first-lap witness
were not accepted and are not implementation targets. Their 79.47-second
600-m/s time and conditional 74% one-second brake bound describe that particular
proposal, not a solved revised flight. Recalculate the full route/speed study
with the newly clarified fast initial descent. Do not silently move later
milestones or claim that changing descent angle alone fixes total-speed braking.

Both previous worked layouts are **withdrawn**, not starting parameters:
[the old multiplier-first study](docs/flight-scale-study.md) retained oversized
grid assumptions; `scripts/flight-speed-first-study.cjs` then went to the
opposite extreme with a user-rejected 650-m-radius miniature planet and 50-m
viewing height. Neither function/table is the new flight proposal. The next
coupled study must regenerate the complete speed table and dimensions from
the accepted early-flight baseline and revised partial-third-orbit descent.
The old 9,144m/600m/s to 20m/133m/s conflict was conditional on those two speed
values. It is NOT a current blocker: fit the final speeds and route together.
The old 40s/58s pair is not the new fitted timeline. No runtime flight,
renderer, world geometry or controls changed during this planning decision.

### Diagnosis: why the iterations have failed

1. Independent scales were conflated. `PLANET_RADIUS_MILES = 13_500` and
   `FLOW_PLANET_RADIUS_WORLD = 0.115` make one world unit 188,922,991 metres.
   The regular grid pitch of 1.5 is consequently 283,384 km: 6.52 planet
   diameters per cell. The tunnel is 955,950 km across. Replacing the old
   pitch progression with this still-huge pitch did not implement the user's
   requested 100x/1,000x-or-more spatial reduction.
2. There are multiple speed authorities: `approach_camera_reference`,
   `world_curve_braked_speed`, the distance/curvature cap in
   `world_curve_controlled_speed`, and later `world_curve_regional_phase`
   plus `world_curve_late_clearance`. At 40 s the code stops integrating the
   earlier distance law and constructs position from the latter timed laws.
   Vertical velocity then raises total speed again. Matching endpoints and
   smoothing each local formula did not make one well-controlled flight.
3. The huge existing path was treated as immutable; the brake was repeatedly
   fitted to cover it by a fixed deadline. This preserved the cause while
   changing where its concentrated slowdown occurred.
4. Tests preserved obsolete constants/old observations and sometimes tested
   the requested envelope, not actual motion. A passing isolated speed test
   says nothing about whether the renderer's camera follows it. Some aggregate
   checks also stop at the first source-lock failure before later gates run.
5. Before phase 1, production, compatibility equations, audits, derived caches
   and obsolete milestone constants shared one large source file. Plan/README
   history was also presented as current instructions. Phase 1 separates these
   responsibilities and archives that history; the rejected runtime control
   laws remain explicitly identified until phase 3 replaces them.

### Binding design

- One physical universe, one metric conversion and one camera. Local features
  may have very different physical sizes; that does not require different
  coordinate systems or time-dependent geometry. No screen-relative planet,
  moving grid, runtime scale change, renderer replacement, or imagery.
- Preserve the approved opening experience. Star acceleration/streaking is
  pre-wormhole, not a second post-emergence act. At emergence (about 4–5 s),
  distant rings are already visible and the ship already has traversal speed.
- The post-wormhole structure is sub-stellar. Distant stars should have little
  translational parallax during grid traversal. Rendering nearby stars without
  blur is not enough: the recent shutter fix retained their nearby geometry.
  Star depth must be independent of grid size; never freeze their projections
  or move them with the camera. Preserve exterior/atmospheric visibility rules.
- Axial cell pitch, transverse cell pitch, circular radius, squared-tunnel
  width/height, structure length, stellar distances and planet radius are
  separate physical parameters. Do not use one `FieldLayout.radius` to decide
  all of them. The square structure can widen smoothly downstream to contain
  the planet and maneuver while retaining small cells.
- Keep the assembling fixed spokes, circular-to-square evolution, and dense
  line passages. The ENTIRE interval from approximately 7.6 s to 23 s is
  the circular-grid structure: irregular/incomplete spokes at entry,
  progressive assembly through that same structure, and evolution into the
  squared grid near its end. It is not a streak act followed by a separately
  introduced cylinder, and 7.6 s does not mean fully assembled spokes.
  The planet stays nested in the squared structure's floor,
  in a pronounced gravity well of that same mesh. The mesh continues beyond
  the planet. No roof/side escape; only the localized under-planet passage
  permits crossing below the nominal floor. Incoming entry point and angle
  are free within those spatial constraints; the established orbital descent
  becomes shallow continuously, without an abrupt level-off or attitude snap.
- One decreasing speed function governs the complete automated flight from
  emergence through near-surface arrival. It controls TOTAL world speed,
  including radial descent and lateral movement. No additional capture brake,
  orbit cap, angular-speed controller, altitude clock, or corrective speed.
- Front-load substantial smooth braking immediately after emergence; do not
  preserve a long near-constant-speed head and put the reduction at the end.
  Use independent small physical grid cells to retain strong passing-line
  motion as speed drops. Publish early absolute/percentage losses explicitly,
  and verify projected geometry/cadence rather than treating speed alone as
  the visual-speed requirement. Smooth endpoint derivatives are not proof
  that braking is correctly distributed.
- Select its amplitude and shape together with the Earth-sized planet's close
  approach so the ACTUAL first revolution is approximately five seconds.
  The former exit ceilings, constant-radius speed estimates and 1.2-second
  first-orbit target do not override the latest decision. Measure real angular
  travel around the planet from the route, not camera rotation or a new clock.
- One continuous geometric route includes approach, capture, descending
  revolutions and canyon arrival. Keep visible sky in early orbits, recurring
  fixed Sun, perceptible shrinking altitude/curvature, and the procedural
  continent-to-canyon geography/cities. No extra holding loops to spend time.
- At the beginning of orbit 3, continue into a rapid, smooth reduction of both
  altitude and total speed; do not wait for another revolution. Fit this as
  part of the same route and global speed function, with continuous direction,
  bank, acceleration and a progressively easing final brake. Aircraft-height
  and canyon arrival are spatial milestones, not orbit-completion endpoints.
  Detailed persistent landforms must make the shrinking scale and ground
  motion readable; visual fidelity is part of acceptance, not later polish.
- Lose altitude rapidly on the incoming space approach, then let the route
  progressively turn its inward velocity into tangential motion. Radial and
  tangential components come from the same unit route tangent multiplied by
  total speed; they are not separately scheduled velocities. There is no
  motion reset or new controller when the descriptive word "orbit" applies.
- Surface arrival means safe clearance over displaced terrain (currently
  20 m), not impact with the mathematical sphere. The pilot then assumes
  manual control of the SAME flight state. This explicitly replaces the old
  obligation to run a prerecorded trajectory through 137 s. Manual input
  bindings/UX will be specified before that controller is implemented; do not
  claim the existing pause/seek controls are flight controls.

### Protected approximate animation schedule

The user explicitly requires the keyframes/key events to remain approximately
where they are in animation time. Solve speed, cell spacing, structure length
and cross-section together to meet this schedule; do NOT shrink the world
while keeping old event distances, or lower speed and let entry drift later.

| Event / interval | Target demo time | Required observation |
|---|---|---|
| Approved opening | 0–4 s | Preserve the approved opening behavior; verify emergence continuity as described below. |
| Wormhole emergence | About 4–5 s | Already at post-wormhole traversal speed; rings distantly visible; no renewed streak-building/acceleration phase. |
| Circular-grid entry | About 7.6 s | Enter the actual structure with irregular, incomplete spokes, not a fully assembled cylinder. |
| Circular-grid traversal and assembly | About 7.6–23 s | ONE continuous structure; spokes progressively connect/regularize; near the end it evolves into the squared grid. |
| Fully squared approach | About 26 s | Preserve the earlier extended squared-grid flight and room for the fluid planet approach. |
| Planet introduction / approach | About 27–28 s | Planet nested in the floor/well; continuous approach into orbit, not a timed spawn or motion reset. |
| Orbit 1 entry | About 28 s | Accepted numerical baseline; actual close first revolution lasts approximately five seconds. |
| Orbit 2 begins | About 33 s | Continuous braking and descending orbital travel; retain early sky/horizon readability. |
| Orbit 3 begins / final descent underway | About 41.419 s | Accepted numerical baseline; rapidly shed altitude and speed together, without completing a third lap or triggering a new controller. |
| Airline-height flight | During the partial-third-orbit descent; time to fit | Approximately 9,144 m AGL; total speed is a fitted output, NOT fixed at 600 m/s. The old 40 s deadline is superseded by the accepted orbit ordering. Detailed continental features must be readable. |
| Near-surface arrival / manual control | Time to fit, no lap-completion dependency | Retain the requested 15–20-second post-aircraft descent as a feasibility target, not a solved promise; safe terrain-relative clearance. The old 58 s deadline is not a fitted result. |

### Required columns and milestone rows in the next numerical table

Include time, event/milestone, total speed in m/s, percent of exit speed,
percent lost over the last one second, terrain-relative altitude, route
distance and continuous orbit progress. Keep every whole-second row and add
rows at actual fractional event times; do not round a 7.6-second entry or an
orbital crossing to a whole second. "Last one second" always compares v(t)
with v(t-1), NOT with the preceding table row when event rows are interleaved.

Label wormhole exit, circular-grid entry, spoke assembly/square evolution,
fully squared flight, planet approach, orbit 1 entry, its far-side passage,
orbit 1 completion/orbit 2 beginning, the second-orbit pass, orbit 2 completion/
orbit 3 beginning, the ensuing partial-orbit descent, atmosphere entry, airline
height, and near-surface/canyon arrival with manual takeover. Orbit 3 completion
is not a target or a condition for any arrival. State which
geometric crossing is meant by a "pass" so successive rows are comparable.

Derive these rows from the evaluated route and world geometry, not just the
target-time list. Mark an event unreached or unresolved if the candidate does
not get there; never label a speed endpoint as orbit completion or surface
arrival without the corresponding position. The table must make actual lap
durations and the distribution of braking across the first two laps and final
partial-orbit descent directly visible. Stop the automated-flight table at
safe surface/manual arrival, not at completion of the third revolution.

The 7.6–23-second definition is the latest explicit direction. Old labels
that call 4–11 s a post-wormhole streak region or defer "circular tunnel"
until `CLEAN_BIRTH` around 20 s are obsolete. Do not reinstate a separately
timed logarithmic-ring act or demand fully connected spokes at 7.6 s. Earlier
21-second square-transition constants are implementation observations, not
an exact lock overriding the specified evolution near 23 s. A gradual morph
can overlap that interval; it must not introduce a sharp boundary at 23 s.

These are target observations along one route, not a scene schedule in the
renderer or a list of velocity joins. In phase 2 publish the proposed and
measured event times side by side, with deviations visible. The later targets
still require the distance/altitude feasibility checks below; do not silently
retime them if that check fails. No unrequested exact timestamp tolerances
or new orbit counts are being approved here.

For each key event `t_i`, compute route distance `s_i = integral(v, t_exit,
t_i)` and evaluate the corresponding fixed route location. Place the physical
field boundaries from those locations, using the actual field coordinate
rather than confusing curved-route arc length with axial distance. Check
both actual geometric entry and what is visible: distant grid visibility
at emergence is not the same event as entering its incomplete spokes at 7.6 s.

### Scale study and feasibility — speed and planet first, dimensions afterward

For comparable flight in the initial grid, reduce physical speed, axial pitch,
axial extents and circular cross-section by the same factor K. Then line
crossing frequency `v / pitch` and dimensionless perspective motion can be
preserved. This is a physical layout change with a fixed metre conversion,
not merely renaming world units. Scaling pitch/speed alone does NOT preserve
screen motion if the walls and rings remain at their former distances.

Do NOT begin with K, old metre conversions, old distances or old speed values.
The latest decision selects an Earth-sized planet and an actual approximately
five-second first revolution reached through a fast inward space approach.
Solve the closer entry geometry and single speed function together, without
fixing the approach to a shallow spiral prematurely. Any comparison multiplier is an
OUTPUT of the new design, never an input that forces speed or planet size.
The preceding similarity relationship is an explanatory tool, not a mandate
to scale every dimension uniformly or preserve an old pitch-to-radius ratio.

Use the now-selected 6,371-km planet radius; do not revive the old 21,726-km
radius or the rejected 81-km or 650-m bodies. Publish the actual incoming path,
entry altitude and inward/tangential speeds as well as the complete first-lap
distance and duration. `2*pi*r/5` is only a constant-radius, constant-speed
comparison, not the complete descending/braking calculation. Retain
independently chosen small grid cells, square dimensions
large enough for the planet and maneuver, and the approximate event times.
Integrate the proposed speed function to obtain event distances, then locate
the fixed structures along the one route. Do not select oversized extents
first and raise the speed again to reach them on time.

Report actual orbit radius, clearance, curvature and atmosphere/terrain scale,
not just whether a sphere fits in a box. Measure the approximately five-second
target using the ACTUAL first lap during braking/descent; never introduce a
second orbital clock to force it. The old 1.2-second period is superseded.
Low-altitude flight is local continental travel, not rapid complete revolutions;
its speed is a design output. Any remaining timeline/orbit-count conflict needs an
explicit proposal, not another miniature planet or hidden speed override.

For each candidate publish:

- Planet radius, terrain relief and atmosphere height in metres; planet to
  square-width/height ratios; entry radius and full maneuver clearances.
- Circular dimensions, square width/height profiles, axial/transverse pitch,
  ring/spoke counts, field length and event distances. Widening and squaring
  are smooth fixed functions of the field coordinate, not camera triggers.
- Expected ring passages per second and projected edge speeds/coverage at
  emergence, ring ordering, full cylinder, square entry and capture. Preserve
  visible spoke assembly independently of both star blur and cell count.
- Stellar depth versus the complete post-exit camera displacement; proposed
  numerical bound on translational star parallax, distinct from camera rotation.
- Whole-flight arc lengths, orbital periods, aircraft-height time and surface
  time under the proposed speed function, including its vertical budget.
- Worst visible object count/raster cost. Do not generate K times as many
  offscreen objects blindly. Stable identities, analytic visible-range queries
  and subpixel filtering must preserve real grid structure and partial rails.

Two continuity/feasibility checks cannot be hidden by the rescale:

1. **Wormhole exit boundary.** The approved opening currently ends its rush
   at 4 s with speed 18 old world units/s. Replacing this with 18/K at 5 s
   creates a new brake unless the boundary geometry/scale is planned too.
   Record position, velocity, acceleration and camera frame on both sides
   of the actual emergence event. A consistent static similarity transform
   of opening geometry and camera may be studied for exact projection parity;
   it must not become a runtime coordinate switch. Do not assume new star
   distances automatically preserve the opening. If exact protected states
   and the rescale cannot coexist, identify the minimal required revision and
   obtain approval before changing it. No hidden 4–5-second speed reset.
2. **Aircraft-to-ground distance.** Fit total speed, descent geometry and the
   requested subsequent 15–20 seconds together, starting near 9,144 m AGL.
   Neither the starting nor the terminal numerical speed is fixed. Calculate
   available arc distance and radial/tangential components from the ONE speed
   law and route, including actual terrain elevation changes. Refit arrival
   times without the superseded 40 s deadline or a third-lap endpoint. Report
   the selected speeds and their visual implications; do not revive a conflict
   caused solely by fixing obsolete study values, add vertical speed after the
   scalar function, or silently extend the flight.

The study must end with ONE proposed parameter set and full percentage table,
not another collection of runtime tuners. Get review of any changed absolute
planet size, opening boundary or event timing before connecting it live.

### Intended motion interface

```
fixed WorldGeometry + fixed Route P(s), with |dP/ds| = 1
single SpeedLaw v(t) > 0, v'(t) <= 0
distance s(t) = integral from emergence to t of v(tau) d tau
position(t) = P(s(t))
velocity(t) = P'(s(t)) * v(t)
acceleration(t) = P''(s(t)) * v(t)^2 + P'(s(t)) * v'(t)
```

For the radial/tangential decomposition, with outward unit normal
`n=P/|P|` and inward tangent fraction `c(s)=-dot(n,P'(s))`:

```
radial descent speed = v(t) * c(s(t))
tangential speed     = v(t) * sqrt(1 - c(s(t))^2)
```

The approach may have a large inward component; the route smoothly reduces
that component as it wraps into close orbit. `c(s)` is derived from the fixed
route, not an independent altitude/time controller. These are reference-sphere
radial components; terrain-relative clearance must additionally use the same
displaced terrain geometry. Leveling off must not introduce a total-speed
change, an extra braking term, or a hidden jump in direction/curvature.

The curve is one route evaluator, not a scheduler selecting different moves.
Numerical integration/cache intervals must not become motion authorities.
Terrain clearance, orbit phase and altitude follow the geometric route;
they do not independently prescribe position as functions of time. Curvature
is a design validation constraint, never a runtime speed limiter. Derive a
continuous camera frame from the same route with smooth look direction/bank;
do not reset attitude when crossing a named event.

Choose and publish one global analytic speed family, with parameters frozen
before flight, after the scale/arc-distance feasibility study. A useful
formulation to investigate is
`v(t)=v_exit*exp(-L*B(u))`, `L=ln(v_exit/v_surface)`,
`u=(t-t_exit)/(t_surface-t_exit)`, where B is a single smooth monotone function
on [0,1] with B(0)=0 and B(1)=1. A normalized integral of a nonnegative smooth
global density gives those properties by construction. Endpoint derivatives
must match the incoming state and permit continuous manual takeover. Its
parameters must also make the late fractional braking rate `-v'/v` relax,
not merely make absolute acceleration small. This is a design formulation,
NOT selection of a fitted curve or permission to reuse the power-32 proposal.
Do not introduce per-orbit coefficients, threshold-dependent caps or a late
arrival correction if the first candidate is infeasible.

### Implementation phases and gates

1. **Hygiene and a trustworthy measurement harness — completed 2026-09-12.**
   Separate fixed geometry/units, route geometry, speed/distance evaluation,
   camera framing and audit-only legacy equations. Use small modules with
   explicit inputs; preserve the single production renderer. Make this an
   extraction-only pass, not a flight tuning pass. Record the existing failing
   behavior and compare numerical outputs before/after exactly. Do not bless
   it as correct. Replace conflicting current documentation with one directive
   and archive historical assertions. Make independent numerical gates run
   even when another gate fails; retain source locks with explicit status.
   Deliver a dependency map and one reproducible complete speed/altitude report
   from the ACTUAL camera, with all current failures listed. No imagery.
2. **Coupled scale, route and speed feasibility study — completed; connected and
   render-checked in phase 3.** Produce the candidate
   comparison above, include exact emergence continuity, then recommend one
   layout and one function that retain the approximate animation schedule,
   especially the full 7.6–23 s circular-grid interval. Publish each whole second from emergence to the
   surface: total m/s, % exit speed remaining, % lost since the preceding
   second, AGL, distance and orbit phase, with the milestone/event column and
   fractional-time rows specified above. Report the substantial early braking
   and grid line-passage cadence, not just the terminal speeds. Use the accepted
   table through orbit 3 entry as the approximate baseline; jointly refit the
   one global function and route for rapid altitude/speed loss during a partial
   third revolution. Do not solve aircraft arrival by completing another lap.
   Publish aircraft/surface times and changes to earlier rows explicitly.
   Also report dense-sample maxima for
   100-ms/1-s losses, fractional braking, acceleration and jerk. This phase
   may use an offline numeric prototype, not a second live camera/controller.
   Reserve explicit terrain/obstacle height bounds for the later geography
   work and check route clearance against them before connecting the flight.
   Resolve explicit target conflicts before implementation, not through caps.
3. **Connect the single distance-driven flight atomically — implementation and
   automated verification completed 2026-09-13.** Parameterize the
   complete route by arc length and install the reviewed speed function once.
   Remove the active approach/orbital speed overrides, curvature speed table,
   runtime braking fit and timed regional angle/altitude laws. Do not retain
   them underneath a new envelope. Every production camera position and
   derivative must now come from P(s(t)), through surface arrival. Geometry
   evaluation and playback may share caches but not independent authorities.
   Install the basic fixed-world dimensions from phase 2 in the same coherent
   change: independent stellar depths, grid pitches and cross-section profiles,
   Earth-sized planet, square, floor gravity well and underpass clearances.
   Keep stable ring/spoke identities and shared widening/squaring vertices.
   Preserve the approved opening as established in phase 2; check cadence,
   visibility, topology, floor attachment and alternate routes through the
   unchanged world. Never combine new speeds with incompatible old dimensions.
   Use the existing shared terrain/renderer and the reserved relief bounds to
   check the complete descent now; no temporary local plane, detail removal,
   holding lap or wait for the new planetary algorithms. Record actual motion,
   containment, clearance and live playback failures. Full visual acceptance
   of the descent remains open until phase 4 supplies the required detail.
4. **Develop the detailed procedural planet on the established flight — next.** Build
   the distinct continent, relief, drainage, settlement and refinement work
   packages 4A–4E below; generic fractal noise is not completion. Reuse the
   same fixed world, shared terrain source and production renderer. Landforms,
   mountain/valley/canyon geometry and day/night cities must resolve continuously
   with projected footprint, without tiled repetition, imagery, moving landmarks,
   silhouette popping or an independent surface. Keep the reviewed metric scale
   and motion unchanged while developing detail; report any clearance conflict
   requiring a design revision rather than silently retuning the flight.
   Validate the partial-third-orbit descent against identifiable features:
   altitude and total speed must fall together visibly, without a hold for
   orbital completion or a late abrupt stop. Include the cross-scale feature/
   lighting checks below and inspect uninterrupted playback and user-paused
   frames. Do not declare pacing accepted from scalar tests alone.
5. **Surface manual control and complete verification.** At safe near-surface
   arrival preserve position, velocity, acceleration and attitude; transfer
   control authority only, never renderer/world/camera identity. With neutral
   input there must be no snap, stop, replay, or continued scheduled orbit
   correction. Keep continuous terrain collision/clearance handling and the
   canyon geometry. Specify bindings without breaking Space pause or Escape.
   Validate drainage, terrain, water and settlement consistency on the actual
   final surface, including views away from the default flight corridor.
   Finish with uninterrupted user live review, not a numerical approval claim.

Do not interleave phase 1 extraction with phase 3 flight/world integration or
phase 4 terrain development. Prototype phase 2 without replacing the live
flight; connect the coupled motion/dimension changes in phase 3 only when
feasible and reviewed. Detailed planetary development follows in phase 4,
not as a prerequisite to phase 3. No partial live build that uses new distances
with an old speed controller, or vice versa.

### Later-phase planetary geography

These are development work packages within CURRENT phase 4, followed by phase 5
verification, not flight segments, new rendering modes, or a revival of
historical phase approvals. Phase 3's unified flight is already connected
before these packages begin.
They implement the user's coast-to-canyon specification. The constructions
below are the planned algorithm families; choose and document their physical
scales, seeds, bounds and storage costs before implementation. Distinct systems
may share basic noise/hash utilities, but cannot be the same terrain noise
reused as unrelated water, mountain and city masks.

All outputs belong to one fixed spherical world. The camera and animation time
do not generate or reposition features. Rendering selects a filtered level of
detail from that fixed hierarchy using projected footprint. Generation may be
cached in bounded, deterministic regional hierarchies; visiting a region in a
different order or along another route must produce the same geography.

| Work package | Planned construction | Required geographic result |
|---|---|---|
| 4A — Continents and coastlines | Nonperiodic continental regions with bounded recursive boundary displacement/domain warping; persistent island and inlet hierarchies | Recognizable large continent outlines from orbit; successively smaller peninsulas, bays, islands and shore detail on approach. No repeating coast tile or fresh coastline at each viewing height. |
| 4B — Continental relief and ranges | Broad interior-distance/uplift envelopes with regional basins; directed branching ridge networks and anisotropic ridged multifractal residuals | Continents generally higher inland and lower at their margins, with asymmetric plateaus, plains and basins rather than a single radial cone. Coherent mountain chains resolve into peaks, spurs, saddles, valleys and cliffs. |
| 4C — Rivers and lakes | Terrain-constrained branching watershed hierarchy, flow accumulation and basin/spill analysis; bounded recursive channel/shore refinement | Dendritic drainage grows more detailed without flowing uphill, crossing divides arbitrarily or ending on slopes. Lakes occupy actual basins and rivers connect to valid outlets or explicitly closed basins. |
| 4D — Settlements | Terrain/water-weighted hierarchical clustering for placement; recursive district/block subdivision and constrained branching street growth for shapes | Uneven, self-similar city/town distributions and distinct settlement silhouettes, resolving into districts, streets and buildings. Locations and layouts respond to coastlines, river valleys and buildable slopes. |
| 4E — Continuous refinement and materials | Stable parent/child geometry, footprint-filtered residuals and materials derived from final elevation, slope and water state | Increasing geometric detail and readable terrain gradients through orbital, airline and canyon views, without changing landmark identity, popping, tiling or hiding defects in atmosphere. |

**4A — Establish land, sea and the broad elevation envelope.** Give continents,
islands and coast features stable identities on the sphere, with consistent
boundaries across chart seams and poles. Use inland distance and regional
uplift/basin structure to bias broad elevation upward away from coastlines;
this is an overall tendency, not a rule that every step inland goes uphill.
Allow low inland basins and occasional coastal cliffs while retaining broad
low coastal margins. Maintain one sea-level datum and derive the final land/
water boundary from the same terrain used for rendering and clearance; a
decorative coastline mask cannot disagree with actual water intersection.

**4B — Build geometric relief, not painted mountain texture.** Place coherent
range axes and branching spurs within the continental envelope, then add
directional, scale-dependent ridge detail and valley incision. Publish the
actual relief heights, wavelengths and slope distributions in metres; do not
fake altitude by changing the vertical scale as the camera descends. Preserve
room for rolling/graded terrain, plains, foothills, plateaus and drainage
basins instead of covering every land pixel in equal-amplitude spikes.
Normals and elevation/slope material gradients derive from this relief.
Nearer ridges must occlude farther valleys and show increasing parallax and
silhouette height as we descend; color contrast alone is not evidence of height.

**4C — Generate a consistent drainage system.** Use the continental relief as
the parent of a hierarchical watershed/flow graph. Smaller tributaries refine
those catchments; fractal branching and meander detail are constrained by the
terrain gradient, catchment boundaries and downstream connectivity. Carve
channels and canyon branches into the same terrain, then validate drainage
against the final carved/refined surface, not just its pre-carving parent.
Fine residuals must not introduce unaccounted dams, uphill reaches or changed
major outlets. Generation and consistency checks precede flight; there is no
camera-triggered erosion or water relocation.

Lake placement comes from depressions and their spill levels. Lake surfaces
have a consistent level relative to the planet datum; their shorelines are
intersections with basin terrain, with finer shoreline complexity inherited
from the same basin. Resolve genuine endorheic basins explicitly; otherwise
connect lake outlets and tributaries to downstream water. River widths and
hierarchical sizes respond to contributing catchment/flow estimates, rather
than drawing every branch at equal width. The destination canyon must be a
geometric continuation of a watershed visible from above, not a new corridor
introduced near the ground.

**4D — Place and shape fractal settlements after terrain and drainage.** Use
stable hierarchical clustering with coast, river/lake access, elevation and
slope constraints to locate cities, smaller towns and gaps between them.
Refine urban regions with recursive parcels and constrained branching street
patterns, allowing varied orientation, density and scale rather than repeating
one rectangular stamp. Do not place ordinary buildings underwater or across
unbuildable cliffs; preserve water channels through the settlement layout.
District and building footprints/heights are fixed world geometry. Day-side
forms and night-side lights refer to those same settlement identities, with
lighting derived from the fixed Sun, not separate day/night city populations.

**4E — Resolve the hierarchy continuously.** Coarse coast/range/watershed/city
parents remain recognizable while bounded child detail becomes resolvable.
Filter unresolved wavelengths and geomorph refinement continuously using
projected footprint, not timestamps or altitude-triggered replacement meshes.
The fully defined surface remains the authority for terrain gradients,
drainage and collision/AGL; coarser rendered approximations have explicit
error bounds and cannot relocate a river, city, peak or canyon entrance.
Use elevation, slope, water proximity and lighting for continuous material/
biome transitions that reinforce the geometry; avoid flat repeated color
patches and contour-band aliasing. All features use the same world transform,
depth/occlusion rules and production renderer. No generated imagery or raster
texture substitution for geometric features.

**Phase 4 terrain-on-flight integration gate.** On the flight connected in
phase 3, during the partial-third-orbit descent, retain
identifiable coastlines, ranges, basins and settlement clusters long enough
to read their changing scale. At aircraft height expose actual relief,
tributaries/lakes and urban structure; nearer the ground expose the connected
canyon walls and valley floor. Use the same fixed Sun for relief and day/night
city visibility. Atmospheric depth/fading may not conceal missing geography
or make an unrendered descent appear successful. This does not authorize a
holding lap, camera snap, independent speed adjustment or protected-opening
change to compensate for inadequate detail.

**Phase 5 verification gate.** Publish deterministic checks for coast/sea
agreement, representative inland-versus-coastal elevation profiles, parent/
child landmark identity, slope/normal consistency, downstream flow with no
unexplained cycles or uphill reaches, lake levels/outlets and settlement
water/slope exclusions. Include negative cases: a tiled continent, randomized
LOD seeds, a river over a ridge, a sloping lake and submerged city blocks must
fail. Check visible relief/occlusion and the same named landmarks at several
viewpoints and resolutions, including an alternate route. Report geometry,
cache and frame-time budgets without reducing quality or weakening existing
performance gates. Finish with user review of continuous playback and paused
detail; passing geometry tests alone is not visual approval.

### Acceptance checks that replace the insufficient evidence

- Production `|dx/dt|` agrees with the chosen v(t) over the ENTIRE automated
  interval, including descent. Independently differentiate position and check
  it, not just a stored velocity field that repeats the requested speed.
- Dense samples plus analytic/interval bounds catch local reacceleration,
  interpolation ripple and discontinuities between sampling knots. Refine
  integration resolution independently; specify error budgets in metres and
  relative terms, not old scale-dependent tolerances.
- Numerical traces include the complete 5s-to-surface percentage table and
  subsecond peaks. Never limit the monotonic gate to 4–28 s or 25.658–40 s.
- Negative witnesses must fail for each old defect: a curvature speed cap,
  separate descent speed, an abrupt exit rescale, a roof escape, a behind-camera
  spoke lost by clipping, nearby post-exit streak/point-star rush, and a missing
  initial ring. Keep independent surface/framing/performance failures visible.
- Fixed metric dimensions, cell spacing and planet/tunnel fit are tested
  separately from pixel cadence. Merely changing the unit label fails the
  scale test. High-frequency line disappearance cannot count as speed.
- Measure circular-structure entry near 7.6 s, incomplete-to-regular spoke
  connectivity throughout 7.6–23 s, and evolution toward square near 23 s.
  Keep the remaining key-event times approximately fixed. Negative witnesses
  must reject moving entry later after a rescale or starting a new streak act
  inside this circular-grid interval.
- Camera pose, world identity and opening regressions are checked explicitly.
  A second substantially different test route cannot alter any world object.
- The final descent starts with the third revolution and reaches local/surface
  flight without requiring its completion. A test route held until three full
  laps must fail; a separate late brake/altitude clock must also fail even if
  aircraft-height and speed endpoints happen to match.
- Track fixed continental, mountain, city and canyon landmarks across orbital,
  aircraft-height and low-flight viewpoints. Geometry and terrain clearance
  must agree through refinement; no tiled repeat, substituted landscape,
  temporal popping or atmosphere hiding a globe-to-flat jump. Live visual
  acceptance must demonstrate readable descent and braking, not merely detail
  in isolated frames. Keep the existing geometry-quality/performance gates.
- Manual takeover preserves the exact boundary state. Autopilot does not
  reassert a later timestamp/position after the pilot takes control. Paused
  state and recorded-input replay remain deterministic; no skipped input
  history is invented when seeking in manual flight.
- Build warnings, performance targets and existing procedural terrain detail
  remain gates. Do not reduce geometry quality to obtain a faster measurement.

**Next action:** phase 4A, the fixed continent/coast hierarchy described above.
Phase 3 is connected and all 25 live gates pass. The actual Earth-sized world
and flight reach circular entry at 7.6 s, orbit entries at 28 / 33.097 / 41.374 s,
airline height at 57 s and a terrain-cleared canyon endpoint at 77 s after about
2.3139 laps. Preserve those dimensions, the single motion function and the
approved opening while developing geography. The integration report records
exact execution parity, measured frame time and full normal-program playback;
none substitutes for the detailed-descent visual acceptance still to come.

**Historical foreground study, resolved by the user's approval of unblurred
points:** the common opening similarity retained 1,721 nearby opening stars in
the reference circular-entry view; they subsequently pass behind the camera.
Adding a distant background does not remove that foreground rush. Review any
change to the protected opening's star layout or any relaxation of the
post-wormhole foreground requirement before doing it. Do not mask the issue
with a time fade, move stars with the camera or claim exact opening parity.
This is a failed candidate check, not a proof of incompatible requirements:
the new complete-catalogue query and screening study test 16,254 objects, not
only the earlier two-frame witness. The bounded search retains 1,790 candidate
orientations/bends with a visible ring at emergence; none eliminates foreground
traversal. This does NOT prove impossibility across other routes or free
scale/speed choices, and does not authorize large detours or a changed opening.
The selected proposal has only the shallow in-tunnel excursion, plus the
corrected actual-radius capture and a fixed Sun visible in both early orbits.
That proposal is now the live flight. Exact opening/render checks remain required
and pass with the separately documented, narrowly approved grid revision.

## Historical checkpoints (archived)

Earlier plans and audit claims are preserved in [the historical record](docs/history/flight-plan.md).
They are not additional active instructions. The previous README is also
[archived](docs/history/README-before-phase1.md) to retain context without
presenting superseded scenes or approvals as current behavior.
