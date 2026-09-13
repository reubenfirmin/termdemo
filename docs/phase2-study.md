# Phase 2 — coupled flight study, 2026-09-12

Status: the user approved retained unblurred foreground points, and the reviewed
candidate is now connected in the [normal phase-3 build](phase3-integration.md).
The user rejected the separate preview workflow. Acceptance is incomplete: the new
exact-opening RGB check found old grid pixels in the last approximately 0.02 s
before 4 s that cannot remain identical when that visible geometry is resized.
That separate permission question remains open, along with frame-time failures.
The numerical proposal below remains the reference, not a declaration of visual approval.

**There are no fixed numerical speed specifications.** Exit, orbital,
airline-height and surface speeds are fitted choices. The old 600/133-m/s
feasibility objection is withdrawn. The selected speed function below is
unchanged by this pass; the geometry and framing checks have improved.

## Selected proposal

| Event | Evaluated time | Selected total speed |
|---|---:|---:|
| Wormhole emergence | 5 s | 66,965,185 m/s |
| Incomplete circular-grid entry | 7.6 s | 54,578,142 m/s |
| Square evolution begins | 23 s | 15,550,665 m/s |
| Fully squared grid | 26 s | 11,907,394 m/s |
| Close orbit 1 entry | 28 s | 9,949,757 m/s |
| Orbit 2 begins | 33.097 s | About 6.61 million m/s |
| Orbit 3 begins, partial-lap descent | 41.374 s | About 3.15 million m/s |
| 9,144 m above the actual terrain | 57 s | 7,144.8 m/s |
| Canyon arrival, approximately 30 m AGL | 77 s | 600 m/s |

Orbit 1 lasts 5.097 s, orbit 2 lasts 8.276 s, and the route ends after
2.3139 laps. No third lap is completed. The last 20 seconds cover 25.391 km,
including descent. The airline-height speed is cinematic, not Concorde speed.

[The complete generated grid](phase2-flight-grid.md) supplies each second,
fractional milestones, percentages, AGL, orbit phase, dimensions, coefficients
and maxima. Machine-readable results are in `target/phase2-study/result.json`.

About 85% of exit speed is lost before orbit entry. Maximum one-second
fractional loss remains 40.732%, ending at 54.189 s. The final fractional braking
rate decreases toward zero. These measurements are not visual acceptance of
braking or passenger-comfort claims.

## Corrected actual-radius geometry

The earlier proposal used a decreasing radial coefficient and then subtracted
the incoming distance from Z. That did **not** guarantee decreasing distance
from the planet: it created a perigee followed by a climb near orbit entry.
The old no-upward-Y check missed it. A negative witness now explicitly rejects
that old formula.

The corrected single spatial evaluator uses metres and f64:

```text
theta > 0; phi = theta - 0.8
I(theta) = 1,000,000 * exp(-4*theta) / theta
H(theta) = 300,000 * exp(-b*phi - c*phi^2)
d = theta_end - theta
clearance(theta) = 30 + H(theta)*(1-exp(-(d/sigma)^2))
terrain_envelope(theta) = 20,000
  + (ground - ground_slope*d - 20,000)*exp(-d^4)
B(theta) = 6,371,000 + terrain_envelope(theta) + clearance(theta)
A(theta) = I(theta)*sin(theta)
r(theta) = A(theta) + sqrt(B(theta)^2 + A(theta)^2)
P_raw(theta) = (0, r*cos(theta), r*sin(theta)-I(theta))
```

Thus `|P_raw|^2 = B^2 + I^2`: the incoming axial term can no longer conceal an
altitude rebound. b=0.14910784101520966, c=0.005309956078885768.
theta_end=15.333530429250654; sigma=0.0026683618430697518.
The final terrain datum is 45.984012228012084 m and its local slope is
777.886748302592 m/radian.

A small fixed normal correction
`N*(a2*q^2+a3*q^3)*exp(-q/0.001)`, q=theta-theta4, matches the opening's
position, tangent, normal acceleration and normal jerk. The coefficients include
the exponential's derivative at zero. The earlier quartic-exponential cutoff
introduced a small reverse-curvature lobe after the actual-radius correction;
the current kernel removes that lobe without loosening the check.

The complete corrected route has no outward radial velocity from 5–28 s.
At entry its radial velocity is -67,302 m/s, about 0.39 degrees below the local
tangent at the selected total speed. Reference-sphere entry altitude is now
320.224 km, compared with the rejected proposal's 283.573 km.

The opening's original quintic rush already has a one-sided jerk discontinuity
at 4 s. This proposal matches position, velocity and acceleration; it does not
claim that the protected rush was C3 or that revised opening pixels are proved.

The arc metric is integrated and inverted. Time supplies only the integral
of one global positive-density Bernstein log-speed function; it does not
independently choose altitude, orbital phase or lateral velocity. Orbit progress
uses the actual unwrapped `atan2(P.z,P.y)`, not theta divided by a turn.

## Off-centre flight, world dimensions and framing

The selected route adds one fixed lateral excursion in the late cylinder and
square approach. It reaches about 203.103 km, within the tunnel. A compact
seventh-degree spatial cutoff removes it through third derivative before
Z=-10,000 km. This matters: an unbounded Gaussian alone left a small repeat
whenever later orbits revisited the same axial plane. No exit bend is selected.

