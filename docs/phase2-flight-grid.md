# Phase 2 — evaluated flight grid

Offline numerical proposal, NOT a live flight or visual approval. Phase 2 remains open
for the foreground-star decision and exact opening/render validation.
See [the interpretation and remaining work](phase2-study.md).

Numerical speeds below are fitted choices, **not fixed specifications**. The old
600-m/s airline and 133-m/s surface pair is not imposed. Geometric orbital crossings
are measured from the actual route; field boundaries are positioned at the desired
spatial crossings. Those latter times are not independent optical-visibility proofs.

## One speed function

Metres and seconds. The same expression is evaluated throughout 4–77 s;
the 4–5 s part establishes continuity with the opening, not a second brake.

```text
x = t - 4; w = 0.25
g(t) = x - 1.5*w + 2*w*exp(-x/w) - 0.5*w*exp(-2*x/w)
u = g(t) / 72.625
I_i(u) = sum(j=i+1..25, binomial(25,j)*u^j*(1-u)^(25-j))
v(t) = 70947663.29049851 * exp(-sum(i=0..24, c_i*I_i(u)))
s(t) = integral(v(tau), tau=4..t)
position(t) = P(s(t)); |dP/ds| = 1
```

All coefficients are nonnegative. They are fitted once offline, never selected by
an orbit/event or adjusted while flying. Thus v is positive and monotonically
decreasing analytically; its terminal first/second time derivatives vanish.
The final fractional braking rate also decreases, not just absolute deceleration.

Nonzero coefficients (all others are exactly zero):

| i | c_i |
|---:|---:|
| 0 | 0.283693214667677 |
| 1 | 0.06909552551168413 |
| 2 | 0.3793492384574534 |
| 3 | 0.500621179259363 |
| 7 | 0.7310930544974156 |
| 8 | 0.7099061360059641 |
| 15 | 0.06455428229223392 |
| 16 | 6.189151518944869 |
| 17 | 2.7530592215803567 |

## Selected fixed dimensions

| Quantity | Metres, unless stated |
|---|---:|
| Planet radius / diameter | 6,371,000 / 12,742,000 |
| Reserved relief / atmosphere height | 20,000 / 100,000 |
| Circular radius / diameter | 6,000,000 / 12,000,000 |
| Square width / height | 18,000,000 / 15,603,605 |
| Planet diameter as % of square width | 70.789% |
| Axis height above nominal floor | 7,801,803 |
| Axial cell pitch | 2,000,000 |
| Perimeter lanes | 192 |
| Circular transverse spacing | 196,349.541 |
| Square transverse spacing | 350,037.552 |
| Ring field start Z / end Z | -530,572,987.979 / 19,113,000.000 |
| Field length / ring count | 549,685,987.979 / 276 |
| Well depth / width parameter | 6,371,000 / 9,556,500 |
| Mesh distance fade starts / ends | 180,000,000 / 210,000,000 |
| Proposed distant background stars / minimum depth | 4096 / 10,000,000,000,000 |

The cross-section widens smoothly from a 6,000-km circular radius to half-width
9,000 km and half-height 7,801.803 km between the 20 s and 26 s spatial planes.
Squaring occurs between the 23 s and 26 s planes.
These are fixed functions of world Z, not functions of playback time.
The nominal square floor is Y=0, roof Y=15,603.605 km, planet centre (0,0,0).
The floor well is Y=-R/sqrt(1+(hypot(X,Z)/(1.5*R))^2): it touches the
planet bottom at its centre. The mesh continues beyond the planet.

## Full percentage table

AGL is sampled from the existing production terrain, not a reference sphere.
Whole seconds and fractional geometric events are both included. The loss column
always compares v(t) with v(t-1), never with the previous displayed row.
A dash in the orbit column means the approach precedes orbit entry.

