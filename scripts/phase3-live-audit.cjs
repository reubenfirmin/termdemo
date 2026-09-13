#!/usr/bin/env node
'use strict';
// Test actual production camera/rendering against the reviewed independent
// numerical model and the OLD opening. Never regenerates an image baseline.
const fs = require('node:fs');
const {spawnSync} = require('node:child_process');
const crypto = require('node:crypto');
const assert = require('node:assert/strict');
const {fitSpeed,coupledRoute,spatialBend,arcTable,kinematics,integral} = require('./phase2-flight-study.cjs');
const audit = process.env.PHASE3_AUDIT_BINARY || 'target/phase0-audit/x86_64-unknown-linux-gnu/release/termdemo';
const normal = process.env.PHASE3_NORMAL_BINARY || 'target/x86_64-unknown-linux-gnu/release/termdemo';
const reference = require('../tests/phase3-opening-reference.json');
const openingRevision = require('../tests/phase3-opening-grid-revision.json');
const study = require('../tests/phase3-proposal-reference.json');
const sha = b => crypto.createHash('sha256').update(b).digest('hex');
const dot = (a,b) => a.reduce((s,x,i)=>s+x*b[i],0);
const sub = (a,b) => a.map((x,i)=>x-b[i]);
const scale = (a,k) => a.map(x=>x*k);
const smooth = u => {u=Math.max(0,Math.min(1,u));return u**3*(10+u*(-15+6*u));};
const result = {gates:{},measurements:{},frames:[],openingFrames:[]};
result.binaries={normal:{path:normal,sha256:sha(fs.readFileSync(normal))},
  audit:{path:audit,sha256:sha(fs.readFileSync(audit))}};
function gate(name,condition,detail) {
  result.gates[name]={pass:!!condition,detail};
  console.log(`${condition?'PASS':'FAIL'} ${name}: ${JSON.stringify(detail)}`);
}
function run(binary,input) {
  const r=spawnSync(binary,[],{input,maxBuffer:64*1024*1024,timeout:60000});
  if(r.error)throw r.error;
  assert.equal(r.status,0,r.stderr?.toString());return r.stdout;
}
function doubles(b,start=0) {const a=[];for(let i=start;i<b.length;i+=8)a.push(b.readDoubleLE(i));return a;}
const meta=run(audit,Buffer.from('J'));
assert.equal(meta.subarray(0,8).toString(),'TDWORLD3');
const world=doubles(meta,8),origin=world.slice(0,3),axes=[world.slice(3,6),world.slice(6,9),world.slice(9,12)],unit=world[12];
const local = p => axes.map(a=>dot(sub(p,origin),a)*unit);
const l=study.layout;
gate('world-dimensions',Math.abs(world[13]-6371000)<1e-6 && Math.abs(world[14]-l.axisHeight)<1e-6
  && world[20]===2000000 && world[21]===192,world);
const trace=run(audit,Buffer.from('s'));
assert.equal(trace.subarray(0,8).toString(),'TDMOTION');
const hz=trace.readUInt32LE(8),count=trace.readUInt32LE(12),cols=trace.readUInt32LE(16);
assert.equal(hz,1920);assert.equal(cols,25);assert.equal(count,77*hz+1);assert.equal(trace.length,28+count*200);
const row=i=>doubles(trace.subarray(28+i*200,28+(i+1)*200));
const h=crypto.createHash('sha256');
for(let i=0;i<reference.native_samples;i++)h.update(trace.subarray(28+i*200+8,28+i*200+152));
gate('protected-opening-native',h.digest('hex')===reference.native_sha256,{samples:reference.native_samples});
gate('fixed-star-catalogue',sha(run(audit,Buffer.from('I')))===reference.catalogue_query_I_sha256,{objects:16254});
gate('world-topology-alternate-cameras-and-clipping',run(audit,Buffer.from('N')).toString().includes('audit ok'),{});
const workerWitness=run(audit,Buffer.from('C')).toString().trim();
gate('parallel-renderer-rgb-and-depth',workerWitness.includes('audit ok')
  && workerWitness.includes('after real worker failure'),{comparisons:45,witness:workerWitness});
