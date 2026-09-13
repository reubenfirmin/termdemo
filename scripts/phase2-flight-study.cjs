// Offline phase-2 study. Read-only queries of shared production terrain/stars;
// no live motion changes, second renderer, assets or file writes.
// node scripts/phase2-flight-study.cjs [--markdown]
'use strict';
const assert = require('node:assert/strict');
const { spawnSync } = require('node:child_process');

const TAU = 2 * Math.PI;
const R = 6371000;
const RELIEF = 20000; // Conservative reserved envelope, not a new terrain surface.
const START = 4;
const EXIT = 5;
const ENTRY = 28;
const ENTRY_THETA = .8;
const THIRD = 41.419;

function integral(f, a, b, step = .01) {
  if (b < a) return -integral(f, b, a, step);
  if (a === b) return 0;
  const n = Math.max(2, 2 * Math.ceil((b - a) / step / 2));
  const h = (b - a) / n;
  let sum = f(a) + f(b);
  for (let i = 1; i < n; i++) sum += (i % 2 ? 4 : 2) * f(a + i * h);
  return sum * h / 3;
}
function root(f, a, b) {
  let fa = f(a);
  assert(fa * f(b) <= 0, `unbracketed root ${a}, ${b}`);
  for (let i = 0; i < 60; i++) {
    const m = (a + b) / 2, fm = f(m);
    if (fa * fm <= 0) b = m; else { a = m; fa = fm; }
  }
  return (a + b) / 2;
}
function onset(t) {
  const x = t - START, w = .25;
  return x - 1.5 * w + 2 * w * Math.exp(-x / w) - .5 * w * Math.exp(-2 * x / w);
}
function onsetRate(t) { return (1 - Math.exp(-(t - START) / .25)) ** 2; }
function binomial(n, k) {
  let a = 1;
  for (let j = 1; j <= k; j++) a = a * (n + 1 - j) / j;
  return a;
}
function bernstein(n, k, u) { return binomial(n, k) * u ** k * (1 - u) ** (n - k); }
function basisValues(n, u) {
  const values = Array(n + 1).fill(0);
  if (u <= .5) {
    values[0] = (1 - u) ** n;
    for (let i = 1; i <= n; i++) values[i] = values[i - 1] * (n - i + 1) / i * u / (1 - u);
  } else {
    values[n] = u ** n;
    for (let i = n; i > 0; i--) values[i - 1] = values[i] * i / (n - i + 1) * (1 - u) / u;
  }
  return values;
}
function basisCDF(n, k, u) {
  let s = 0;
  for (let j = k + 1; j <= n + 1; j++) s += bernstein(n + 1, j, u);
  return s;
}
function linearSolve(matrix, vector) {
  const a = matrix.map((row, i) => row.concat(vector[i])), n = vector.length;
  for (let k = 0; k < n; k++) {
    let pivot = k;
    for (let i = k + 1; i < n; i++) if (Math.abs(a[i][k]) > Math.abs(a[pivot][k])) pivot = i;
    [a[k], a[pivot]] = [a[pivot], a[k]];
    assert(Math.abs(a[k][k]) > 1e-15, 'singular fit');
    for (let i = k + 1; i < n; i++) {
      const ratio = a[i][k] / a[k][k];
      for (let j = k; j <= n; j++) a[i][j] -= ratio * a[k][j];
    }
  }
  const x = Array(n).fill(0);
  for (let i = n - 1; i >= 0; i--) {
    x[i] = (a[i][n] - a[i].slice(i + 1, n).reduce((s, v, j) => s + v * x[j + i + 1], 0)) / a[i][i];
  }
  return x;
}
function nnls(rows, n) {
  const gram = Array.from({ length: n }, (_, i) => Array.from({ length: n }, (_, j) =>
    rows.reduce((s, r) => s + r.a[i] * r.a[j], i === j ? 1e-10 : 0)));
  const rhs = Array.from({ length: n }, (_, i) => rows.reduce((s, r) => s + r.a[i] * r.b, 0));
  let x = Array(n).fill(0), active = [];
  for (let iteration = 0; iteration < 1000; iteration++) {
    const gradient = rhs.map((b, i) => b - gram[i].reduce((s, v, j) => s + v * x[j], 0));
    const candidates = gradient.map((g, i) => [g, i]).filter(([g, i]) => !active.includes(i) && g > 1e-9);
    if (!candidates.length) return x;
    candidates.sort((a, b) => b[0] - a[0]);
    active.push(candidates[0][1]);
    for (;;) {
      const solution = linearSolve(active.map(i => active.map(j => gram[i][j])), active.map(i => rhs[i]));
      const z = Array(n).fill(0);
      active.forEach((i, j) => { z[i] = solution[j]; });
      if (active.every(i => z[i] > 0)) { x = z; break; }
      const alpha = Math.min(...active.filter(i => z[i] <= 0).map(i => x[i] / (x[i] - z[i])));
      x = x.map((v, i) => v + alpha * (z[i] - v));
      active = active.filter(i => x[i] > 1e-10);
    }
  }
  throw Error('NNLS failed to converge');
}
function baselineSpeed(t) {
  return 133 + 67323107.54277262 * Math.exp(-.08269318894118219 * (onset(t) - onset(5)));
}

// Positive coefficients of ONE global log-speed density. Coefficients are
// fitted once offline; no runtime milestone chooses a law or updates a value.
// Last two coefficients are zero, so both braking rate and its derivative
// vanish at the terminal endpoint. Onset does the same at the opening boundary.
function fitSpeed({ end = 77, terminal = 600, degree = 24, air = 57, airSpeed = 7000,
  openingSpeed = baselineSpeed(4) } = {}) {
  const v4 = openingSpeed, span = onset(end), L = Math.log(v4 / terminal);
  assert(v4 > terminal && terminal > 0, 'positive descending speed endpoints');
  const indices = Array.from({ length: degree - 1 }, (_, i) => i);
  const targets = [5, 7.6, 10, 15, 20, 23, 26, 28, 30, 33, 36, 39, THIRD]
    .map(t => [t, baselineSpeed(t), t >= 28 ? 4 : 2]);
  targets.push([air, airSpeed, 3], [end, terminal, 100]);
  const rows = targets.map(([t, v, weight]) => ({
    a: indices.map(i => basisCDF(degree, i, onset(t) / span) * weight),
    b: Math.log(v4 / v) * weight,
  }));
  // A small smoothness preference resolves the underdetermined offline fit.
  // It is not a speed/curvature limiter and does not run in the live demo.
  for (let i = 1; i < indices.length - 1; i++) {
    const a = indices.map(() => 0);
    a[i - 1] = .015; a[i] = -.03; a[i + 1] = .015;
    rows.push({ a, b: 0 });
  }
  const q = nnls(rows, indices.length);
  const scale = L / q.reduce((a, b) => a + b, 0);
  const coefficients = q.map(x => x * scale).concat([0, 0]);
  const speed = t => {
    assert(t >= START && t <= end);
    const u = onset(t) / span;
    const basis = basisValues(degree + 1, u);
    let cdf = 0, exponent = 0;
    for (let i = degree; i >= 0; i--) { cdf += basis[i + 1]; exponent += coefficients[i] * cdf; }
    return v4 * Math.exp(-exponent);
  };
  const rate = t => {
    const u = onset(t) / span;
    return (degree + 1) * onsetRate(t) / span * dot(coefficients, basisValues(degree, u));
  };
  const rateDerivative = t => {
    const u = onset(t) / span, g1 = onsetRate(t), e = Math.exp(-(t - START) / .25), g2 = 8 * e * (1 - e);
    const density = (degree + 1) * dot(coefficients, basisValues(degree, u));
    const derivative = degree * (degree + 1) * dot(coefficients.slice(1).map((x, i) => x - coefficients[i]), basisValues(degree - 1, u));
    return derivative * (g1 / span) ** 2 + density * g2 / span;
  };
  return { end, terminal, degree, air, airSpeed, v4, span, coefficients, speed, rate, rateDerivative,
    residuals: targets.map(([t, target]) => ({ t, target, actual: speed(t), percent: 100 * (speed(t) / target - 1) })) };
}