| Time, s | Event | Total m/s | % exit speed | Lost in last 1 s | AGL, m | Route since exit, km | Orbits completed |
|---:|---|---:|---:|---:|---:|---:|---:|
| 5.000000 | Wormhole emergence | 66,965,184.9 | 100.000000% | — | 681,591,827.9 | 0.000 | — |
| 6.000000 | — | 61,812,305.1 | 92.305136% | 7.694864% | 617,271,112.6 | 64,325.264 | — |
| 7.000000 | — | 57,215,760.4 | 85.441055% | 7.436294% | 557,791,026.9 | 123,810.479 | — |
| 7.600000 | Circular-grid entry: incomplete spokes | 54,578,141.7 | 81.502264% | 7.517223% | 524,259,228.9 | 157,345.676 | — |
| 8.000000 | — | 52,849,107.1 | 78.920274% | 7.631906% | 502,776,898.5 | 178,830.418 | — |
| 9.000000 | — | 48,623,727.9 | 72.610459% | 7.995176% | 452,059,520.0 | 229,554.388 | — |
| 10.000000 | — | 44,582,781.8 | 66.576060% | 8.310646% | 405,482,372.3 | 276,139.017 | — |
| 11.000000 | Spoke assembly underway | 40,805,041.5 | 60.934710% | 8.473541% | 362,821,983.5 | 318,807.909 | — |
| 12.000000 | — | 37,353,213.4 | 55.780050% | 8.459318% | 323,781,440.2 | 357,858.153 | — |
| 13.000000 | — | 34,254,821.9 | 51.153181% | 8.294846% | 288,018,181.3 | 393,632.546 | — |
| 14.000000 | — | 31,502,518.6 | 47.043129% | 8.034791% | 255,180,169.5 | 426,483.448 | — |
| 15.000000 | — | 29,063,184.0 | 43.400439% | 7.743300% | 224,936,640.5 | 456,742.066 | — |
| 16.000000 | Circular assembly continues | 26,889,117.9 | 40.153877% | 7.480482% | 196,998,320.7 | 484,698.286 | — |
| 17.000000 | — | 24,928,026.5 | 37.225353% | 7.293253% | 171,126,919.2 | 510,591.256 | — |
| 18.000000 | — | 23,130,657.9 | 34.541319% | 7.210232% | 147,135,812.1 | 534,608.820 | — |
| 19.000000 | — | 21,455,936.1 | 32.040434% | 7.240269% | 124,884,433.2 | 556,893.330 | — |
| 20.000000 | Connected circular structure | 19,873,773.0 | 29.677769% | 7.374011% | 104,268,868.1 | 577,551.412 | — |
| 21.000000 | — | 18,365,820.1 | 27.425923% | 7.587653% | 85,211,015.1 | 596,665.491 | — |
| 22.000000 | — | 16,924,485.9 | 25.273560% | 7.847917% | 67,648,626.9 | 614,305.163 | — |
| 23.000000 | Square evolution begins | 15,550,665.4 | 23.222015% | 8.117354% | 51,530,728.1 | 630,536.906 | — |
| 24.000000 | — | 14,250,748.4 | 21.280832% | 8.359237% | 36,813,346.8 | 645,431.103 | — |
| 25.000000 | — | 13,033,508.1 | 19.463111% | 8.541589% | 23,488,407.1 | 659,065.966 | — |
| 26.000000 | Fully squared grid | 11,907,394.1 | 17.781470% | 8.640145% | 11,696,879.5 | 671,528.521 | — |
| 27.000000 | Planet approach | 10,878,563.2 | 16.245103% | 8.640269% | 2,527,341.7 | 682,913.221 | — |
| 28.000000 | Orbit 1 entry | 9,949,756.6 | 14.858104% | 8.537953% | 320,224.0 | 693,319.031 | — |
| 29.000000 | — | 9,119,937.5 | 13.618924% | 8.340094% | 258,867.4 | 702,845.771 | 0.227641 |
| 30.000000 | — | 8,384,482.9 | 12.520660% | 8.064250% | 210,773.3 | 711,590.396 | 0.438315 |
| 30.307830 | Orbit 1 far-side half-lap | 8,175,962.4 | 12.209273% | 7.967655% | 197,860.3 | 714,139.086 | 0.500000 |
| 31.000000 | — | 7,735,693.0 | 11.551813% | 7.737984% | 172,506.6 | 719,643.652 | 0.633624 |
| 32.000000 | — | 7,163,413.9 | 10.697221% | 7.397904% | 141,772.2 | 727,087.306 | 0.815109 |
| 33.000000 | — | 6,655,642.5 | 9.938959% | 7.088399% | 117,015.1 | 733,991.997 | 0.984164 |
| 33.097328 | Orbit 2 begins | 6,609,178.3 | 9.869574% | 7.061754% | 115,463.2 | 734,637.511 | 1.000000 |
| 33.907784 | 100-km atmosphere boundary | 6,239,397.1 | 9.317374% | 6.876257% | 100,000.0 | 739,842.153 | 1.127859 |
| 34.000000 | — | 6,199,063.5 | 9.257144% | 6.860030% | 98,422.4 | 740,415.664 | 1.141966 |
| 35.000000 | — | 5,779,541.9 | 8.630667% | 6.767499% | 83,367.2 | 746,402.480 | 1.289424 |
| 36.000000 | — | 5,382,654.3 | 8.037989% | 6.867113% | 71,384.5 | 751,982.288 | 1.427144 |
| 36.558959 | Orbit 2 far-side half-lap | 5,165,415.2 | 7.713583% | 7.027042% | 65,458.5 | 754,930.191 | 1.500000 |
| 37.000000 | — | 4,994,367.1 | 7.458155% | 7.213674% | 62,159.4 | 757,170.641 | 1.555412 |
| 38.000000 | — | 4,601,969.2 | 6.872182% | 7.856809% | 54,810.8 | 761,969.624 | 1.674207 |
| 39.000000 | — | 4,195,299.1 | 6.264896% | 8.836873% | 49,048.3 | 766,369.777 | 1.783240 |
| 40.000000 | — | 3,768,188.6 | 5.627086% | 10.180691% | 44,550.4 | 770,353.349 | 1.882030 |
| 41.000000 | — | 3,319,864.7 | 4.957598% | 11.897598% | 40,928.5 | 773,899.012 | 1.970014 |
| 41.373521 | Orbit 3 begins; partial-lap descent | 3,147,882.8 | 4.700775% | 12.633014% | 39,975.9 | 775,106.996 | 2.000000 |
| 42.000000 | — | 2,855,870.6 | 4.264710% | 13.976294% | 38,301.8 | 776,987.783 | 2.046697 |
| 43.000000 | — | 2,387,990.0 | 3.566017% | 16.383118% | 34,788.6 | 779,609.403 | 2.111810 |
| 44.000000 | — | 1,932,786.9 | 2.886256% | 19.062185% | 25,105.8 | 781,767.966 | 2.165476 |
| 45.000000 | — | 1,508,779.8 | 2.253081% | 21.937604% | 16,222.9 | 783,485.394 | 2.208240 |
| 46.000000 | — | 1,132,827.2 | 1.691666% | 24.917659% | 12,934.0 | 784,801.602 | 2.241044 |
| 47.000000 | — | 816,763.7 | 1.219684% | 27.900416% | 12,288.7 | 785,771.092 | 2.265214 |
| 48.000000 | — | 565,363.8 | 0.844265% | 30.779998% | 11,836.9 | 786,456.766 | 2.282311 |
| 49.000000 | — | 376,234.9 | 0.561837% | 33.452601% | 11,608.5 | 786,922.640 | 2.293927 |
| 50.000000 | — | 241,462.0 | 0.360578% | 35.821476% | 11,474.7 | 787,227.387 | 2.301527 |
| 51.000000 | — | 150,188.5 | 0.224278% | 37.800366% | 11,351.4 | 787,420.067 | 2.306331 |
| 52.000000 | — | 91,141.4 | 0.136103% | 39.315305% | 11,156.9 | 787,538.490 | 2.309284 |
| 53.000000 | — | 54,406.8 | 0.081246% | 40.305074% | 11,009.8 | 787,609.762 | 2.311062 |
| 54.000000 | — | 32,251.8 | 0.048162% | 40.720951% | 10,940.2 | 787,652.138 | 2.312118 |
| 55.000000 | — | 19,181.3 | 0.028644% | 40.526551% | 10,893.8 | 787,677.273 | 2.312745 |
| 56.000000 | — | 11,566.6 | 0.017273% | 39.698587% | 10,571.2 | 787,692.304 | 2.313120 |
| 57.000000 | Airline height: terrain crossing | 7,144.8 | 0.010669% | 38.229207% | 9,144.0 | 787,701.461 | 2.313345 |
| 58.000000 | — | 4,563.3 | 0.006815% | 36.130235% | 7,161.0 | 787,707.201 | 2.313480 |
| 59.000000 | — | 3,037.4 | 0.004536% | 33.439016% | 5,479.0 | 787,710.935 | 2.313563 |
| 60.000000 | — | 2,119.4 | 0.003165% | 30.224826% | 4,242.3 | 787,713.476 | 2.313618 |
| 61.000000 | — | 1,555.7 | 0.002323% | 26.593847% | 3,349.2 | 787,715.291 | 2.313658 |
| 62.000000 | — | 1,202.7 | 0.001796% | 22.690081% | 2,688.6 | 787,716.657 | 2.313688 |
| 63.000000 | — | 978.0 | 0.001460% | 18.689511% | 2,180.1 | 787,717.739 | 2.313711 |
| 64.000000 | — | 833.4 | 0.001244% | 14.785990% | 2,178.9 | 787,718.639 | 2.313732 |
| 65.000000 | — | 740.3 | 0.001105% | 11.169636% | 1,841.2 | 787,719.423 | 2.313749 |
| 66.000000 | — | 681.0 | 0.001017% | 8.001520% | 1,550.8 | 787,720.131 | 2.313765 |
| 67.000000 | — | 644.3 | 0.000962% | 5.390752% | 1,294.9 | 787,720.792 | 2.313781 |
| 68.000000 | — | 622.5 | 0.000930% | 3.380416% | 1,065.9 | 787,721.425 | 2.313795 |
| 69.000000 | — | 610.4 | 0.000912% | 1.946573% | 677.4 | 787,722.041 | 2.313810 |
| 70.000000 | — | 604.3 | 0.000902% | 1.010567% | 674.4 | 787,722.648 | 2.313824 |
| 71.000000 | — | 601.5 | 0.000898% | 0.460762% | 509.9 | 787,723.250 | 2.313839 |
| 72.000000 | — | 600.4 | 0.000897% | 0.177419% | 367.6 | 787,723.851 | 2.313853 |
| 73.000000 | — | 600.1 | 0.000896% | 0.054226% | 248.5 | 787,724.451 | 2.313868 |
| 74.000000 | — | 600.0 | 0.000896% | 0.011835% | 154.0 | 787,725.051 | 2.313883 |
| 75.000000 | — | 600.0 | 0.000896% | 0.001510% | 85.5 | 787,725.651 | 2.313898 |
| 76.000000 | — | 600.0 | 0.000896% | 0.000072% | 43.9 | 787,726.251 | 2.313913 |
| 77.000000 | Canyon arrival; proposed manual takeover | 600.0 | 0.000896% | 0.000000% | 30.0 | 787,726.851 | 2.313928 |

