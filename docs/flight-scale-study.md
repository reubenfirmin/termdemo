# Worked scale and speed study — 2026-09-12

**WITHDRAWN / HISTORICAL.** This multiplier-first layout is not the current
proposal. The user rejected its scale assumptions, then rejected the later
miniature-planet alternative. The latest decision is an Earth-sized planet,
a fast inward space approach becoming a shallow close orbit, and an actual
first lap of approximately five seconds. See the current directive in PLAN.md.
All recommendations and prospective tables below are superseded history.

Analysis only. No production flight, renderer, camera or controls were changed.
Reproduce the numeric study with `node scripts/flight-scale-study.cjs`.
The script prints JSON and writes no files. The active specification is
[PLAN.md](../PLAN.md). These are prospective speeds, not the current demo.

## Conclusion / recommendation status

The **1,000x case was the historical layout study**, not an approval-ready flight.
It preserves the circular-grid clock and provides a single explicit decreasing
speed law. A small fictional planet can accommodate the requested fast orbit.
However, the inherited 40s altitude/speed and 58s ground targets are incompatible
with an easing braking tail. A tested simple spiral also compresses the visible
descent too early. Neither defect is hidden by calling the scalar/arc test a pass.

The next implementation remains behavior-preserving hygiene. Do not install
this function before the terminal constraints and complete route are resolved.
The earlier audit-only `src/flight_speed.rs` is a DIFFERENT, superseded proposal.

## 1. Fixed physical dimensions, not a unit-name change

| Quantity | Old physical value | 1,000x study / proposed revision |
|---|---:|---:|
| Opening post-rush reference speed | 3,400,613,843 m/s | 3,400,614 m/s |
| Mature axial grid pitch | 283,384.487 km | 283.384 km |
| Initial axial pitch (representative) | 620,989.872 km | 620.990 km |
| Circular tunnel diameter | 955,950.336 km | 955.950 km |
| Squared tunnel width and height | 955,950.336 km | 1,911.901 km |
| Planet radius | 21,726.144 km | **81 km**, subject to explicit review |
| Planet diameter | 43,452.288 km | 162 km |
| Planet-centered orbit-entry radius, simple witness | 453,415 km | 282.719 km |
| Nominal atmosphere height | 100 km | Study about 30 km for the small body; optical profile not yet selected |
| Distant star depth near the grid | Locally overlapping stars, tied to field dimensions | At least 500 million km; independent of grid pitch/width |

The square is twice the new circular diameter, while its axial cells stay at
283.384 km. Thus the square shrinks 500x, not 1,000x. The planet shrinks about
268x, not 1,000x. Those differences are deliberate independent dimensions.
The 162-km body occupies about 8.5% of the square width and sits in its floor/well,
not on its axis or roof. Final well displacement and capture containment still
require the complete route, not just these scalar dimensions.

Keep 72 principal spokes in the mature circle. Its transverse pitch is about
41.711 km. A widened square with only 72 spokes would grow transverse cells to
106.217 km; that repeats the coupling error. Study a permanent nested catalogue
with up to 216 meridians, progressively connected at fixed locations, giving
about 35.406-km transverse pitch in the square. Do not toggle extra lines by
timestamp or add a second assembly act. Ring/spoke junctions must share vertices.

Axial pitch remains independent: do not quietly densify its lines near the
planet and manufacture a visible acceleration. Curved-line tessellation needed
to resolve the gravity well is geometric sampling of the same lines, not
additional decorative grid lines. Check whether the 283-km material pitch
adequately depicts the well around a 162-km planet; if not, publish a revised
pitch/cadence proposal rather than assume increasing the count is harmless.

Use planet-scale broad wells and smaller core curvature with exact floor/planet
contact. The route must remain below the roof and between the sides; only the
localized under-planet exception may go beneath the nominal floor. A bounding
sphere fitting in the square is not a proof of the complete approach.

This is a small fictional planet, not an Earth-sized body. At 9,144 m above an
81-km sphere, the geometric horizon is depressed about 26.0 degrees. It will
look more curved than the old giant planet. An approximately 44-degree downward
view would place the central horizon around the upper quarter of the frame;
the full terrain coverage/Sun projection is not yet validated. The old
12-degree camera pitch cannot be retained blindly. A larger-body alternative
is the 100x case below, with a 270-km radius (about 14.7-degree horizon depression).

Geography must be specified in metres independently: study continent features
tens of kilometres across, kilometre-scale ranges/canyons, and metre-to-hundreds-
of-metres surface/settlement detail. Do not shrink a cached terrain image or
repeat a wrapped tile. Atmosphere thickness, relief and city dimensions need
review with the chosen radius. Procedural spherical geometry remains mandatory.