function surfaceQuery(normals) {
  const input = Buffer.alloc(16 + normals.length * 24);
  input[0] = 'G'.charCodeAt(0);
  normals.forEach((n, i) => n.forEach((x, j) => input.writeDoubleLE(x, 16 + i * 24 + j * 8)));
  const run = spawnSync('target/phase0-audit/x86_64-unknown-linux-gnu/release/termdemo', [],
    { input, maxBuffer: 64 * 1024 * 1024, timeout: 60000 });
  assert.equal(run.error, undefined, String(run.error));
  assert.equal(run.status, 0, run.stderr.toString());
  assert.equal(run.stdout.subarray(0, 8).toString(), 'TDSURF01');
  assert.equal(run.stdout.length, 80 + normals.length * 32);
  return { frame: Array.from({ length: 3 }, (_, i) => Array.from({ length: 3 }, (_, j) =>
    run.stdout.readDoubleLE(8 + i * 24 + j * 8))),
  samples: normals.map((_, i) => Array.from({ length: 4 }, (_, j) => run.stdout.readDoubleLE(80 + i * 32 + j * 8))) };
}
function openingStarWitness(result) {
  const run = spawnSync('target/phase0-audit/x86_64-unknown-linux-gnu/release/termdemo', [],
    { input: Buffer.from('H'), maxBuffer: 16 * 1024 * 1024, timeout: 60000 });
  assert.equal(run.error, undefined, String(run.error));
  assert.equal(run.status, 0, run.stderr.toString());
  const starProtocol = run.stdout.subarray(0, 8).toString();
  assert(['TDSTAR01', 'TDSTAR02'].includes(starProtocol));
  assert.equal((run.stdout.length - 8) % 72, 0);
  const start = result.boundary.at4.p, forward = result.boundary.at4.tangent, down = result.route.opening.normal;
  const right = [1, 0, 0], scale = starProtocol === 'TDSTAR02' ? 1 : result.layout.foregroundOpeningSimilarity;
  const row = t => result.rows.find(r => r.t === t);
  const a = [0, row(7.6).vertical, row(7.6).axial], b = [0, row(26).vertical, row(26).axial];
  function project(point, camera) {
    const q = add(point, mul(camera, -1)), depth = dot(q, forward);
    if (depth <= 0) return null;
    return [160 + 154 * dot(q, right) / depth, 100 + 154 * dot(q, down) / depth];
  }
  let retained = 0, visibleAtEntry = 0, moving = 0, crossedBehind = 0, largest = null;
  for (let offset = 8; offset < run.stdout.length; offset += 72) {
    const r = Array.from({ length: 9 }, (_, j) => run.stdout.readDoubleLE(offset + 8 * j));
    const point = add(start, mul(add(add(mul(right, r[2]), mul(down, r[3])), mul(forward, r[4])), scale));
    retained++;
    const pa = project(point, a), pb = project(point, b);
    if (!pa || pa[0] < 0 || pa[0] >= 320 || pa[1] < 0 || pa[1] >= 200) continue;
    visibleAtEntry++;
    if (!pb) { crossedBehind++; continue; }
    const pixels = Math.hypot(pa[0] - pb[0], pa[1] - pb[1]);
    if (pixels > 1) moving++;
    if (!largest || pixels > largest.translationPixels) largest = { cell: r[0], object: r[1],
      translationPixels: pixels, oldPixelAt395: r.slice(5, 7), oldPixelAt4: r.slice(7, 9),
      newPixelAt76: pa, newPixelAt26: pb, distanceAt76: Math.hypot(...add(point, mul(a, -1))) };
  }
  return { retainedFromBothProtectedViews: retained, visibleAtEntry, movingMoreThanOnePixel: moving,
    passedBehindCameraBefore26: crossedBehind, largest,
    conclusion: 'Keeping opening stars under a static similarity does not make the foreground population distant or static. No star positions, visibility rules, or opening pixels were changed by this study.' };
}

// One analytic incoming-and-orbital curve. theta approaches zero at the
// distant straight incoming asymptote, then increases continuously into orbit.
// No joins at orbit entry; no independently timed altitude or angular law.
function referenceRoute() {
  const incomingLength = 1000000, incomingDecay = 4;
  const H = 300000;
  const l1 = Math.log(H / 95324);
  const l2 = Math.log(H / 19916);
  const c = (l2 - 2 * l1) / (2 * TAU * TAU), b = (l1 - c * TAU * TAU) / TAU;
  const height = theta => H * Math.exp(-b * (theta - ENTRY_THETA) - c * (theta - ENTRY_THETA) ** 2);
  const dh = theta => height(theta) * (-b - 2 * c * (theta - ENTRY_THETA));
  const closeAxis = R + RELIEF + 30 + height(0);
  const axis = incomingLength + Math.hypot(closeAxis,incomingLength);
  return { axis, incomingLength, incomingDecay, H, b, c, height, dh };
}