## Event timing comparison

| Event | Target, s | Evaluated, s | Difference, s |
|---|---:|---:|---:|
| Wormhole emergence | 5.000 | 5.000000 | 0.000000 |
| Circular-grid entry: incomplete spokes | 7.600 | 7.600000 | 0.000000 |
| Spoke assembly underway | 11.000 | 11.000000 | 0.000000 |
| Circular assembly continues | 16.000 | 16.000000 | 0.000000 |
| Connected circular structure | 20.000 | 20.000000 | 0.000000 |
| Square evolution begins | 23.000 | 23.000000 | 0.000000 |
| Fully squared grid | 26.000 | 26.000000 | 0.000000 |
| Planet approach | 27.000 | 27.000000 | 0.000000 |
| Orbit 1 entry | 28.000 | 28.000000 | -0.000000 |
| Orbit 1 far-side half-lap | Derived | 30.307830 | — |
| Orbit 2 begins | 33.000 | 33.097328 | 0.097328 |
| Orbit 2 far-side half-lap | Derived | 36.558959 | — |
| Orbit 3 begins; partial-lap descent | 41.419 | 41.373521 | -0.045479 |
| Airline height: terrain crossing | 57.000 | 57.000000 | 0.000000 |
| Canyon arrival; proposed manual takeover | 77.000 | 77.000000 | 0.000000 |
| 100-km atmosphere boundary | Derived | 33.907784 | — |