const speed=fitSpeed();
const terrain={ground:study.route.ground,slope:study.route.groundSlopePerRadian};
const base=coupledRoute(study.route.thetaEnd,study.route.sigma,terrain,study.route.opening);
const route={...spatialBend(base,study.route.theta4,0,400000),theta4:study.route.theta4};
const arc=arcTable(route,4);
let speedError=0,derivativeError=0,accelerationError=0,maxIncrease=0;
let minAGL=Infinity,minRoof=Infinity,minSide=Infinity,minCircle=Infinity,maxUnderpass=0,maxOutward=-Infinity;
let maxCameraRate=0,maxCameraRateTime=0,previousFrame=null;
for(let i=4*hz+1;i<count;i++) {
  const r=row(i),t=r[0],p=local(r.slice(1,4)),v=scale(r.slice(4,7),unit),expected=speed.speed(t);
  speedError=Math.max(speedError,Math.abs(Math.hypot(...v)/expected-1));
  const before=row(i-1);
  maxIncrease=Math.max(maxIncrease,(r[20]-before[20])*unit);
  minAGL=Math.min(minAGL,r[19]);
  if(t>=5 && t<=28)maxOutward=Math.max(maxOutward,dot(p,axes.map(a=>dot(v,a)))/Math.hypot(...p));
  if(p[2]>=l.ringStartZ && t<28) {
    const half=6e6+(l.axisHeight-6e6)*smooth((p[2]-l.connectedZ)/(l.squareEndZ-l.connectedZ));
    const radial=Math.hypot(p[0],p[1]-l.axisHeight);
    const square=smooth((p[2]-l.squareStartZ)/(l.squareEndZ-l.squareStartZ));
    const boundary=half*(1+square*(radial/Math.max(Math.abs(p[0]),Math.abs(p[1]-l.axisHeight),1e-20)-1));
    minCircle=Math.min(minCircle,boundary-radial);
  }
  if(p[2]>=l.squareEndZ) {
    minRoof=Math.min(minRoof,l.squareHeight-p[1]);minSide=Math.min(minSide,9e6-Math.abs(p[0]));
    const footprint=Math.hypot(p[0],p[2]),floor=-6371000/Math.sqrt(1+(footprint/9556500)**2);
    if(p[1]<floor)maxUnderpass=Math.max(maxUnderpass,footprint);
  }
  if(i>4*hz+1 && i<count-1) {
    const after=row(i+1),left=t-before[0],right=after[0]-t;
    const derivative=col=>( (after[col]-r[col])*left/right + (r[col]-before[col])*right/left )/(left+right);
    derivativeError=Math.max(derivativeError,Math.hypot(...[1,2,3].map((col,j)=>derivative(col)*unit-v[j]))/expected);
    // Five-point, actual nonuniform f32 timestamps. A three-point stencil has
    // ~8% truncation error during the tiny quadratic onset immediately after 4s.
    // Do not differentiate across the protected opening's different jerk.
    if(i>4*hz+2 && i<count-2) {
      const stencil=[row(i-2),before,r,after,row(i+2)],x=stencil.map(q=>q[0]-t);
      const weights=x.map((value,j)=>{
        if(j===2)return 0;
        let numerator=1,denominator=1;
        for(let k=0;k<5;k++)if(k!==j){denominator*=value-x[k];if(k!==2)numerator*=-x[k];}
        return numerator/denominator;
      });
      const measured=[4,5,6].map(col=>stencil.reduce((sum,q,j)=>sum+weights[j]*(q[col]-r[col]),0)*unit);
      const acceleration=scale(r.slice(7,10),unit);
      accelerationError=Math.max(accelerationError,Math.hypot(...sub(measured,acceleration))/Math.max(1,Math.hypot(...acceleration)));
    }
  }
  if(i%32===0) {
    const f=scale(r.slice(10,13),1/Math.hypot(...r.slice(10,13)));
    if(previousFrame) {
      const rate=Math.acos(Math.min(1,Math.max(-1,dot(f,previousFrame.f))))*180/Math.PI/(t-previousFrame.t);
      if(rate>maxCameraRate){maxCameraRate=rate;maxCameraRateTime=t;}
    }
    previousFrame={f,t};
  }
}
gate('total-speed-function',speedError<1e-12 && maxIncrease<1e-6,{maximumRelativeError:speedError,maximumIncreaseMps:maxIncrease});
gate('actual-position-derivative',derivativeError<2e-6,{maximumRelativeVectorError:derivativeError});
gate('actual-velocity-derivative',accelerationError<0.002,{maximumRelativeVectorError:accelerationError});
gate('tunnel-containment',minRoof>0 && minSide>0 && minCircle>0 && maxUnderpass<6371000+350000,
  {minRoof,minSide,minCircle,maxUnderpassFootprint:maxUnderpass});