const add = (a, b) => a.map((x, i) => x + b[i]);
const mul = (a, s) => a.map(x => x * s);
const dot = (a, b) => a.reduce((s, x, i) => s + x * b[i], 0);
const unit = a => mul(a, 1 / Math.hypot(...a));
function adaptive(f, a, b, tolerance = .001, depth = 30) {
  if (a === b) return 0;
  const mid = (a + b) / 2;
  const whole = (b - a) / 6 * (f(a) + 4 * f(mid) + f(b));
  function recurse(a, b, fa, fm, fb, whole, tol, n) {
    const mid = (a + b) / 2, fl = f((a + mid) / 2), fr = f((mid + b) / 2);
    const left = (mid - a) / 6 * (fa + 4 * fl + fm), right = (b - mid) / 6 * (fm + 4 * fr + fb);
    const roundoff = 2048 * Number.EPSILON * (Math.abs(left) + Math.abs(right));
    if (Math.abs(left + right - whole) < Math.max(15 * tol, roundoff)) return left + right + (left + right - whole) / 15;
    if (n === 0) throw Error(`quadrature did not converge: ${a}, ${b}, ${whole}, ${tol}`);
    return recurse(a, mid, fa, fl, fm, left, tol / 2, n - 1)
      + recurse(mid, b, fm, fr, fb, right, tol / 2, n - 1);
  }
  return recurse(a, b, f(a), f(mid), f(b), whole, tolerance, depth);
}
function travel(speed, a, b, tolerance = .0001) {
  const split = Math.min(b, a + 1);
  return adaptive(speed.speed, a, split, tolerance / 2) + adaptive(speed.speed, split, b, tolerance / 2);
}
function terrainReference() {
  const frame = surfaceQuery([]).frame;
  const up = unit(frame[0]), forward = unit(add(frame[1], mul(up, -dot(frame[1], up))));
  // End on the existing, shared downstream canyon trunk at map x ~= 35 miles.
  // Keep its angular terrain identity during the basic Earth-radius rescale.
  // Its chart scale is explicitly separate from the new physical planet radius.
  const offset = Math.asin((35 - 18) / 13500);
  const normal = q => add(mul(up, Math.cos(q + offset)), mul(forward, Math.sin(q + offset)));
  const step = 1e-6, count = 15001;
  const offsets = Array.from({ length: count }, (_, i) => (i - count + 1) * step);
  const query = surfaceQuery(offsets.map(normal));
  const values = query.samples.map(x => x[0]);
  const terrain = q => {
    assert(q >= offsets[0] && q <= 1e-10, `terrain query outside table: ${q}`);
    const u = Math.max(0, Math.min(count - 1, (q - offsets[0]) / step)), i = Math.min(count - 2, Math.floor(u));
    return values[i] + (values[i + 1] - values[i]) * (u - i);
  };
  // Backward derivative at the actual endpoint; the path will match the local
  // terrain tangent, not chase its individual samples as a runtime controller.
  const ground = values.at(-1), slope = (3 * ground - 4 * values.at(-2) + values.at(-3)) / (2 * step);
  return { normal, terrain, ground, slope, first: offsets[0], step,
    endpoint: query.samples.at(-1), frame, offsets, values };
}
function coupledRoute(thetaEnd, sigma, terrain, opening = null) {
  const base = referenceRoute();
  function raw(theta) {
    const d = thetaEnd - theta, u = theta;
    const h0 = base.height(theta), h1 = base.dh(theta);
    const inward = base.incomingLength * Math.exp(-4 * u) / u;
    const l = -base.b - 2 * base.c * (theta - ENTRY_THETA);
    const h2 = h0 * (l * l - 2 * base.c);
    const h3 = h0 * (l ** 3 - 6 * base.c * l);
    function jet(scale, power = 4) {
      const e = Math.exp(-((d / scale) ** power));
      if (e === 0) return [0, 0, 0, 0];
      const l1 = power * d ** (power - 1) / scale ** power;
      const l2 = -power * (power - 1) * d ** (power - 2) / scale ** power;
      const l3 = power === 2 ? 0 : power * (power - 1) * (power - 2) * d ** (power - 3) / scale ** power;
      return [e, e * l1, e * (l1 * l1 + l2), e * (l1 ** 3 + 3 * l1 * l2 + l3)];
    }
    const taper = jet(sigma, 2), envelope = jet(1);
    const f = [1 - taper[0], -taper[1], -taper[2], -taper[3]];
    const h = [h0, h1, h2, h3];
    const tangentGround = terrain.ground - terrain.slope * d - RELIEF;
    const target = [R + RELIEF + 30, 0, 0, 0].map((v, n) => v
      + h.slice(0, n + 1).reduce((s, value, i) => s + binomial(n, i) * value * f[n - i], 0)
      + tangentGround * envelope[n] + (n ? n * terrain.slope * envelope[n - 1] : 0));
    const cos = Math.cos(theta), sin = Math.sin(theta);
    // The axial tangent extension changes the TRUE radius. Solve the radial
    // coefficient so |P|² = target² + incoming²; both terms decrease during
    // capture. Merely subtracting incoming from Z made an unintended perigee
    // and subsequent climb before the nominal orbit entry.
    const incomingJet=[inward,-inward*(4+1/u),inward*(16+8/u+2/u**2),
      -inward*(64+48/u+24/u**2+6/u**3)];
    const along=[inward*sin,incomingJet[1]*sin+inward*cos,
      (incomingJet[2]-inward)*sin+2*incomingJet[1]*cos,
      (incomingJet[3]-3*incomingJet[1])*sin+(3*incomingJet[2]-inward)*cos];
    const squared=target.map((_,n)=>target.slice(0,n+1).reduce((s,x,j)=>s+binomial(n,j)
      *(x*target[n-j]+along[j]*along[n-j]),0));
    const rad=[Math.sqrt(squared[0])];
    rad[1]=squared[1]/(2*rad[0]);
    rad[2]=(squared[2]-2*rad[1]**2)/(2*rad[0]);
    rad[3]=(squared[3]-6*rad[1]*rad[2])/(2*rad[0]);
    const r=rad.map((x,i)=>x+along[i]);
    return [
      [0, r[0] * cos, r[0] * sin - inward],
      [0, r[1] * cos - r[0] * sin, r[1] * sin + r[0] * cos + inward * (4 + 1 / u)],
      [0, (r[2] - r[0]) * cos - 2 * r[1] * sin, (r[2] - r[0]) * sin + 2 * r[1] * cos - inward * (16 + 8 / u + 2 / u ** 2)],
      [0, (r[3] - 3 * r[1]) * cos + (r[0] - 3 * r[2]) * sin,
        (r[3] - 3 * r[1]) * sin + (3 * r[2] - r[0]) * cos + inward * (64 + 48 / u + 24 / u ** 2 + 6 / u ** 3)],
    ];
  }
  function state(theta) {
    const result = raw(theta);
    if (opening) {
      const d = theta - opening.theta, w = .001;
      const e = Math.exp(-d / w);
      if (e > 0) {
        const l = -1/w, lp = 0, lpp = 0;
        const a = opening.a2, b = opening.a3;
        const f = a * d * d + b * d ** 3, f1 = 2 * a * d + 3 * b * d * d;
        const f2 = 2 * a + 6 * b * d, f3 = 6 * b;
        const q = [e * f, e * (f1 + f * l), e * (f2 + 2 * f1 * l + f * (l * l + lp)),
          e * (f3 + 3 * f2 * l + 3 * f1 * (l * l + lp) + f * (l ** 3 + 3 * l * lp + lpp))];
        q.forEach((value, i) => { result[i] = add(result[i], mul(opening.normal, value)); });
      }
    }
    return result;
  }
  const metric = theta => Math.hypot(...state(theta)[1]);
  // Bound each integration cell. A single adaptive sample over several laps
  // could otherwise miss the narrow, fixed near-surface feature at the end.
  const arc = (a, b, tolerance = .001) => {
    if (b < a) return -arc(b, a, tolerance);
    const cuts = [a, b, thetaEnd - .05, thetaEnd - .01, thetaEnd - .002,
      ...(opening ? [opening.theta + .001, opening.theta + .002, opening.theta + .004] : [])]
      .filter(x => x >= a && x <= b).sort((a, b) => a - b);
    let total = 0;
    for (let i = 1; i < cuts.length; i++) if (cuts[i] > cuts[i - 1]) total += adaptive(metric, cuts[i - 1], cuts[i], tolerance / cuts.length);
    return total;
  };
  return { raw, state, metric, arc, thetaEnd, sigma };
}

// One fixed spatial heading field, integrated into the route. Heading and its
// first two spatial derivatives match the straight asymptotes. A unit tangent
// avoids the speed/curvature concentration of mixing two direction vectors.
const GL8 = [[.1834346424956498,.362683783378362], [.525532409916329,.3137066458778873],
  [.7966664774136267,.2223810344533745], [.9602898564975363,.1012285362903763]];
function spatialBend(route, theta4, length = 210000000, weave = 400000, angle = Math.PI / 2) {
  const startZ = route.state(theta4)[0][2];
  function heading(q) {
    if (length <= 0) return [0,0,0];
    const u = Math.max(0, Math.min(1, q/length));
    return [angle*(1-smooth(u)), -angle*30*u*u*(1-u)**2/length,
      -angle*60*u*(1-u)*(1-2*u)/length**2];
  }
  function tail(q) {
    if(q>=length) return [0,0];
    const lower=Math.max(0,q), n=8, h=(length-lower)/(2*n); let x=0,z=0;
    for(let i=0;i<n;i++) for(const [a,w] of GL8) for(const sign of [-1,1]) {
      const beta=heading(lower+(2*i+1+sign*a)*h)[0];
      x+=h*w*Math.sin(beta); z+=h*w*(1-Math.cos(beta));
    }
    if(q<0){x-=q*Math.sin(angle);z-=q*(1-Math.cos(angle));}
    return [x,z];
  }
  function state(theta) {
    const out = route.state(theta).map(a => a.slice()), z = out.map(a => a[2]);
    const q=z[0]-startZ, [b,b1,b2]=heading(q), [xTail,zTail]=tail(q), sn=Math.sin(b),cs=Math.cos(b);
    out[0][0]-=xTail;out[0][2]+=zTail;
    out[1][0]+=sn*z[1];out[1][2]=cs*z[1];
    out[2][0]+=cs*b1*z[1]**2+sn*z[2];out[2][2]=-sn*b1*z[1]**2+cs*z[2];
    out[3][0]+=(-sn*b1*b1+cs*b2)*z[1]**3+3*cs*b1*z[1]*z[2]+sn*z[3];
    out[3][2]=(-cs*b1*b1-sn*b2)*z[1]**3-3*sn*b1*z[1]*z[2]+cs*z[3];
    // One shallow fixed lateral excursion within the late cylindrical/square
    // volume. Its derivatives vanish exponentially long before close orbit.
    const u = (z[0] + 45000000) / 20000000, g = Math.exp(-(u ** 4));
    if (g > 0) {
      const l = -4 * u ** 3, lp = -12 * u*u, lpp = -24*u, sn = Math.sin(u), cs = Math.cos(u);
      const raw=[weave*g*sn, weave*g*(cs+l*sn)/20000000,
        weave*g*(-sn+2*l*cs+(l*l+lp)*sn)/20000000**2,
        weave*g*(-cs-3*l*sn+3*(l*l+lp)*cs+(l**3+3*l*lp+lpp)*sn)/20000000**3];
      // Compact C3 spatial support. A Gaussian alone leaves small excursions
      // whenever subsequent orbits revisit the same axial plane.
      const v=Math.max(0,Math.min(1,(z[0]+25000000)/15000000)),w=1-v;
      const cutoff=[1-v**4*(35+v*(-84+v*(70-20*v))),
        -140*v**3*w**3/15000000,
        -420*v*v*w*w*(1-2*v)/15000000**2,
        -840*v*w*(1-5*v+5*v*v)/15000000**3];
      const [a,b,c,d]=raw.map((_,n)=>raw.slice(0,n+1).reduce((s,x,j)=>s+binomial(n,j)*x*cutoff[n-j],0));
      out[0][0] += a; out[1][0] += b*z[1];
      out[2][0] += c*z[1]**2+b*z[2]; out[3][0] += d*z[1]**3+3*c*z[1]*z[2]+b*z[3];
    }
    return out;
  }
  const metric = theta => Math.hypot(...state(theta)[1]);
  const arc = (a,b,tol=.001) => {
    if (b<a) return -arc(b,a,tol);
    const cuts = [a,b,theta4+.0002,theta4+.0005,theta4+.001,theta4+.002,
      route.thetaEnd-.05,route.thetaEnd-.01,route.thetaEnd-.002].filter(x=>x>=a&&x<=b).sort((a,b)=>a-b);
    let s=0;
    for(let i=1;i<cuts.length;i++) if(cuts[i]>cuts[i-1]) s+=adaptive(metric,cuts[i-1],cuts[i],tol/cuts.length);
    return s;
  };
  return {...route,state,metric,arc,bend:{length,weave,startZ,angle}};
}

