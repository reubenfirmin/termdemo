'use strict';
const assert = require('node:assert/strict');
const { test } = require('node:test');
const { integral, root, onset, bernstein, basisCDF, fitSpeed, coupledRoute, kinematics } = require('./phase2-flight-study.cjs');
const { spatialBend, studyCamera, projectStudy, SUN } = require('./phase2-flight-study.cjs');

const speed = fitSpeed();
test('global Bernstein density is nonnegative and normalized at both endpoints', () => {
  assert(speed.coefficients.every(x => Number.isFinite(x) && x >= 0));
  for (let i = 0; i <= speed.degree; i++) {
    assert.equal(basisCDF(speed.degree, i, 0), 0);
    assert.equal(basisCDF(speed.degree, i, 1), 1);
  }
  assert(Math.abs(speed.speed(4) - speed.v4) < 1e-7);
  assert(Math.abs(speed.speed(speed.end) - speed.terminal) < 1e-7);
  assert.equal(speed.rate(4), 0);
  assert.equal(speed.rate(speed.end), 0);
  assert.equal(speed.rateDerivative(4), 0);
  assert.equal(speed.rateDerivative(speed.end), 0);
});
test('optimized evaluation matches the explicit single polynomial', () => {
  for (let t = 4; t <= speed.end; t += .03125) {
    const u = onset(t) / speed.span;
    const exponent = speed.coefficients.reduce((s, c, i) => s + c * basisCDF(speed.degree, i, u), 0);
    const direct = speed.v4 * Math.exp(-exponent);
    assert(Math.abs(speed.speed(t) / direct - 1) < 2e-13);
    const rate = (speed.degree + 1) / speed.span * (1 - Math.exp(-(t - 4) / .25)) ** 2
      * speed.coefficients.reduce((s, c, i) => s + c * bernstein(speed.degree, i, u), 0);
    assert(Math.abs(speed.rate(t) - rate) < 1e-13);
  }
});
test('whole speed curve decreases, including every late second', () => {
  let previous = speed.speed(4);
  for (let i = 1; i <= (speed.end - 4) * 1920; i++) {
    const t = 4 + i / 1920, v = speed.speed(t);
    // Floating exponent evaluation has machine-roundoff variation where
    // the terminal analytic derivative tends to zero; analytic sign above
    // remains the authority, not this numerical slack.
    assert(v <= previous * (1 + 3e-14));
    if (t >= speed.air) assert(speed.rateDerivative(t) <= 1e-14);
    previous = v;
  }
});
test('reported derivatives agree with independent finite differences', () => {
  for (let t = 4.1; t < speed.end - .1; t += .125) {
    const h = .0001, measured = (speed.speed(t + h) - speed.speed(t - h)) / (2 * h);
    assert(Math.abs(measured + speed.speed(t) * speed.rate(t)) < Math.max(.1, speed.speed(t) * 2e-8));
    const rateDerivative = (speed.rate(t + h) - speed.rate(t - h)) / (2 * h);
    assert(Math.abs(rateDerivative - speed.rateDerivative(t)) < 1e-7);
  }
});
test('geometric derivatives include the actual radial motion, not a second clock', () => {
  // Synthetic local datum is confined to this algebra test. The study itself
  // queries and checks the shared production terrain, never this fixture.
  const route = coupledRoute(14.54, .003, { ground: 46, slope: 778 });
  for (const theta of [.001, .01, .1, .5, .8, 3, 7, 12, 14.535, 14.539]) {
    const h = Math.min(1e-6, theta * 1e-5);
    const a = route.state(theta - h), b = route.state(theta + h), actual = route.state(theta);
    for (let derivative = 0; derivative < 3; derivative++) {
      const measured = b[derivative].map((x, i) => (x - a[derivative][i]) / (2 * h));
      const residual = Math.hypot(...measured.map((x, i) => x - actual[derivative + 1][i]));
      assert(residual / Math.max(1, Math.hypot(...actual[derivative + 1])) < 2e-5);
    }
    const state = kinematics(route, theta, speed, 30);
    assert(Math.abs(Math.hypot(...state.velocity) / speed.speed(30) - 1) < 1e-14);
  }
});
test('quadrature and inversions reject unbracketed events', () => {
  assert(Math.abs(integral(x => x * x, 0, 3) - 9) < 1e-12);
  assert(Math.abs(root(x => x * x - 4, 0, 3) - 2) < 1e-12);
  assert.throws(() => root(x => x * x + 1, 0, 3), /unbracketed/);
});
test('tangent approach turns one way; the rejected radial incoming term counter-banks', () => {
  const route = coupledRoute(15.34, .0026, { ground: 46, slope: 778 });
  for (let theta = .001; theta <= .8; theta += .0001) {
    const [, v, a] = route.state(theta);
    assert(v[1] < 0, 'an upward approach leg');
    assert(v[1] * a[2] - v[2] * a[1] > 0, 'counter-bank');
  }
  // Negative witness from the rejected phase-2 candidate: its incoming
  // distance was added to RADIUS, creating an S-bend despite C3 continuity.
  const q = -.92, u = q + Math.PI / 2, b = .14043777611390892, c = .005769917307539986;
  const e = 289300.3412990751 * Math.exp(-b * q - c * q * q), i = 9e6 * Math.exp(-4 * u) / u;
  const l = -b - 2 * c * q, r = 6371000 + 20000 + 30 + e + i;
  const dr = e * l - i * (4 + 1 / u), ddr = e * (l * l - 2 * c) + i * (16 + 8 / u + 2 / u ** 2);
  assert(r * r + 2 * dr * dr - r * ddr < 0, 'old S-bend must remain a failing witness');
});
test('true planet distance decreases into orbit; an axial-offset perigee cannot pass', () => {
  const route=coupledRoute(15.34,.0026,{ground:46,slope:778});
  for(let theta=.001;theta<=.8;theta+=.0001) {
    const [p,v]=route.state(theta);
    assert(p.reduce((s,x,i)=>s+x*v[i],0)<0,'outward velocity before orbit entry');
  }
  // Previous candidate's radius coefficient decreased but its true |P| rose.
  const theta=.8,h=300000,r=6371000+20000+30+h,dr=-.14910784101520966*h;
  const incoming=1e6*Math.exp(-4*theta)/theta;
  const p=[r*Math.cos(theta),r*Math.sin(theta)-incoming];
  const v=[dr*Math.cos(theta)-r*Math.sin(theta),dr*Math.sin(theta)+r*Math.cos(theta)+incoming*(4+1/theta)];
  assert(p[0]*v[0]+p[1]*v[1]>0,'rejected true-radius rebound must remain a failing witness');
});
test('negative witnesses: fixed obsolete speeds and a vertical override cannot pass the distance test', () => {
  const logMean = (600 - 133) / Math.log(600 / 133);
  assert(20 * logMean < 9144 - 20); // Conditional diagnostic, NOT a fixed spec.
  const v = speed.speed(57), withExtraDescent = Math.hypot(v, 1000);
  assert(withExtraDescent > v);
  assert(Math.abs(integral(speed.speed, 57, 77) - 25390.7551537) < .01);
});