gate('capture-no-altitude-rebound',maxOutward<0,{maximumOutwardVelocityMps:maxOutward});
gate('sampled-actual-terrain-clearance',minAGL>=29.99,{minimumMetres:minAGL,samples:count-4*hz-1,formalIntervalProof:false});
gate('partial-third-orbit-only',row(count-1)[22]>2 && row(count-1)[22]<3,{finalOrbitProgress:row(count-1)[22]});
let distance=0,lastTime=4,maxModelPositionError=0;
result.keyframes=[];
for(const time of [5,7.6,11,16,20,23,26,27,28,33,41.37352,57,70,77]) {
  const r=row(Math.round(time*hz)),t=r[0];
  distance+=integral(speed.speed,lastTime,t,0.001);lastTime=t;
  const expected=kinematics(route,arc.theta(distance),speed,t);
  const error=Math.hypot(...sub(local(r.slice(1,4)),expected.p));maxModelPositionError=Math.max(maxModelPositionError,error);
  result.keyframes.push({time:t,speedMps:r[20]*unit,exitPercent:100*r[20]*unit/speed.speed(5),agl:r[19],orbitProgress:r[22],modelErrorMetres:error});
}
gate('independent-route-and-arc',maxModelPositionError<0.01,{maximumMetres:maxModelPositionError});
const events=[];
for(const [laps,name]of [[0,'orbit1'],[1,'orbit2'],[2,'orbit3']]) {
  let lo=27*hz,hi=count-1;while(hi-lo>1){const m=(lo+hi)>>1;if(row(m)[22]<laps)lo=m;else hi=m;}
  const a=row(lo),b=row(hi);events.push({name,time:a[0]+(b[0]-a[0])*(laps-a[22])/(b[22]-a[22])});
}
gate('orbit-keyframes',Math.abs(events[0].time-28)<0.01 && Math.abs(events[1].time-33.097328)<0.01
  && Math.abs(events[2].time-41.373521)<0.01,events);
result.measurements.cameraAngularRate={degreesPerSecond:maxCameraRate,time:maxCameraRateTime,visualComfortApproval:false};

// Observe crossings of fixed axial planes from actual positions. No age-driven
// rings or synthetic screen motion are allowed to stand in for passage speed.
const crossings=[];
let previousLocal=local(row(Math.floor(7.5*hz)).slice(1,4));
let forwardOnly=true;
for(let i=Math.floor(7.5*hz)+1;i<=26*hz;i++) {
  const current=row(i),p=local(current.slice(1,4)),before=row(i-1);
  forwardOnly &&= p[2]>previousLocal[2];
  const first=Math.max(0,Math.floor((previousLocal[2]-l.ringStartZ)/world[20])+1);
  const last=Math.floor((p[2]-l.ringStartZ)/world[20]);
  for(let ordinal=first;ordinal<=last;ordinal++) {
    const z=l.ringStartZ+ordinal*world[20];
    crossings.push({ordinal,time:before[0]+(current[0]-before[0])*(z-previousLocal[2])/(p[2]-previousLocal[2])});
  }
  previousLocal=p;
}
const cadence=crossings.slice(1).map((c,i)=>1/(c.time-crossings[i].time));
gate('fixed-grid-passage',forwardOnly && Math.abs(crossings[0]?.time-7.6)<0.01
  && crossings.every((c,i)=>c.ordinal===i) && cadence.every((rate,i)=>i===0 || rate<=cadence[i-1]+0.001),
  {crossings:crossings.length,first:crossings[0],last:crossings.at(-1),firstHz:cadence[0],lastHz:cadence.at(-1)});
result.measurements.gridCadence={maxHz:Math.max(...cadence),maxCyclesPer30fpsFrame:Math.max(...cadence)/30,
  exceedsTemporalNyquist:Math.max(...cadence)>15,visualMotionApproval:false};