## 2. Scale comparison using the SAME function family

Planet radii below were selected for orbit-distance feasibility, not merely
scaled with the grid. They are proposed alternatives, not simultaneous worlds.

| Reduction | Exit reference, m/s | Planet radius, km | Circular diameter, km | Axial pitch, km | Peak fractional brake, /s | Worst 1-second loss |
|---:|---:|---:|---:|---:|---:|---:|
| 100x | 34,006,138.435 | 270 | 9,559.503 | 2,833.845 | 1.065 | 65.436% |
| 1000x | 3,400,613.843 | 81 | 955.95 | 283.384 | 0.848 | 57.078% |
| 10000x | 340,061.384 | 17 | 95.595 | 28.338 | 0.635 | 46.956% |

The 10,000x case puts 9,144-m flight altitude more than halfway up the body's
radius; its horizon depression is about 49.4 degrees. That is a poor default
for the requested plane-like continent view. The 100x case allows a larger
planet but requires a stronger fractional brake. The 1,000x case is a middle
study, not proof that all remaining requirements can be met.

## 3. One explicit prospective speed function

Use metres and seconds throughout. Let:

```
t0 = 4                         # current approved rush-end boundary
V0 = 3400613.8434782606
Vinf = 133
p = 10
L = ln(V0 / Vinf) = 10.149117387268342
a = 36 / (ln(V0/600) / ln(600/133))^(1/10)
  = 30.229962726536776

y(t) = ((t - t0) / a)^10
v(t) = V0 * exp(-L * y(t) / (1 + y(t)))       for t >= t0
s(t) = integral(v(tau), tau=t0..t)
position(t) = P(s(t)), with |P'(s)| = 1
```

The requested post-wormhole table begins at 5s. The function begins at the
existing 4s rush-end boundary to avoid inventing another 4–5s braking/reset
maneuver. V0 is exactly the old boundary speed divided by 1,000. Its first
two time derivatives vanish there. Exact opening projection/state parity under
a consistent static geometric rescale is a REQUIRED future proof, not a claim
already established by this scalar script.

This is one function, not interpolation between the table rows. No orbit
number, curvature, altitude, terrain sample or event timestamp alters its
coefficients. It reaches 600 m/s at 40s algebraically. It approaches 133 m/s;
at 58s it is **137.131 m/s**, not exactly 133.

Fractional braking rate is:

```
r(t) = -v'(t)/v(t) = L*p*y / ((t-t0)*(1+y)^2)
r'(t)/r(t) = ((p-1) - (p+1)*y) / ((t-t0)*(1+y))
```

It has one maximum at t=33.629382s and decreases thereafter. Therefore both
fractional and absolute braking ease off after that maximum; no jitter,
reacceleration or separate late brake is needed to evaluate it.

Measured at 1,920 Hz:
- Peak fractional rate: 0.847776/s.
- Worst 100-ms loss: 8.1282%, versus about 25.4% in the rejected flight.
- Worst 1-second loss: 57.0782%, versus about 94.5% in the rejected flight.
- Peak absolute deceleration: 476,313.4 m/s² near
  27.6578s. This remains fantastical spaceship motion,
  NOT passenger-safe physical acceleration. Smooth derivatives alone do not
  establish visual comfort.
- Independent 5-ms versus 2.5-ms Simpson integration differs by
  5.21540641784668e-7 m across 5–58s.

A 57% one-second reduction may still feel too strong; it must not be described
as visually accepted. This concrete curve exposes the tradeoff for review.
It also changes normalized approach speed: 28s is about 39.9% of exit speed,
not the older 13.26/18 ratio. Timing preservation does not imply preserving
that old speed law.

## 4. Keyframe clock and path distances

Distances below are **arc length travelled**, not automatically axial field
coordinates. Named events are placement constraints on a fixed world. The
renderer must not switch scenes/functions at these times.

| Time, s | Intended observation | Function speed, m/s | Route distance since 5s, km |
|---:|---|---:|---:|
| 5 | Emergence observation | 3,400,613.84 | 0 |
| 7.6 | Circular entry; incomplete spokes | 3,400,613.82 | 8,841.596 |
| 11 | Circular assembly | 3,400,598.55 | 20,403.673 |
| 16 | Circular assembly | 3,397,262.91 | 37,403.096 |
| 20 | Circular assembly | 3,341,692.74 | 50,923.077 |
| 23 | Evolving toward square | 3,087,170.85 | 60,654.62 |
| 26 | Fully squared approach | 2,265,817.64 | 68,873.171 |
| 27 | Planet introduction | 1,830,412.12 | 70,926.445 |
| 28 | Orbital-entry reference | 1,357,536.39 | 72,521.236 |
| 31 | Orbital descent | 285,325.3 | 74,767.103 |
| 34 | Orbital descent | 25,811.24 | 75,103.973 |
| 40 | Aircraft-speed target | 600 | 75,136.861 |
| 58 | Requested ground/manual target | 137.13 | 75,140.567 |