## Grid cadence and reference projections

The pixel calculations below use a 154-pixel focal length and an axis-aligned
reference ring. Inside the grid the reference edge is 100 pixels from screen
centre; at emergence the actual first-ring depth is used. They are NOT a full
camera/vertex rendering, clipping, aliasing or visibility acceptance test.

| Time, s | Fixed axial Z, km | Rings passed/s | Reference diameter, px | Edge speed, px/s | Axial pitch at 100-px edge, px |
|---:|---:|---:|---:|---:|---:|
| 5.0 | -687,918.664 | 33.483 | 11.745 | 2.499 | 21.645 |
| 7.6 | -530,572.988 | 27.289 | 200.000 | 590.673 | 21.645 |
| 11.0 | -369,110.755 | 20.403 | 200.000 | 441.613 | 21.645 |
| 16.0 | -203,220.378 | 13.445 | 200.000 | 291.008 | 21.645 |
| 20.0 | -110,367.254 | 9.937 | 200.000 | 215.084 | 21.645 |
| 23.0 | -57,383.357 | 7.775 | 200.000 | 141.937 | 18.255 |
| 26.0 | -16,397.233 | 5.953 | 200.000 | 99.101 | 16.646 |
| 27.0 | -5,015.508 | 5.434 | 200.000 | 90.455 | 16.646 |
| 28.0 | 4,775.190 | 3.461 | 200.000 | 57.609 | 16.646 |