test('off-centre excursion is fixed geometry with independently checked derivatives', () => {
  const base=coupledRoute(15.334829488010165,.0026668014406835278,{ground:45.984012228012084,slope:777.886748302592});
  const route=spatialBend(base,.0013131386998555322,0,400000,0);
  let lateral=0;
  for(let theta=.01;theta<.07;theta+=.0001) {
    const h=theta*1e-6, a=route.state(theta-h),b=route.state(theta+h),value=route.state(theta);
    lateral=Math.max(lateral,Math.abs(value[0][0]));
    for(let derivative=0;derivative<3;derivative++) {
      const error=Math.hypot(...b[derivative].map((x,i)=>(x-a[derivative][i])/(2*h)-value[derivative+1][i]));
      assert(error/Math.max(1,Math.hypot(...value[derivative+1]))<2e-6);
    }
  }
  assert(lateral>100000&&lateral<400000);
  assert.deepEqual(route.state(.8),base.state(.8),'the excursion must not retune the close orbit');
});

test('camera frame stays orthonormal and the same fixed Sun recurs each lap', () => {
  const dot=(a,b)=>a.reduce((s,x,i)=>s+x*b[i],0),visible=[0,0];
  for(let theta=.8;theta<.8+4*Math.PI;theta+=.001) {
    const r=6671000,p=[0,r*Math.cos(theta),r*Math.sin(theta)],v=[0,-Math.sin(theta),Math.cos(theta)];
    const camera=studyCamera(p,v),axes=[camera.forward,camera.right,camera.down];
    for(let i=0;i<3;i++)for(let j=0;j<3;j++)assert(Math.abs(dot(axes[i],axes[j])-(i===j?1:0))<1e-12);
    const sun=projectStudy(camera,SUN,true),inFront=dot(p,SUN)>0;
    if(sun&&sun[0]>=0&&sun[0]<320&&sun[1]>=0&&sun[1]<200&&inFront)visible[Math.floor((theta-.8)/(2*Math.PI))]++;
  }
  assert(visible.every(n=>n>100),'Sun must be visible in both orbits without following the camera');
});

test('exit-bend algebra matches a unit heading field, not a direction-vector blend', () => {
  const base=coupledRoute(15.34,.0026,{ground:46,slope:778}),start=.0013131386998555322;
  const bent=spatialBend(base,start,210e6,0,Math.PI/3);
  for(let theta=start;theta<.002;theta+=.00001) {
    // At the compact-support boundary, the fourth position derivative is
    // one-sided; a third-derivative finite difference converges linearly there.
    const plain=base.state(theta),value=bent.state(theta),h=theta*(theta===start?1e-9:1e-6);
    assert(Math.abs(Math.hypot(...plain[1])/Math.hypot(...value[1])-1)<1e-14);
    const a=bent.state(theta-h),b=bent.state(theta+h);
    for(let derivative=0;derivative<3;derivative++) {
      const error=Math.hypot(...b[derivative].map((x,i)=>(x-a[derivative][i])/(2*h)-value[derivative+1][i]));
      assert(error/Math.hypot(...value[derivative+1])<1e-6);
    }
  }
});