The circular structure is the ENTIRE 7.6–23s interval, including incomplete
irregular spokes, gradual assembly and evolution toward square. It is not
just the late fully connected portion.

The circular interval consumes about 51,813.024 km
of route, while square evolution from 23–26s consumes
8,218.551 km.
The route from 26–28s consumes
3,648.066 km.
The capture's axial length must be solved from its actual bend, not equated to
that last arc length. Keep off-axis maneuvering bounded inside the varying
cross-section; match the exact incoming tangent/curvature, never clip a bad path.

At 5s, an axial first-ring estimate 8,841.6 km ahead with 478.0-km radius projects
to roughly an 8.3-pixel radius at the current focal length: already visible
but distant. Entry then occurs near 7.6s. Initial cadence is about 5.48 rings/s;
the mature 283.384-km pitch gives about 10.89 rings/s at 23s, 8.00 at 26s and
4.79 at 28s. Cadence increases while spacing organizes even though physical
speed never increases; that distinction must be reviewed, not called
acceleration. The onset of squaring itself must not create a new speed increase.

Total post-exit translation is about 75,141 km through 58s. Stars at a minimum
5e11 m distance have a central-field translation estimate of only about
0.023 pixels at focal length 154. Full-frustum and rotated-pose checks must
bound the actual motion (proposed target <0.1 px translational parallax).
Camera rotation must still move the stars normally. The near catalogue and
opening parity have not yet been reconciled; the current shutter-only fix
does not achieve this geometry.

## 5. Orbit-only geometric witness — what it proves and what it fails

For a spherical reference body, the following simple spiral can consume
exactly the function's 2,615.625 km between 28 and 40s:

```
Theta = 6*pi
Rplanet = 81,000 m
Rf = Rplanet + 9,144 m
R0 = 282718.91492265044 m
R(theta) = Rf + (R0-Rf)*(1-theta/Theta)^3
arc(theta) = integral sqrt(R(theta)^2 + (dR/dtheta)^2) dtheta
theta(t) = inverse_arc(integral(v, 28..t))
```

Its inward radial angle at entry is 6.187 degrees;
combined with a 20-degree past-top entry frame the tangent would descend about
26.187 degrees, below the 30-degree requirement.

| Revolution completed | Time, s | Reference-sphere AGL, km | Speed, m/s |
|---:|---:|---:|---:|
| 1 | 29.2006 | 66.203 | 825,633.1 |
| 2 | 30.4068 | 16.276 | 422,339.4 |
| 3 | 40 | 9.144 | 600 |

The first orbit takes about 1.20 s, the second about 1.21 s, and the third
about 9.59 s. This demonstrates a distance/cadence fit is possible at these
dimensions. It does **not** prove the capture, terrain-relative clearance,
full tunnel containment, Sun framing or descent experience.

In fact, this particular cubic spiral must NOT be adopted as the final route:

| Time, s | Revolutions | Reference AGL, m |
|---:|---:|---:|
| 28 | 0 | 201,718.915 |
| 29 | 0.822642 | 82,769.067 |
| 30 | 1.693752 | 25,040.921 |
| 31 | 2.3507 | 11,096.414 |
| 32 | 2.705672 | 9,325.857 |
| 33 | 2.870264 | 9,159.574 |
| 34 | 2.941934 | 9,145.396 |
| 35 | 2.972819 | 9,144.143 |
| 36 | 2.986607 | 9,144.017 |
| 38 | 2.996689 | 9,144 |
| 40 | 3 | 9,144 |

It has nearly finished its height loss by 33s. Counting a third revolution
ending at 40s would conceal the same uninteresting late altitude evolution
the user rejected. Build the final radius/clearance profile against the entire
distance budget and visible-curvature samples, including ongoing descent
through 40s, with no timed altitude law and no added speed. Matching the final
route into subsequent descent must match derivatives; do not graft this
isolated cubic onto another altitude function.

## 6. The incompatible endpoint requirements

Even independently of this candidate, 40s at 9,144 m/600 m/s and 58s at
20 m/133 m/s cannot be combined with a braking tail whose deceleration
gradually relaxes throughout that interval.