Cell-pitch alternatives hold the other dimensions and the same route/speed fixed:

| Pitch, km | Reduction from old physical pitch | Ring passages/s at entry | Rings in visibility diameter |
|---:|---:|---:|---:|
| 200 | 1,416.922x | 272.891 | 2102 |
| 1,000 | 283.384x | 54.578 | 422 |
| 2,000 | 141.692x | 27.289 | 212 |

Even the selected 2,000-km pitch crosses more than 15 rings/s early in the
cylinder. A 30-Hz renderer therefore needs measured temporal integration/alias
control, not skipped spokes or a new velocity cap. At most 212 ring ordinals
fit inside the selected visibility diameter: about 81,408 candidate ring/rail
edges at 192 lanes, before clipping, assembly and footprint filtering.
A conservative 520 raster steps per candidate edge is about 42.3 million
steps/frame, not a measured frame time. No 33.333-ms performance pass is claimed.

## Route, clearance and numerical checks

- First-lap arc: 41,318.480 km; second-lap arc: 40,469.485 km.
- Final partial lap: 12,619.856 km; surface reached after 2.313928 total laps, not 3.
- Final 20 s provides 25.391 km of actual route length.
- 140,161 motion samples at 1920 Hz; 229,210 lower-route terrain samples.
- Minimum sampled AGL 30.000000 m; the minimum safety requirement remains 20 m.
- Minimum roof / side clearance: 8,015.522 / 8,997.931 km.
- Minimum incoming circular/widening clearance: 4,687.221 km.
- Signed incoming curvature minimum: 8.3016210e-16 /m; maximum upward incoming angle -0.000457523 degrees. No upward approach leg.
- Below-well excursions stay within 2,308.776 km of the planet axis; allowed localized footprint 6,721.000 km.
- Position-derived speed/vector residual: 0.00005127%.
- Arc quadrature difference: 0.000489712 m; refinement difference -0.000469327 m.
- Independent scalar quadrature difference: -0.000001907 m.