function fitRoute(speed, terrain, { bendLength = 0, weave = 400000, bendAngle = Math.PI / 2 } = {}) {
  const total = travel(speed, ENTRY, speed.end);
  const toAir = travel(speed, ENTRY, speed.air);
  let chosen;
  function residual(logSigma) {
    const sigma = Math.exp(logSigma);
    const thetaEnd = root(end => coupledRoute(end, sigma, terrain).arc(ENTRY_THETA, end) - total, ENTRY_THETA + 2 * TAU, ENTRY_THETA + 3 * TAU);
    const route = coupledRoute(thetaEnd, sigma, terrain);
    const thetaAir = root(theta => route.arc(ENTRY_THETA, theta) - toAir, thetaEnd - .015, thetaEnd);
    const position = route.state(thetaAir)[0];
    const agl = Math.hypot(...position) - R - terrain.terrain(thetaAir - thetaEnd);
    chosen = { thetaEnd, sigma, thetaAir, agl };
    return agl - 9144;
  }
  const sigma = Math.exp(root(residual, Math.log(.00001), Math.log(.02)));
  residual(Math.log(sigma));
  let route = coupledRoute(chosen.thetaEnd, sigma, terrain);
  const before = travel(speed, START, ENTRY);
  let theta4 = root(theta => route.arc(theta, ENTRY_THETA) - before, .0001, .1), opening;
  for (let i = 0; i < 4; i++) {
    const raw = route.raw(theta4), tangent = unit(raw[1]);
    const normal = [0, -tangent[2], tangent[1]];
    const a2=-dot(normal,raw[2])/2;
    opening = { theta: theta4, normal, a2, a3: -dot(normal, raw[3]) / 6 + a2/.001 };
    route = coupledRoute(chosen.thetaEnd, sigma, terrain, opening);
    const residual = route.arc(theta4, ENTRY_THETA) - before;
    if (Math.abs(residual) < .0001) break;
    theta4 += residual / route.metric(theta4);
  }
  if (bendLength > 0 || weave > 0) {
    const build = theta => {
      const raw = coupledRoute(chosen.thetaEnd,sigma,terrain).raw(theta), tangent=unit(raw[1]);
      const normal=[0,-tangent[2],tangent[1]];
      const a2=-dot(normal,raw[2])/2;
      const o={theta,normal,a2,a3:-dot(normal,raw[3])/6+a2/.001};
      return {route:spatialBend(coupledRoute(chosen.thetaEnd,sigma,terrain,o),theta,bendLength,weave,bendAngle),opening:o};
    };
    theta4=root(theta=>build(theta).route.arc(theta,ENTRY_THETA)-before,.0001,.1);
    const built=build(theta4); route=built.route; opening=built.opening;
  }
  const s4 = route.arc(theta4, ENTRY_THETA);
  return { ...route, ...chosen, theta4, opening, openingArcResidual: s4 - before };
}

function arcTable(route, refinement = 1) {
  const nodes = [{ theta: route.theta4, s: 0, metric: route.metric(route.theta4) }];
  while (nodes.at(-1).theta < route.thetaEnd) {
    const a = nodes.at(-1);
    const step = Math.min(.002, Math.max(1e-7, Math.min(a.theta, route.thetaEnd - a.theta + .0005) * .004)) / refinement;
    const b = Math.min(route.thetaEnd, a.theta + step), m = route.metric(b);
    const s = a.s + (b - a.theta) / 6 * (a.metric + 4 * route.metric((a.theta + b) / 2) + m);
    nodes.push({ theta: b, s, metric: m });
  }
  const theta = s => {
    assert(s >= -.002 && s <= nodes.at(-1).s + .002, `arc outside route: ${s}`);
    let lo = 0, hi = nodes.length - 1;
    while (hi - lo > 1) { const m = (lo + hi) >> 1; if (nodes[m].s < s) lo = m; else hi = m; }
    const a = nodes[lo], b = nodes[hi], h = b.theta - a.theta;
    // Cubic Hermite interpolation of the measured ARC, not position blending.
    let u = Math.max(0, Math.min(1, (s - a.s) / (b.s - a.s)));
    for (let j = 0; j < 6; j++) {
      const value = a.s + (b.s - a.s) * u * u * (3 - 2 * u)
        + h * (a.metric * u * (1 - u) ** 2 + b.metric * u * u * (u - 1));
      const derivative = (b.s - a.s) * 6 * u * (1 - u)
        + h * (a.metric * (1 - 4 * u + 3 * u * u) + b.metric * (3 * u * u - 2 * u));
      u = Math.max(0, Math.min(1, u - (value - s) / derivative));
    }
    return a.theta + h * u;
  };
  return { theta, nodes, length: nodes.at(-1).s };
}
function kinematics(route, theta, speed, t) {
  const [p, d, e, f] = route.state(theta), w = Math.hypot(...d), a = dot(d, e);
  const tangent = mul(d, 1 / w);
  const curvature = add(mul(e, 1 / w ** 2), mul(d, -a / w ** 4));
  const third = add(add(mul(f, 1 / w ** 3), mul(e, -3 * a / w ** 5)),
    mul(d, -(dot(e, e) + dot(d, f)) / w ** 5 + 4 * a * a / w ** 7));
  const v = speed.speed(t), v1 = -v * speed.rate(t), v2 = v * (speed.rate(t) ** 2 - speed.rateDerivative(t));
  return { p, v, tangent, curvature: Math.hypot(...curvature),
    velocity: mul(tangent, v),
    acceleration: add(mul(tangent, v1), mul(curvature, v * v)),
    jerk: add(add(mul(tangent, v2), mul(curvature, 3 * v * v1)), mul(third, v ** 3)),
    radialSpeed: v * dot(tangent, unit(p)),
  };
}
function smooth(u) { u = Math.max(0, Math.min(1, u)); return u ** 3 * (10 + u * (-15 + 6 * u)); }

const cross = (a,b) => [a[1]*b[2]-a[2]*b[1],a[2]*b[0]-a[0]*b[2],a[0]*b[1]-a[1]*b[0]];
const SUN = [0,Math.cos(4),Math.sin(4)];
function studyCamera(position, velocity) {
  const radius=Math.hypot(...position), inward=mul(position,-1/radius), flight=unit(velocity);
  const ratio=Math.min(1,R/radius), limb=Math.sqrt(1-ratio*ratio);
  const tangent=unit(add(flight,mul(inward,-dot(flight,inward))));
  const aircraft=smooth((100000-(radius-R))/90000);
  const dip=(4+8*aircraft)*Math.PI/180, sn=Math.sin(dip),cs=Math.cos(dip);
  const horizon=add(mul(tangent,ratio*cs-limb*sn),mul(inward,limb*cs+ratio*sn));
  const look=smooth((ratio-.05)/.30), forward=unit(add(mul(flight,1-look),mul(horizon,look)));
  const reference=add(mul([-1,0,0],1-look),mul(unit(cross(inward,tangent)),look));
  const right=unit(add(reference,mul(forward,-dot(reference,forward)))),down=unit(cross(forward,right));
  return {position,forward,right,down};
}
function projectStudy(camera, point, direction = false) {
  const r=direction?point:add(point,mul(camera.position,-1)),z=dot(r,camera.forward);
  return z<=0?null:[160+154*dot(r,camera.right)/z,100+154*dot(r,camera.down)/z];
}

