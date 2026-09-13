// Read-only screening of the complete, unchanged production star catalogue.
// This compares proposed spatial routes; it does not select or install one.
'use strict';
const assert = require('node:assert/strict');
const { spawnSync } = require('node:child_process');
const study = require('./phase2-flight-study.cjs');
const dot = (a,b) => a.reduce((s,x,i)=>s+x*b[i],0);
const unit = a => a.map(x=>x/Math.hypot(...a));

function readCatalogue(binary='target/phase0-audit/x86_64-unknown-linux-gnu/release/termdemo') {
  const run=spawnSync(binary,[],{input:Buffer.from('I'),maxBuffer:16*1024*1024,timeout:60000});
  assert.equal(run.error,undefined,String(run.error));
  assert.equal(run.status,0,run.stderr.toString());
  const b=run.stdout;
  assert.equal(b.subarray(0,8).toString(),'TDALLS01');
  assert.equal((b.length-104)%40,0);
  const vector=offset=>[0,1,2].map(j=>b.readDoubleLE(offset+8*j));
  const origin=vector(8),forward=vector(32),right=vector(56),down=vector(80);
  const f=unit(forward),r=unit(right),d=unit(down),stars=[];
  for(let offset=104;offset<b.length;offset+=40) {
    const p=vector(offset+16).map((x,j)=>x-origin[j]);
    stars.push({cell:b.readDoubleLE(offset),object:b.readDoubleLE(offset+8),
      relative:[dot(p,r),dot(p,d),dot(p,f)]});
  }
  return {origin,forward,right,down,stars,
    horizontalCalibration:Math.hypot(...right)/Math.hypot(...forward),
    verticalCalibration:Math.hypot(...down)/Math.hypot(...forward)};
}

function rotation(angle,azimuth,p) {
  const ex=Math.cos(azimuth),ey=Math.sin(azimuth),a=ex*p[0]+ey*p[1];
  const delta=(Math.cos(angle)-1)*a+Math.sin(angle)*p[2];
  return [p[0]+ex*delta,p[1]+ey*delta,Math.cos(angle)*p[2]-Math.sin(angle)*a];
}

function screenCandidates() {
  const speed=study.fitSpeed(),terrain=study.terrainReference(),catalogue=readCatalogue();
  const base=study.fitRoute(speed,terrain,{bendLength:0,weave:0}),table=study.arcTable(base,2);
  const theta=t=>table.theta(study.integral(speed.speed,4,t,.002));
  const theta5=theta(5),thetaEntry=theta(7.6),scale=speed.v4/18;
  const all=[];
  for(let degrees=40;degrees<=100;degrees+=5) for(let length=100e6;length<=220e6;length+=20e6) {
    // A screening preference, NOT a new speed or angular-speed specification.
    // Report it so rejected search space is not mistaken for impossibility.
    const angularRateBound=1.875*degrees*speed.v4/length;
    if(angularRateBound>85) continue;
    const angle=degrees*Math.PI/180,route=study.spatialBend(base,base.theta4,length,0,angle);
    const p4=route.state(base.theta4)[0],p5=route.state(theta5)[0],p7=route.state(thetaEntry)[0];
    const d5=route.state(theta5)[1],heading5=Math.atan2(d5[0],d5[2]);
    for(let azimuthDegrees=0;azimuthDegrees<360;azimuthDegrees+=10) {
      const az=azimuthDegrees*Math.PI/180,ex=Math.cos(az),ey=Math.sin(az);
      const place=p=>[ex*p[0],p[1]+ey*p[0],p[2]];
      const start=place(p4),camera=place(p7),exit=place(p5);
      const r=rotation(angle,az,[-1,0,0]),d=rotation(angle,az,[0,-1,0]),f=rotation(angle,az,[0,0,1]);
      const rel=camera.map((x,i)=>x-exit[i]);
      const depth=dot(rel,rotation(heading5,az,[0,0,1]));
      const ring=[160+154*catalogue.horizontalCalibration*dot(rel,rotation(heading5,az,[-1,0,0]))/depth,
        100+154*catalogue.verticalCalibration*dot(rel,rotation(heading5,az,[0,-1,0]))/depth];
      if(depth<=0||ring[0]<12||ring[0]>308||ring[1]<12||ring[1]>188)continue;
      let visible=0,moving=0,maximum=0;
      for(const star of catalogue.stars) {
        const [x,y,z]=star.relative.map(x=>x*scale);
        const q=start.map((p,i)=>p-camera[i]+r[i]*x+d[i]*y+f[i]*z);
        const px=-154*catalogue.horizontalCalibration*q[0]/q[2];
        const py=-154*catalogue.verticalCalibration*q[1]/q[2];
        if(q[2]<=0||Math.abs(px)>=160||Math.abs(py)>=100)continue;
        visible++;
        const drift=speed.speed(7.6)*Math.hypot(px,py)/q[2];
        if(drift>1)moving++;
        maximum=Math.max(maximum,drift);
      }
      all.push({degrees,length,azimuthDegrees,angularRateBound,ringAt5:ring,
        visibleAtEntry:visible,movingOverOnePixelPerSecond:moving,maximumTranslationPixelsPerSecond:maximum});
    }
  }
  all.sort((a,b)=>a.movingOverOnePixelPerSecond-b.movingOverOnePixelPerSecond
    ||a.maximumTranslationPixelsPerSecond-b.maximumTranslationPixelsPerSecond);
  return {catalogueCount:catalogue.stars.length,candidatesRetainingRingVisibility:all.length,
    best:all.slice(0,20),installed:false,
    limitations:[
      'Screening is not a feasibility proof over all routes or all free speed/metric choices.',
      'The 85-degree/s bound and 1-pixel/s count are diagnostics, not user specifications.',
      'Rotated out-of-plane candidates require a full arc refit and containment/clearance checks before use.',
      'An ideal projection comparison is not an exact protected-opening raster comparison.',
      'No star is moved, faded, masked, or deleted by this study; no candidate has been connected live.',
    ]};
}
if(require.main===module)console.log(JSON.stringify(screenCandidates(),null,2));
module.exports={readCatalogue,rotation,screenCandidates};