The corrected asymptote raises the proposed tunnel axis to 7,801.803 km and
square height to 15,603.605 km. The nominal floor remains Y=0, the Earth-sized
planet remains centred at (0,0,0), and the width remains 18,000 km. The circular
diameter is 12,000 km. These are proposed fixed-world dimensions, not runtime
rescaling or screen placement.

Axial pitch remains 2,000 km, independent of star spacing and cross-section:
141.69 times smaller than the old physical pitch. The field has approximately
276 rings and 192 perimeter lanes, continuing beyond the planet. Spatial
assembly/widening/squaring and stable object/edge identities remain required.
The reference ring centre is visible at emergence under the proposed distance
limit and projects to approximately (160,100).

The localized floor well remains
`Y=-R/sqrt(1+(hypot(X,Z)/(1.5*R))^2)`. It touches the planet's bottom and permits
only the localized under-planet excursion. Measured minimum roof clearance is
8,015.522 km, side clearance 8,997.931 km, and incoming circular/widening clearance
4,687.221 km. Below-well excursions remain within 2,308.776 km of the planet axis.

Camera framing is a continuous geometric look toward the horizon, increasingly
downward as altitude falls. At orbit entry the reference horizon lies at row
89.231 of 200; at airline height it lies at row 67.266. This keeps space above
the limb while emphasizing terrain later.

The one fixed Sun direction is (0,-0.6536436208636119,-0.7568024953079282).
Numerical camera/frustum and spherical-core occlusion checks give early Sun
windows at 28.950–29.317 s and 34.617–35.167 s: both early orbits contain a transit.
Production cloud/terrain/atmosphere occlusion still needs checking.

Correcting the true-radius rebound lowers maximum sampled camera-forward
rotation from 111.377 to 83.879 degrees/s, at 27.883 s. This is a measured
improvement, **not** an assertion that the bank now looks comfortable.

## Same terrain and opening — no substitutions

The study queries the production `flow_surface_sample` through the audit-only
G protocol; no terrain is ported to JavaScript, flattened, replaced or imaged.
The selected orientation ends in the existing downstream canyon near chart X=35.

An earlier quartic final-clearance taper struck a ridge. The selected quadratic
exponential taper distributes that descent earlier. 229,210
lower-route terrain samples give approximately 30 m minimum AGL, above the
20-m safety requirement. This is dense sampling, not an interval collision proof.

The basic Earth rescale preserves terrain angular identity and physical relief
metres. The legacy value 13,500 is **chart units per radian**, not the new metric
planet radius. Phase 3 must separate those concepts; blindly changing the old
chart formula invalidates the terrain check. Phase 4 may develop new geography
but must respect the reviewed clearance corridor or explicitly report conflicts.

For protected opening precision, retain the original native camera/star
coordinates and use one fixed global conversion of v(4)/18 metres/native unit.
Install all new geometry with one fixed transform in that same world. This is
an implementation avenue, not a proved renderer comparison. The original camera's
right-vector length is about 1.00169575; its projection calibration must not be
silently normalized away.

The first ring is approximately 226,785 km away at 4 s and 157,346 km at 5 s.
A 210,000-km mesh visibility limit excludes it through the protected opening.
The proposed 4,096 distant background stars have tiny translational parallax,
but do not remove preserved nearby stars. See [the full-catalogue review](phase2-opening-review.md).

## Verification and next action

Twelve algebra/numerical tests and nine candidate numerical gates pass.
The candidate is sampled at 1,920 Hz across 140,161 states. Independent
position differentiation differs from reported velocity by at most about
0.0000513%; independently refined arc calculations agree within 0.001 m.

At the end of the offline phase-2 work, `make check build audit-extraction
audit-harness audit-study` passed. The then-unchanged normal binary had this hash
(historical reference, **not the current phase-3 executable**):

```text
e5a23b0119fa4898189fd8422ecf29db00b2cbfcd3e4bbb8579efe8fb39bdc30
```

The 490-declaration extraction check and ten audit-harness tests still pass.
No image baseline or bitmap was generated. The live flight's seven previously
known failing gates have not been waived or presented as fixed.

For the next integration comparison, the unchanged normal executable is saved
at `/tmp/termdemo-phase3.yQyfb8/normal-before`; the audit executable with both
the shared-terrain and complete-catalogue queries is saved alongside it as
`audit-with-catalogue`. Use actual normal-render bytes for protected-opening
parity; do not replace that test with the study's projection approximation.

**Current next action:** resolve the grid-only late-opening pixel conflict and
the measured phase-3 performance/visual failures in [the integration status](phase3-integration.md).
Foreground-point visibility is approved; no large exit detour is needed.

Phase 3 must remove the competing production controllers, install the reviewed
dimensions and P(s(t)) together, verify actual camera motion, clipping, grid
cadence/aliasing, terrain clearance, performance and the eventual manual-flight
initial state. The current surface endpoint still has nonzero curvature; manual
control must inherit angular motion and acceleration rather than zero them.
Phase 4 supplies the detailed fractal planet needed for visual descent acceptance.

Reproduce with `make study-flight` and `make study-opening`.