function completeStudy({ denseHz = 1920 } = {}) {
  const speed = fitSpeed(), terrain = terrainReference(), route = fitRoute(speed, terrain);
  const table = arcTable(route, 2), refined = arcTable(route, 4);
  const distance = t => travel(speed, START, t);
  const thetaAt = t => table.theta(distance(t));
  const polar = theta => {
    const p = route.state(theta)[0], wrapped = Math.atan2(p[2], p[1]);
    return wrapped + TAU * Math.round((theta - wrapped) / TAU);
  };
  const entryPolar = polar(ENTRY_THETA), endPolar = polar(route.thetaEnd);
  const crossings = [1, 2, 3, 4].map(n => root(theta => polar(theta) - entryPolar - n * Math.PI, ENTRY_THETA, route.thetaEnd));
  const timeAt = theta => {
    const s = route.arc(route.theta4, theta);
    return root(t => distance(t) - s, START, speed.end);
  };
  const events = [
    [5, 'Wormhole emergence', 5], [7.6, 'Circular-grid entry: incomplete spokes', 7.6],
    [11, 'Spoke assembly underway', 11], [16, 'Circular assembly continues', 16],
    [20, 'Connected circular structure', 20], [23, 'Square evolution begins', 23],
    [26, 'Fully squared grid', 26], [27, 'Planet approach', 27],
    [timeAt(ENTRY_THETA), 'Orbit 1 entry', 28], [timeAt(crossings[0]), 'Orbit 1 far-side half-lap', null],
    [timeAt(crossings[1]), 'Orbit 2 begins', 33], [timeAt(crossings[2]), 'Orbit 2 far-side half-lap', null],
    [timeAt(crossings[3]), 'Orbit 3 begins; partial-lap descent', THIRD],
    [speed.air, 'Airline height: terrain crossing', 57],
    [speed.end, 'Canyon arrival; proposed manual takeover', 77],
  ];
  const atmosphereTheta = root(theta => Math.hypot(...route.state(theta)[0]) - R - 100000, ENTRY_THETA, crossings[3]);
  events.push([timeAt(atmosphereTheta), '100-km atmosphere boundary', null]);
  const times = Array.from({ length: speed.end - EXIT + 1 }, (_, i) => i + EXIT);
  for (const [t] of events) if (!times.some(x => Math.abs(x - t) < 1e-7)) times.push(t);
  times.sort((a, b) => a - b);
  const angles = times.map(thetaAt);
  const surface = surfaceQuery(angles.map(theta => terrain.normal(polar(theta) - endPolar))).samples;
  const rows = times.map((t, i) => {
    const state = kinematics(route, angles[i], speed, t);
    return { t, event: events.filter(([time]) => Math.abs(time - t) < 1e-7).map(([, name]) => name).join('; '),
      speed: state.v, exitPercent: 100 * state.v / speed.speed(EXIT),
      lastSecondLossPercent: t >= 6 ? 100 * (1 - state.v / speed.speed(t - 1)) : null,
      agl: Math.hypot(...state.p) - R - surface[i][0], referenceHeight: Math.hypot(...state.p) - R,
      orbitProgress: (polar(angles[i]) - entryPolar) / TAU, distanceFromExit: distance(t) - distance(EXIT),
      axial: state.p[2], vertical: state.p[1], lateral: state.p[0], radialSpeed: state.radialSpeed,
      tangentialSpeed: Math.sqrt(Math.max(0, state.v ** 2 - state.radialSpeed ** 2)),
    };
  });
  const rowAt = t => rows.reduce((a, b) => Math.abs(a.t - t) < Math.abs(b.t - t) ? a : b);
  const layout = {
    planetRadius: R, reliefBound: RELIEF, atmosphereHeight: 100000,
    circularRadius: 6000000, squareWidth: 18000000, squareHeight: 2 * referenceRoute().axis,
    axisHeight: referenceRoute().axis, axialPitch: 2000000, lanes: 192,
    ringStartZ: rowAt(7.6).axial, connectedZ: rowAt(20).axial,
    squareStartZ: rowAt(23).axial, squareEndZ: rowAt(26).axial,
    endZ: 3 * R, wellDepth: R, wellWidth: 1.5 * R,
    meshVisibilityStart: 180000000, meshVisibilityEnd: 210000000,
    backgroundStarCount: 4096, backgroundMinimumDistance: 1e13,
    foregroundOpeningSimilarity: speed.v4 / 3400613843.4782605,
    terrainChartUnitsPerRadian: 13500,
    terminalMinimumRequiredClearance: 20, terminalPlannedClearance: 30,
  };
  const maxima = {};
  const framing = { sunDirection: SUN, sunWindows: [], horizonRows: [], maximumLateralExcursion: 0,
    maximumFrameRateDegreesPerSecond: 0, maximumFrameRateTime: null, firstRingAtExit: null };
  let previousFrame=null, sunWindow=null;
  function max(name, value, t) { if (!maxima[name] || value > maxima[name].value) maxima[name] = { value, t }; }
  const count = Math.round((speed.end - START) * denseHz), dt = 1 / denseHz;
  let s = 0, previous = null, previousPrevious = null, minRoof = Infinity, minSide = Infinity;
  let maxOutsideLowerFootprint = 0, minCircularClearance = Infinity, minSignedApproachCurvature = Infinity, maxUpwardApproachAngle = -Infinity;
  let minAGLBound = Infinity, minActualAGL = Infinity, minAGLTheta = null, maxTerrain = 0;
  const queryAngles = [];
  for (let i = 0; i <= count; i++) {
    const t = START + i * dt;
    if (i) s += dt / 6 * (speed.speed(t - dt) + 4 * speed.speed(t - dt / 2) + speed.speed(t));
    const theta = table.theta(s), state = kinematics(route, theta, speed, t), v = state.v;
    if (i % Math.max(1,Math.round(denseHz/60)) === 0) {
      const camera=studyCamera(state.p,state.velocity), sun=projectStudy(camera,SUN,true);
      const b=dot(state.p,SUN), discriminant=b*b-(dot(state.p,state.p)-R*R);
      const visible=sun&&sun[0]>=0&&sun[0]<320&&sun[1]>=0&&sun[1]<200&&!(b<0&&discriminant>=0);
      if(visible&&!sunWindow) sunWindow={start:t,end:t};
      if(visible) sunWindow.end=t;
      if(!visible&&sunWindow){framing.sunWindows.push(sunWindow);sunWindow=null;}
      if(previousFrame) {
        const cosine=Math.min(1,Math.max(-1,dot(camera.forward,previousFrame.forward)));
        const rate=Math.acos(cosine)*180/Math.PI/(t-previousFrame.time);
        if(rate>framing.maximumFrameRateDegreesPerSecond) {
          framing.maximumFrameRateDegreesPerSecond=rate;framing.maximumFrameRateTime=t;
        }
      }
      previousFrame={...camera,time:t};
      if(t>=7.6) framing.maximumLateralExcursion=Math.max(framing.maximumLateralExcursion,Math.abs(state.p[0]));
      if(Math.abs(t-5)<1e-8) framing.firstRingAtExit=projectStudy(camera,[0,layout.axisHeight,layout.ringStartZ]);
      if(t>=28&&i%denseHz===0) {
        const n=unit(state.p), tang=unit(add(state.velocity,mul(n,-dot(state.velocity,n))));
        const ratio=R/Math.hypot(...state.p), horizon=add(mul(tang,ratio),mul(n,-Math.sqrt(1-ratio*ratio)));
        framing.horizonRows.push({time:t,row:projectStudy(camera,horizon,true)?.[1]});
      }
    }
    const rate = speed.rate(t);
    if(t>=EXIT&&t<=ENTRY) max('outwardApproachVelocityMps',state.radialSpeed,t);
    if (t >= EXIT) {
      max('fractionalBrakingPerSecond', rate, t);
      max('scalarDecelerationMps2', v * rate, t);
      max('scalarJerkMps3', Math.abs(v * (rate * rate - speed.rateDerivative(t))), t);
      max('vectorAccelerationMps2', Math.hypot(...state.acceleration), t);
      max('vectorJerkMps3', Math.hypot(...state.jerk), t);
      max('curvaturePerMetre', state.curvature, t);
      if (t >= 5.1) max('loss100msPercent', 100 * (1 - v / speed.speed(t - .1)), t);
      if (t >= 6) max('loss1sPercent', 100 * (1 - v / speed.speed(t - 1)), t);
      if (t >= speed.air) max('tailFractionalRateDerivative', speed.rateDerivative(t), t);
    }
    if (previousPrevious) {
      const measured = mul(add(state.p, mul(previousPrevious.p, -1)), 1 / (2 * dt));
      max('positionDerivativeRelativeError', Math.hypot(...add(measured, mul(previous.velocity, -1))) / previous.v, t - dt);
    }
    const p = state.p, half = 6000000 + (layout.axisHeight - 6000000) * smooth((p[2] - layout.connectedZ) / (layout.squareEndZ - layout.connectedZ));
    const square = smooth((p[2] - layout.squareStartZ) / (layout.squareEndZ - layout.squareStartZ));
    const radial = Math.hypot(p[0], p[1] - layout.axisHeight);
    if (theta < ENTRY_THETA && t >= 5) {
      const [, vq, aq] = route.state(theta);
      minSignedApproachCurvature = Math.min(minSignedApproachCurvature, (vq[1] * aq[2] - vq[2] * aq[1]) / Math.hypot(...vq) ** 3);
      maxUpwardApproachAngle = Math.max(maxUpwardApproachAngle, Math.atan2(vq[1], vq[2]));
    }
    if (p[2] >= layout.ringStartZ && theta < ENTRY_THETA) {
      const boundary = half * (1 + square * (radial / Math.max(Math.abs(p[0]), Math.abs(p[1] - layout.axisHeight), 1e-20) - 1));
      minCircularClearance = Math.min(minCircularClearance, boundary - radial);
    }
    if (p[2] >= layout.squareEndZ) {
      minRoof = Math.min(minRoof, layout.squareHeight - p[1]);
      minSide = Math.min(minSide, 9000000 - Math.abs(p[0]));
      const floor = -R / Math.sqrt(1 + (Math.hypot(p[0], p[2]) / layout.wellWidth) ** 2);
      if (p[1] < floor) maxOutsideLowerFootprint = Math.max(maxOutsideLowerFootprint, Math.hypot(p[0], p[2]));
    }
    const bound = Math.hypot(...p) - R - RELIEF;
    minAGLBound = Math.min(minAGLBound, bound);
    // Spatial sampling complements time samples: high speed must not skip
    // narrow landforms. Full terrain queries below cover the lower route.
    if (i % 32 === 0 && bound < 2000) queryAngles.push(theta);
    previousPrevious = previous; previous = state;
  }
  for (let theta = crossings[3]; theta < route.thetaEnd; theta += .00001) queryAngles.push(theta);
  for (let theta = route.thetaEnd - .015; theta < route.thetaEnd; theta += .0000005) queryAngles.push(theta);
  queryAngles.push(route.thetaEnd);
  if(sunWindow) framing.sunWindows.push(sunWindow);
  const lowerSurface = surfaceQuery(queryAngles.map(theta => terrain.normal(polar(theta) - endPolar))).samples;
  queryAngles.forEach((theta, i) => {
    const agl = Math.hypot(...route.state(theta)[0]) - R - lowerSurface[i][0];
    if (agl < minActualAGL) { minActualAGL = agl; minAGLTheta = theta; }
    maxTerrain = Math.max(maxTerrain, lowerSurface[i][0]);
  });
  const p4 = route.state(route.theta4), boundary = kinematics(route, route.theta4, speed, START);
  const angle5 = thetaAt(EXIT), exitState = kinematics(route, angle5, speed, EXIT);
  const exactArc = route.arc(route.theta4, route.thetaEnd, .0005);
  const pitches = [200000, 1000000, 2000000].map(pitch => ({ pitch,
    physicalReductionFromOld: 283384486.9565217 / pitch,
    entryPassagesPerSecond: rowAt(7.6).speed / pitch,
    ringsInVisibilityDiameter: Math.ceil(2 * layout.meshVisibilityEnd / pitch) + 2,
  }));
  const keyframes = [5, 7.6, 11, 16, 20, 23, 26, 27, 28].map(t => {
    const row = rowAt(t), cameraZ = row.axial, radius = 6000000 + (layout.axisHeight - 6000000) * smooth((cameraZ - layout.connectedZ) / (layout.squareEndZ - layout.connectedZ));
    const nearestDepth = t < 7.6 ? layout.ringStartZ - cameraZ : 154 * radius / 100;
    return { t, axialZ: cameraZ, physicalRingCadence: Math.abs(kinematics(route, thetaAt(t), speed, t).velocity[2]) / layout.axialPitch,
      referenceRingDiameterPixels: 2 * 154 * radius / nearestDepth,
      referenceEdgeMotionPixelsPerSecond: 154 * radius * Math.abs(kinematics(route, thetaAt(t), speed, t).velocity[2]) / nearestDepth ** 2,
      // Axis-aligned reference-ring estimate. Full camera/vertex projection
      // and temporal line integration remain production integration gates.
      projectedAxialPitchAt100PixelEdge: 100 * layout.axialPitch / (154 * radius / 100),
    };
  });
  const result = { status: 'OFFLINE PROPOSAL: not live or visually approved',
    speed: { ...speed, speed: undefined, rate: undefined, rateDerivative: undefined },
    route: { theta4: route.theta4, thetaEnd: route.thetaEnd, sigma: route.sigma, opening: route.opening, bend: route.bend,
      ground: terrain.ground, groundSlopePerRadian: terrain.slope, endpointTerrain: terrain.endpoint,
      nodes: table.nodes.length, length: table.length, endOrbitProgress: (endPolar - entryPolar) / TAU,
      entryTheta: ENTRY_THETA, entryPolar, radialProfile: referenceRoute(),
      entryRadius: Math.hypot(...route.state(ENTRY_THETA)[0]),
      firstLapArc: route.arc(ENTRY_THETA, crossings[1]), secondLapArc: route.arc(crossings[1], crossings[3]),
      partialThirdArc: route.arc(crossings[3], route.thetaEnd),
      final20sArc: distance(speed.end) - distance(speed.air),
      openingArcResidual: route.openingArcResidual },
    layout, events: events.map(([time, name, target]) => ({ name, target, measured: time, deviation: target === null ? null : time - target })),
    rows, keyframes, pitches, maxima, framing,
    checks: { denseHz, samples: count + 1, scalarMonotoneAnalytic: speed.coefficients.every(x => x >= 0),
      terminalScalarAcceleration: -speed.speed(speed.end) * speed.rate(speed.end),
      openingVectorAcceleration: Math.hypot(...boundary.acceleration), openingVectorJerk: Math.hypot(...boundary.jerk),
      arcQuadratureDifferenceMetres: table.length - exactArc,
      arcRefinementDifferenceMetres: refined.length - table.length,
      scalarIntegrationDifferenceMetres: distance(speed.end) - integral(speed.speed, START, speed.end, .002),
      minRoofClearance: minRoof, minSideClearance: minSide, minCircularClearance,
      minSignedApproachCurvature, maxUpwardApproachAngle,
      maxBelowWellFootprint: maxOutsideLowerFootprint, permittedUnderpassRadius: R + 350000,
      minSampledAGL: minActualAGL, minAGLTheta, lowerTerrainSamples: queryAngles.length, maxSampledTerrain: maxTerrain,
      terminalAGL: rows.at(-1).agl, airlineAGL: rowAt(speed.air).agl,
      maximumStarTranslationRadians: (distance(speed.end) - distance(EXIT)) / layout.backgroundMinimumDistance,
      gridDistanceAt4: layout.ringStartZ - p4[0][2], gridDistanceAt5: layout.ringStartZ - exitState.p[2] },
    boundary: { at4: boundary, at5: exitState,
      atSurface: kinematics(route, route.thetaEnd, speed, speed.end) },
    unresolved: [
      'Exact opening raster parity with a revised foreground-star catalogue has NOT been proved. A similarity transform preserves ideal projections but cannot move those same stars independently farther away.',
      'Projected cadence figures are reference-ring calculations, not a rendered motion/aliasing pass. The 33.333-ms live raster budget is unmeasured for these dimensions.',
      'Terrain clearance is densely sampled against the existing terrain and a 20-km envelope; an interval bound and collision-safe continuation into manual flight remain integration gates.',
      'The selected route includes a compact C3 lateral excursion. The proposed camera and fixed Sun have numerical framing checks; production projection, lighting/cloud occlusion and uninterrupted playback remain unverified.',
    ],
  };
  result.openingStars = openingStarWitness(result);
  result.numericalGates = {
    decreasingTotalSpeed: result.checks.scalarMonotoneAnalytic,
    easingFinalFractionalBrake: maxima.tailFractionalRateDerivative.value <= 1e-12,
    independentlyDifferentiatedPosition: maxima.positionDerivativeRelativeError.value < 1e-5,
    arcConvergenceBelowOneCentimetre: Math.abs(result.checks.arcQuadratureDifferenceMetres) < .01
      && Math.abs(result.checks.arcRefinementDifferenceMetres) < .01,
    tunnelContainment: minRoof > 0 && minSide > 0 && minCircularClearance > 0
      && maxOutsideLowerFootprint < R + 350000,
    sampledTerrainClearance: minActualAGL >= 20,
    noApproachCounterBank: minSignedApproachCurvature >= -1e-14 && maxUpwardApproachAngle <= 0,
    noCaptureAltitudeRebound: maxima.outwardApproachVelocityMps.value <= 0,
    partialThirdOrbitOnly: result.route.endOrbitProgress > 2 && result.route.endOrbitProgress < 3,
  };
  result.readyForLiveIntegration = false;
  return result;
}