| Dense maximum | Value | Time, s |
|---|---:|---:|
| positionDerivativeRelativeError | 5.127004094e-7 | 71.273958 |
| outwardApproachVelocityMps | -67302.09919 | 28.000000 |
| fractionalBrakingPerSecond | 0.5235328939 | 53.690625 |
| scalarDecelerationMps2 | 5527659.715 | 5.000000 |
| scalarJerkMps3 | 841384.7208 | 5.331771 |
| vectorAccelerationMps2 | 14899518.54 | 27.906771 |
| vectorJerkMps3 | 33952611.59 | 27.605729 |
| curvaturePerMetre | 0.00007728805390 | 76.992708 |
| loss100msPercent | 5.100604929 | 53.740104 |
| loss1sPercent | 40.73187706 | 54.188542 |
| tailFractionalRateDerivative | 0.000000000 | 77.000000 |

These are cinematic accelerations, not passenger-safe physical dynamics.
Smoothness alone does not establish perceptual comfort; live review is still required.

## Proposed camera and fixed Sun

The selected fixed lateral excursion reaches 203.103 km;
its compact C3 spatial support ends before close orbit. The proposed large exit
bends are screening alternatives only and are NOT in this selected route.
The first-ring centre projects to (160.000, 99.992) at 5 s.
The fixed Sun direction is (0, -0.6536436208636119, -0.7568024953079282).
The following visibility windows test the camera frustum and spherical-core
occlusion, not the production terrain/cloud/atmosphere renderer:

| Sun window starts, s | Ends, s |
|---:|---:|
| 28.950 | 29.317 |
| 34.617 | 35.167 |
| 46.350 | 51.867 |
| 62.533 | 77.000 |

Maximum sampled camera forward rotation is 83.879 degrees/s at 27.883 s.
That is a reported measurement, NOT a perceptual-comfort approval.
The reference horizon is at row 89.231 of 200 at orbit entry
and row 67.266 at airline height.

## Opening and stars — unresolved

The proposed static opening similarity is 0.020863193104551644.
A common similarity of the opening camera and star objects preserves ideal
pinhole projections. The fixed route matches the transformed 4 s position and
tangent; its opening normal acceleration/jerk vanish analytically. Measured
residuals are 6.245508423340794e-9 m/s² and 3.301383768745921e-9 m/s³.
This matches opening position, velocity and acceleration. It is not a claim
of matching the protected rush's one-sided third derivative: that opening
already ends a quintic displacement at 4 s with a jerk discontinuity.
The first grid is 226,784.897 km away at 4 s and 157,345.676 km at 5 s:
the proposed spatial visibility limit excludes it through 4 s and admits it at exit.

However, 2122 existing stars project inside both protected 3.95 s and 4 s views.
1721 remain in the reference view at circular entry; 1721 pass behind the
camera by 26 s after the common rescale. Retaining those objects has NOT solved
the unwanted nearby-star rush. Moving them independently to background depth
invalidates the opening similarity argument. No opening pixels or star objects
were changed. This is a real unresolved design/integration issue, not a passed test.

The NEW distant background alone has translation <=0.000078773 rad
(approximately 0.012131 pixels near screen centre) across the entire post-exit route.
That bound does not apply to the preserved foreground population.

## Remaining gates

- Exact opening raster parity with a revised foreground-star catalogue has NOT been proved. A similarity transform preserves ideal projections but cannot move those same stars independently farther away.
- Projected cadence figures are reference-ring calculations, not a rendered motion/aliasing pass. The 33.333-ms live raster budget is unmeasured for these dimensions.
- Terrain clearance is densely sampled against the existing terrain and a 20-km envelope; an interval bound and collision-safe continuation into manual flight remain integration gates.
- The selected route includes a compact C3 lateral excursion. The proposed camera and fixed Sun have numerical framing checks; production projection, lighting/cloud occlusion and uninterrupted playback remain unverified.

The live production binary and its known seven failing gates are unchanged.
Reproduction: `make audit-build`, `node --test scripts/test-phase2-flight-study.cjs`,
then `node scripts/phase2-flight-study.cjs --markdown` (or omit the flag for JSON).