If absolute deceleration relaxes, v(t) is convex and lies below its endpoint
chord. Its distance is at most:

```
18 * (600 + 133) / 2 = 6,597 m
```

If fractional braking relaxes, log(v(t)) is convex and the stronger bound is:

```
integral(v, 40..58) <= 18*(600-133)/ln(600/133)
                    = 5,579.523 m
```

The altitude drop alone is 9,124 m. The straight-down lower bound on duration
under the latter endpoint/easing requirements is
29.435 seconds AFTER 40s.
Any forward flight and smooth leveling need more. These are geometric bounds,
not a limitation of Rust or the current integrator.

The concrete candidate above supplies only 3,705.241 m during 40–58s.
At 58s even an impossible instantaneous vertical turn would leave at least
5,438.759 m altitude. Its straight-down ground time is
98.559s; a constant 30-degree descent would take until about
167.159s. Neither is a proposed cinematic schedule, and
a smooth turn/leveling would lengthen them.

Explicit alternatives requiring a user choice:

| Priority preserved | What must change / consequence |
|---|---|
| 40s at 9,144 m and 600 m/s | Surface time must move later. Even the optimal fractional-easing endpoint bound starts at 69.435s for vertical flight; this concrete curve is much slower. Do not silently move it. |
| 40s at 600 m/s and surface near 58s | Lower the 40s altitude. This curve has only 3.705 km of total remaining path; e.g. a 1.5-km AGL target gives useful geometric room, but is NOT 30,000 ft. |
| 40s at 9,144 m and surface near 58s | Increase the 40s speed and redesign the same whole-flight function. Best-case fractional-easing lower bounds are >1,281.211 m/s even vertically, or >3,426.796 m/s at 30 degrees; smooth turns need more. This is NOT Concorde/F-15 speed. |
| Keep both time/altitude endpoints by holding high speed, then braking abruptly | Reject: violates the explicit gradual-tail requirement. |
| Add vertical velocity or a curvature override | Reject: violates the single total-speed function. |

For the lower-altitude alternative only, a 1.5-km-to-20-m descent over this
3.705-km arc with a quintic smoothstep height profile has peak |dh/ds| about
0.749 (48.5 degrees). It is geometrically possible but steep; this is not
proof of comfortable framing or acceptable dynamics. Even lowering altitude
does not automatically make the complete curve satisfactory.

## 7. Execution plan / gates

1. Extraction-only hygiene: fixed geometry/units, path, scalar motion, camera,
   and legacy audit modules. Keep numerical outputs unchanged; record existing
   failures rather than blessing them.
2. Resolve the endpoint choice and review physical planet size. Finalize ONE
   complete function and dimensions. Publish its whole-second percentages and
   subsecond extrema again; keep 7.6–23s and downstream event clocks explicit.
3. Build a time-independent complete P(s), including descent and displaced
   terrain, parameterized by true arc length. Reject the simple spiral above
   for its early altitude flattening. Validate curvature and actual screen
   coverage numerically across 31–40s, not just at endpoints.
4. Build the fixed grid/stars at the selected independent scales. Preserve
   gradual fixed spoke connectivity through widening/squaring; bound visible
   edge cost with stable identities and projection-based culling/filtering.
   Do not trade drawing quality for performance or increase axial cadence
   accidentally when the square widens.
5. Install P(integral(v)) atomically; remove ALL active older speed/altitude/
   angular controllers and curvature speed limits. Independently differentiate
   actual position to verify total velocity, not just the requested scalar.
6. Manual authority begins from the exact current pose/velocity/acceleration
   and camera state at safe near-surface arrival. No renderer/camera replacement.
   Manual bindings/replay are a separate interface decision, not yet implemented.
7. Run complete numerical continuity, scale, cadence, clipping, containment,
   terrain, Sun, actual-speed, pause/seek and performance checks. Full user live
   review remains required. No scene images or visual baselines are generated.

## 8. Complete prospective post-wormhole table

5s speed = 100%. Negative loss would mean reacceleration; there is none.
The 58s row is a speed evaluation at the requested deadline, NOT a claim that
the candidate can have reached the surface then.