function markdown(d) {
  const n = (x, digits = 3) => x.toLocaleString('en-US', { minimumFractionDigits: digits, maximumFractionDigits: digits });
  const c = d.checks, r = d.route, l = d.layout;
  const lines = [
    '# Phase 2 — evaluated flight grid', '',
    'Offline numerical proposal, NOT a live flight or visual approval. Phase 2 remains open',
    'for the foreground-star decision and exact opening/render validation.',
    'See [the interpretation and remaining work](phase2-study.md).', '',
    'Numerical speeds below are fitted choices, **not fixed specifications**. The old',
    '600-m/s airline and 133-m/s surface pair is not imposed. Geometric orbital crossings',
    'are measured from the actual route; field boundaries are positioned at the desired',
    'spatial crossings. Those latter times are not independent optical-visibility proofs.', '',
    '## One speed function', '',
    'Metres and seconds. The same expression is evaluated throughout 4–77 s;',
    'the 4–5 s part establishes continuity with the opening, not a second brake.', '',
    '```text',
    'x = t - 4; w = 0.25',
    'g(t) = x - 1.5*w + 2*w*exp(-x/w) - 0.5*w*exp(-2*x/w)',
    `u = g(t) / ${d.speed.span}`,
    'I_i(u) = sum(j=i+1..25, binomial(25,j)*u^j*(1-u)^(25-j))',
    `v(t) = ${d.speed.v4} * exp(-sum(i=0..24, c_i*I_i(u)))`,
    's(t) = integral(v(tau), tau=4..t)',
    'position(t) = P(s(t)); |dP/ds| = 1',
    '```', '',
    'All coefficients are nonnegative. They are fitted once offline, never selected by',
    'an orbit/event or adjusted while flying. Thus v is positive and monotonically',
    'decreasing analytically; its terminal first/second time derivatives vanish.',
    'The final fractional braking rate also decreases, not just absolute deceleration.', '',
    'Nonzero coefficients (all others are exactly zero):', '',
    '| i | c_i |', '|---:|---:|',
    ...d.speed.coefficients.flatMap((value, i) => value ? [`| ${i} | ${value} |`] : []), '',
    '## Selected fixed dimensions', '',
    '| Quantity | Metres, unless stated |', '|---|---:|',
    `| Planet radius / diameter | ${n(l.planetRadius, 0)} / ${n(2 * l.planetRadius, 0)} |`,
    `| Reserved relief / atmosphere height | ${n(l.reliefBound, 0)} / ${n(l.atmosphereHeight, 0)} |`,
    `| Circular radius / diameter | ${n(l.circularRadius, 0)} / ${n(2 * l.circularRadius, 0)} |`,
    `| Square width / height | ${n(l.squareWidth, 0)} / ${n(l.squareHeight, 0)} |`,
    `| Planet diameter as % of square width | ${n(200 * l.planetRadius / l.squareWidth)}% |`,
    `| Axis height above nominal floor | ${n(l.axisHeight, 0)} |`,
    `| Axial cell pitch | ${n(l.axialPitch, 0)} |`,
    `| Perimeter lanes | ${l.lanes} |`,
    `| Circular transverse spacing | ${n(TAU * l.circularRadius / l.lanes)} |`,
    `| Square transverse spacing | ${n(2 * (l.squareWidth + l.squareHeight) / l.lanes)} |`,
    `| Ring field start Z / end Z | ${n(l.ringStartZ)} / ${n(l.endZ)} |`,
    `| Field length / ring count | ${n(l.endZ - l.ringStartZ)} / ${Math.ceil((l.endZ - l.ringStartZ) / l.axialPitch) + 1} |`,
    `| Well depth / width parameter | ${n(l.wellDepth, 0)} / ${n(l.wellWidth, 0)} |`,
    `| Mesh distance fade starts / ends | ${n(l.meshVisibilityStart, 0)} / ${n(l.meshVisibilityEnd, 0)} |`,
    `| Proposed distant background stars / minimum depth | ${l.backgroundStarCount} / ${n(l.backgroundMinimumDistance, 0)} |`, '',
    `The cross-section widens smoothly from a 6,000-km circular radius to half-width`,
    `9,000 km and half-height ${n(l.axisHeight / 1000)} km between the 20 s and 26 s spatial planes.`,
    'Squaring occurs between the 23 s and 26 s planes.',
    'These are fixed functions of world Z, not functions of playback time.',
    `The nominal square floor is Y=0, roof Y=${n(l.squareHeight / 1000)} km, planet centre (0,0,0).`,
    'The floor well is Y=-R/sqrt(1+(hypot(X,Z)/(1.5*R))^2): it touches the',
    'planet bottom at its centre. The mesh continues beyond the planet.', '',
    '## Full percentage table', '',
    'AGL is sampled from the existing production terrain, not a reference sphere.',
    'Whole seconds and fractional geometric events are both included. The loss column',
    'always compares v(t) with v(t-1), never with the previous displayed row.',
    'A dash in the orbit column means the approach precedes orbit entry.', '',
    '| Time, s | Event | Total m/s | % exit speed | Lost in last 1 s | AGL, m | Route since exit, km | Orbits completed |',
    '|---:|---|---:|---:|---:|---:|---:|---:|',
    ...d.rows.map(row => `| ${n(row.t, 6)} | ${row.event || '—'} | ${n(row.speed, 1)} | ${n(row.exitPercent, 6)}% | ${row.lastSecondLossPercent === null ? '—' : n(row.lastSecondLossPercent, 6) + '%'} | ${n(row.agl, 1)} | ${n(row.distanceFromExit / 1000)} | ${row.orbitProgress < 0 ? '—' : n(row.orbitProgress, 6)} |`), '',
    '## Event timing comparison', '',
    '| Event | Target, s | Evaluated, s | Difference, s |', '|---|---:|---:|---:|',
    ...d.events.map(e => `| ${e.name} | ${e.target === null ? 'Derived' : n(e.target)} | ${n(e.measured, 6)} | ${e.deviation === null ? '—' : n(e.deviation, 6)} |`), '',
    '## Grid cadence and reference projections', '',
    'The pixel calculations below use a 154-pixel focal length and an axis-aligned',
    'reference ring. Inside the grid the reference edge is 100 pixels from screen',
    'centre; at emergence the actual first-ring depth is used. They are NOT a full',
    'camera/vertex rendering, clipping, aliasing or visibility acceptance test.', '',
    '| Time, s | Fixed axial Z, km | Rings passed/s | Reference diameter, px | Edge speed, px/s | Axial pitch at 100-px edge, px |',
    '|---:|---:|---:|---:|---:|---:|',
    ...d.keyframes.map(k => `| ${n(k.t, 1)} | ${n(k.axialZ / 1000)} | ${n(k.physicalRingCadence)} | ${n(k.referenceRingDiameterPixels)} | ${n(k.referenceEdgeMotionPixelsPerSecond)} | ${n(k.projectedAxialPitchAt100PixelEdge)} |`), '',
    'Cell-pitch alternatives hold the other dimensions and the same route/speed fixed:', '',
    '| Pitch, km | Reduction from old physical pitch | Ring passages/s at entry | Rings in visibility diameter |',
    '|---:|---:|---:|---:|',
    ...d.pitches.map(p => `| ${n(p.pitch / 1000, 0)} | ${n(p.physicalReductionFromOld)}x | ${n(p.entryPassagesPerSecond)} | ${p.ringsInVisibilityDiameter} |`), '',
    'Even the selected 2,000-km pitch crosses more than 15 rings/s early in the',
    'cylinder. A 30-Hz renderer therefore needs measured temporal integration/alias',
    'control, not skipped spokes or a new velocity cap. At most 212 ring ordinals',
    'fit inside the selected visibility diameter: about 81,408 candidate ring/rail',
    'edges at 192 lanes, before clipping, assembly and footprint filtering.',
    'A conservative 520 raster steps per candidate edge is about 42.3 million',
    'steps/frame, not a measured frame time. No 33.333-ms performance pass is claimed.', '',
    '## Route, clearance and numerical checks', '',
    `- First-lap arc: ${n(r.firstLapArc / 1000)} km; second-lap arc: ${n(r.secondLapArc / 1000)} km.`,
    `- Final partial lap: ${n(r.partialThirdArc / 1000)} km; surface reached after ${n(r.endOrbitProgress, 6)} total laps, not 3.`,
    `- Final 20 s provides ${n(r.final20sArc / 1000)} km of actual route length.`,
    `- ${c.samples.toLocaleString('en-US')} motion samples at ${c.denseHz} Hz; ${c.lowerTerrainSamples.toLocaleString('en-US')} lower-route terrain samples.`,
    `- Minimum sampled AGL ${n(c.minSampledAGL, 6)} m; the minimum safety requirement remains 20 m.`,
    `- Minimum roof / side clearance: ${n(c.minRoofClearance / 1000)} / ${n(c.minSideClearance / 1000)} km.`,
    `- Minimum incoming circular/widening clearance: ${n(c.minCircularClearance / 1000)} km.`,
    `- Signed incoming curvature minimum: ${c.minSignedApproachCurvature.toPrecision(8)} /m; maximum upward incoming angle ${n(c.maxUpwardApproachAngle * 180 / Math.PI, 9)} degrees. No upward approach leg.`,
    `- Below-well excursions stay within ${n(c.maxBelowWellFootprint / 1000)} km of the planet axis; allowed localized footprint ${n(c.permittedUnderpassRadius / 1000)} km.`,
    `- Position-derived speed/vector residual: ${n(d.maxima.positionDerivativeRelativeError.value * 100, 8)}%.`,
    `- Arc quadrature difference: ${n(c.arcQuadratureDifferenceMetres, 9)} m; refinement difference ${n(c.arcRefinementDifferenceMetres, 9)} m.`,
    `- Independent scalar quadrature difference: ${n(c.scalarIntegrationDifferenceMetres, 9)} m.`, '',
    '| Dense maximum | Value | Time, s |', '|---|---:|---:|',
    ...Object.entries(d.maxima).map(([name, value]) => `| ${name} | ${value.value.toPrecision(10)} | ${n(value.t, 6)} |`), '',
    'These are cinematic accelerations, not passenger-safe physical dynamics.',
    'Smoothness alone does not establish perceptual comfort; live review is still required.', '',
    '## Proposed camera and fixed Sun', '',
    `The selected fixed lateral excursion reaches ${n(d.framing.maximumLateralExcursion/1000)} km;`,
    'its compact C3 spatial support ends before close orbit. The proposed large exit',
    'bends are screening alternatives only and are NOT in this selected route.',
    `The first-ring centre projects to (${n(d.framing.firstRingAtExit[0])}, ${n(d.framing.firstRingAtExit[1])}) at 5 s.`,
    `The fixed Sun direction is (${d.framing.sunDirection.join(', ')}).`,
    'The following visibility windows test the camera frustum and spherical-core',
    'occlusion, not the production terrain/cloud/atmosphere renderer:', '',
    '| Sun window starts, s | Ends, s |', '|---:|---:|',
    ...d.framing.sunWindows.map(w=>`| ${n(w.start)} | ${n(w.end)} |`), '',
    `Maximum sampled camera forward rotation is ${n(d.framing.maximumFrameRateDegreesPerSecond)} degrees/s at ${n(d.framing.maximumFrameRateTime)} s.`,
    'That is a reported measurement, NOT a perceptual-comfort approval.',
    `The reference horizon is at row ${n(d.framing.horizonRows[0].row)} of 200 at orbit entry`,
    `and row ${n(d.framing.horizonRows.find(x=>x.time===57).row)} at airline height.`, '',
    '## Opening and stars — unresolved', '',
    `The proposed static opening similarity is ${l.foregroundOpeningSimilarity}.`,
    'A common similarity of the opening camera and star objects preserves ideal',
    'pinhole projections. The fixed route matches the transformed 4 s position and',
    'tangent; its opening normal acceleration/jerk vanish analytically. Measured',
    `residuals are ${c.openingVectorAcceleration} m/s² and ${c.openingVectorJerk} m/s³.`,
    'This matches opening position, velocity and acceleration. It is not a claim',
    'of matching the protected rush\'s one-sided third derivative: that opening',
    'already ends a quintic displacement at 4 s with a jerk discontinuity.',
    `The first grid is ${n(c.gridDistanceAt4 / 1000)} km away at 4 s and ${n(c.gridDistanceAt5 / 1000)} km at 5 s:`,
    'the proposed spatial visibility limit excludes it through 4 s and admits it at exit.', '',
    `However, ${d.openingStars.retainedFromBothProtectedViews} existing stars project inside both protected 3.95 s and 4 s views.`,
    `${d.openingStars.visibleAtEntry} remain in the reference view at circular entry; ${d.openingStars.passedBehindCameraBefore26} pass behind the`,
    'camera by 26 s after the common rescale. Retaining those objects has NOT solved',
    'the unwanted nearby-star rush. Moving them independently to background depth',
    'invalidates the opening similarity argument. No opening pixels or star objects',
    'were changed. This is a real unresolved design/integration issue, not a passed test.', '',
    `The NEW distant background alone has translation <=${n(c.maximumStarTranslationRadians, 9)} rad`,
    `(approximately ${n(154 * c.maximumStarTranslationRadians, 6)} pixels near screen centre) across the entire post-exit route.`,
    'That bound does not apply to the preserved foreground population.', '',
    '## Remaining gates', '',
    ...d.unresolved.map(x => `- ${x}`), '',
    'The live production binary and its known seven failing gates are unchanged.',
    'Reproduction: `make audit-build`, `node --test scripts/test-phase2-flight-study.cjs`,',
    'then `node scripts/phase2-flight-study.cjs --markdown` (or omit the flag for JSON).', '',
  ];
  return lines.join('\n');
}

if (require.main === module) {
  const result = completeStudy();
  console.log(process.argv.includes('--markdown') ? markdown(result) : JSON.stringify(result, null, 2));
  if (!Object.values(result.numericalGates).every(Boolean)) process.exitCode = 1;
}
module.exports = { integral, root, onset, onsetRate, bernstein, basisCDF, baselineSpeed, fitSpeed, referenceRoute,
  terrainReference, coupledRoute, fitRoute, surfaceQuery, arcTable, kinematics, completeStudy, openingStarWitness, markdown,
  spatialBend, studyCamera, projectStudy, SUN };