for(const frame of reference.frames) {
  const bytes=run(normal,Buffer.from(String(frame.time)));assert.equal(bytes.length,192000);
  const revision=openingRevision.frames.find(x=>x.time===frame.time);
  const actual=sha(bytes);
  let outside=null;
  if(revision) {
    const {left,right,top,bottom}=openingRevision.rectangle_inclusive;
    const h=crypto.createHash('sha256');
    for(let y=0;y<200;y++) {
      const start=y*320*3;
      if(y<top || y>bottom)h.update(bytes.subarray(start,start+320*3));
      else {
        h.update(bytes.subarray(start,start+left*3));
        h.update(bytes.subarray(start+(right+1)*3,start+320*3));
      }
    }
    outside=h.digest('hex');
  }
  result.openingFrames.push({...frame,actual,approvedGridRevision:revision||null,outside,
    pass:revision ? actual===revision.sha256 && outside===revision.original_outside_sha256 : actual===frame.sha256});
}
gate('protected-opening-rgb',result.openingFrames.every(x=>x.pass),{
  failedTimes:result.openingFrames.filter(x=>!x.pass).map(x=>x.time),
  approvedGridRevision:openingRevision.rectangle_inclusive,originalReferenceRetained:true});
const parityTimes=[5,16,29,35,57,65,70,77];
const parityFailures=parityTimes.filter(time=>{
  const input=Buffer.from(String(time));
  return !run(normal,input).equals(run(audit,input));
});
gate('instrumented-renderer-parity',parityFailures.length===0,{times:parityTimes,failedTimes:parityFailures});
const times=[4,5,7.6,11,16,20,23,26,27,28,29,30,31,32,33,34,35,36,37,38,39,40,41,45,50,57,60,65,70,75,77];
const queries=times.flatMap(t=>[0.75,1,1.5].map(aspect=>[t,aspect]));
const request=Buffer.alloc(16+queries.length*16);request[0]=84;
queries.forEach(([t,a],i)=>{request.writeDoubleLE(t,16+i*16);request.writeDoubleLE(a,24+i*16);});
const rendered=run(audit,request);assert.equal(rendered.subarray(0,8).toString(),'TDFRAME4');
assert.equal(rendered.length,8+queries.length*104);
for(let i=8;i<rendered.length;i+=104){const v=doubles(rendered.subarray(i,i+104));result.frames.push({time:v[0],aspect:v[1],wallMs:v[2],starPoints:v[3],starStreaks:v[4],ringEdges:v[5],spokeEdges:v[6],surfaceVisible:v[7],sun:[v[8],v[9]],projectionError:v[10],agl:v[11],renderCpuMs:v[12]});}
gate('post-emergence-points',result.frames.filter(x=>x.time>=5).every(x=>x.starStreaks===0),{});
gate('ring-visible-on-emergence',result.frames.filter(x=>x.time===5).every(x=>x.ringEdges>0),{});
gate('spokes-survive-entry',result.frames.filter(x=>x.time>=7.599 && x.time<=26).every(x=>x.spokeEdges>0),{});
gate('local-stars-hidden-in-square',result.frames.filter(x=>x.time>=26).every(x=>x.starPoints===0 && x.starStreaks===0),{});
gate('projection-ray-duality',result.frames.every(x=>x.projectionError<0.002),{maxPixels:Math.max(...result.frames.map(x=>x.projectionError))});
const sunWitnesses=[29,35].map(time=>{
  const frame=result.frames.find(f=>f.time===time && f.aspect===1);
  const rgb=run(normal,Buffer.from(String(time)));
  const [cx,cy]=frame.sun.map(Math.round);
  let sunPixels=0;
  if(cx>=10 && cx<310 && cy>=10 && cy<190) {
    for(let y=cy-9;y<=cy+9;y++)for(let x=cx-9;x<=cx+9;x++) {
      const index=(y*320+x)*3;
      if(rgb[index]>220 && rgb[index+1]>170 && rgb[index+2]>130)sunPixels++;
    }
  }
  return {time,position:frame.sun,sunPixels};
});
gate('rendered-sun-in-first-two-orbits',sunWitnesses.every(w=>w.sunPixels>200),sunWitnesses);
gate('frame-time-30fps',result.frames.every(x=>x.wallMs<1000/30),{metric:'monotonic wall time, including worker dispatch and RGB/depth assembly',worstFrames:result.frames.toSorted((a,b)=>b.wallMs-a.wallMs).slice(0,5)});
gate('playback-controls',run(audit,Buffer.from('_')).toString().includes('audit ok'),{});
result.status=Object.values(result.gates).every(x=>x.pass)?'PASS':'FAIL';
fs.mkdirSync('target/phase3-audit',{recursive:true});
fs.writeFileSync('target/phase3-audit/result.json',JSON.stringify(result,null,2)+'\n');
console.log(`OVERALL ${result.status}; target/phase3-audit/result.json`);
process.exitCode=result.status==='PASS'?0:1;