| Time, s | Proposed total speed, m/s | % of 5s speed remaining | % lost since preceding second | Distance since 5s, km |
|---:|---:|---:|---:|---:|
| 5 | 3,400,613.84 | 100 | — | 0 |
| 6 | 3,400,613.84 | 100 | <0.0001 | 3,400.614 |
| 7 | 3,400,613.84 | 100 | <0.0001 | 6,801.228 |
| 8 | 3,400,613.79 | 99.999998 | <0.0001 | 10,201.842 |
| 9 | 3,400,613.31 | 99.999984 | <0.0001 | 13,602.455 |
| 10 | 3,400,610.57 | 99.999904 | <0.0001 | 17,003.067 |
| 11 | 3,400,598.55 | 99.99955 | 0.000354 | 20,403.673 |
| 12 | 3,400,555.7 | 99.99829 | 0.00126 | 23,804.255 |
| 13 | 3,400,425.04 | 99.994448 | 0.003842 | 27,204.756 |
| 14 | 3,400,072.38 | 99.984078 | 0.010371 | 30,605.032 |
| 15 | 3,399,209.64 | 99.958707 | 0.025374 | 34,004.734 |
| 16 | 3,397,262.91 | 99.901461 | 0.05727 | 37,403.096 |
| 17 | 3,393,158.42 | 99.780762 | 0.120818 | 40,798.55 |
| 18 | 3,384,993.4 | 99.540658 | 0.240632 | 44,188.073 |
| 19 | 3,369,558.33 | 99.086768 | 0.455985 | 47,566.135 |
| 20 | 3,341,692.74 | 98.267339 | 0.826981 | 50,923.077 |
| 21 | 3,293,512.07 | 96.850517 | 1.441804 | 54,242.787 |
| 22 | 3,213,679.61 | 94.502927 | 2.423931 | 57,499.593 |
| 23 | 3,087,170.85 | 90.782753 | 3.93657 | 60,654.62 |
| 24 | 2,896,411.69 | 85.173202 | 6.179093 | 63,652.509 |
| 25 | 2,625,101.32 | 77.194925 | 9.36712 | 66,420.499 |
| 26 | 2,265,817.64 | 66.629666 | 13.686469 | 68,873.171 |
| 27 | 1,830,412.12 | 53.825933 | 19.216265 | 70,926.445 |
| 28 | 1,357,536.39 | 39.920334 | 25.834385 | 72,521.236 |
| 29 | 907,541.84 | 26.687589 | 33.147881 | 73,649.144 |
| 30 | 539,899.85 | 15.876541 | 40.509647 | 74,364.118 |
| 31 | 285,325.3 | 8.390406 | 47.15218 | 74,767.103 |
| 32 | 135,853.68 | 3.994975 | 52.38639 | 74,970.072 |
| 33 | 60,110.5 | 1.767637 | 55.753496 | 75,063.377 |
| 34 | 25,811.24 | 0.759017 | 57.06035 | 75,103.973 |
| 35 | 11,271.51 | 0.331455 | 56.330988 | 75,121.466 |
| 36 | 5,212.8 | 0.15329 | 53.75247 | 75,129.275 |
| 37 | 2,624.66 | 0.077182 | 49.649707 | 75,133.018 |
| 38 | 1,457.63 | 0.042864 | 44.464143 | 75,134.986 |
| 39 | 893.51 | 0.026275 | 38.701329 | 75,136.129 |
| 40 | 600 | 0.017644 | 32.848775 | 75,136.861 |
| 41 | 436.21 | 0.012827 | 27.2987 | 75,137.372 |
| 42 | 338.9 | 0.009966 | 22.30764 | 75,137.756 |
| 43 | 277.9 | 0.008172 | 17.998709 | 75,138.062 |
| 44 | 237.91 | 0.006996 | 14.39127 | 75,138.318 |
| 45 | 210.7 | 0.006196 | 11.438554 | 75,138.542 |
| 46 | 191.61 | 0.005634 | 9.060333 | 75,138.742 |
| 47 | 177.88 | 0.005231 | 7.165869 | 75,138.927 |
| 48 | 167.79 | 0.004934 | 5.667535 | 75,139.099 |
| 49 | 160.26 | 0.004713 | 4.487503 | 75,139.263 |
| 50 | 154.56 | 0.004545 | 3.560037 | 75,139.42 |
| 51 | 150.18 | 0.004416 | 2.831373 | 75,139.573 |
| 52 | 146.79 | 0.004317 | 2.258442 | 75,139.721 |
| 53 | 144.14 | 0.004239 | 1.80721 | 75,139.867 |
| 54 | 142.05 | 0.004177 | 1.451013 | 75,140.01 |
| 55 | 140.39 | 0.004128 | 1.16907 | 75,140.151 |
| 56 | 139.06 | 0.004089 | 0.945225 | 75,140.29 |
| 57 | 137.99 | 0.004058 | 0.766933 | 75,140.429 |
| 58 | 137.13 | 0.004033 | 0.624449 | 75,140.567 |
