//! Per-frame projected draw commands from the production field renderer.
//! The parent publishes an immutable snapshot BEFORE dispatching a frame;
//! all workers finish reading it BEFORE the next frame may overwrite it.
//! This is shared rendering scratch, not world geometry or a second camera.
use super::*;
use core::sync::atomic::{AtomicUsize,Ordering};

pub(super) const TILE_ROWS:usize=8;
pub(super) const TILES:usize=HEIGHT.div_ceil(TILE_ROWS);

type Point=(f32,f32,f32);
#[derive(Clone,Copy)]
struct Edge {a:Point,b:Point,color:(u8,u8,u8),alpha:u8,glow:u8}
// At most one ring and one rail edge per physical catalogue vertex.
const CAPACITY:usize=276*(GRID_LANES-1)*2;
struct Bin {count:usize,indices:[u32;CAPACITY]}
#[repr(C)]
struct Snapshot {
    count:usize,
    next:AtomicUsize,
    rgb:[u8;FRAME_BYTES],
    depth:[f32;PIXELS],
    output_rgb:[u8;FRAME_BYTES],
    output_depth:[f32;PIXELS],
    #[cfg(feature="phase0-audit")] stats:FieldRenderStats,
    edges:[Edge;CAPACITY],
    bins:[Bin;TILES],
}
static mut MEMORY:*mut Snapshot=core::ptr::null_mut();
static mut RECORDING:bool=false;
static mut OVERFLOW:bool=false;

pub(super) fn initialize()->bool {
    if !unsafe {MEMORY}.is_null() {return true;}
    let result:isize;
    unsafe {asm!("syscall",inlateout("rax") 9isize=>result,
        in("rdi") 0usize,in("rsi") core::mem::size_of::<Snapshot>(),
        in("rdx") 3usize,in("r10") 0x21usize,in("r8") usize::MAX,in("r9") 0usize,
        lateout("rcx") _,lateout("r11") _);}
    // mmap(PROT_READ|PROT_WRITE, MAP_SHARED|MAP_ANONYMOUS).
    if result<0 {return false;}
    unsafe {MEMORY=result as *mut Snapshot;}
    true
}

pub(super) fn shutdown() {
    let pointer=unsafe {MEMORY};
    if !pointer.is_null() {
        syscall2(11,pointer as usize,core::mem::size_of::<Snapshot>());
        unsafe {MEMORY=core::ptr::null_mut();}
    }
}

pub(super) fn prepare(t:f32,aspect:f32)->bool {
    let pointer=unsafe {MEMORY};
    if pointer.is_null() {return false;}
    unsafe {
        RECORDING=true;OVERFLOW=false;(*pointer).count=0;
        (*pointer).next.store(0,Ordering::SeqCst);
        for tile in 0..TILES {(*pointer).bins[tile].count=0;}
    }
    let camera=flow_camera(t);
    let exposure=field_star_exposure_camera(t,&camera);
    clear_depth_buffer();
    render_universe_field(&exposure,&camera,aspect);
    unsafe {
        // This frame contains background/stars only: grid calls were recorded.
        core::ptr::copy_nonoverlapping(core::ptr::addr_of!(FRAME).cast::<u8>(),
            core::ptr::addr_of_mut!((*pointer).rgb).cast::<u8>(),FRAME_BYTES);
        core::ptr::copy_nonoverlapping(core::ptr::addr_of!(DEPTH).cast::<f32>(),
            core::ptr::addr_of_mut!((*pointer).depth).cast::<f32>(),PIXELS);
        #[cfg(feature="phase0-audit")] {(*pointer).stats=FIELD_RENDER_STATS;}
        RECORDING=false;
        !OVERFLOW
    }
}

pub(super) fn edge(a:Point,b:Point,color:(u8,u8,u8),alpha:u8,glow:u8) {
    if unsafe {RECORDING} {
        let pointer=unsafe {MEMORY};
        let count=unsafe {(*pointer).count};
        if count==CAPACITY {unsafe {OVERFLOW=true;}return;}
        unsafe {
            core::ptr::addr_of_mut!((*pointer).edges).cast::<Edge>().add(count)
                .write(Edge{a,b,color,alpha,glow});
            (*pointer).count=count+1;
            // Bin indices retain the original draw order. A bin is only a
            // conservative screen-row work list, never a geometry simplifier.
            let top=((a.1.min(b.1)-3.0) as i32-1).max(0);
            let bottom=((a.1.max(b.1)+3.0) as i32+1).min(HEIGHT as i32-1);
            if top<=bottom {
                for tile in top as usize/TILE_ROWS..=bottom as usize/TILE_ROWS {
                    let bin=core::ptr::addr_of_mut!((*pointer).bins).cast::<Bin>().add(tile);
                    core::ptr::addr_of_mut!((*bin).indices).cast::<u32>().add((*bin).count).write(count as u32);
                    (*bin).count+=1;
                }
            }
        }
    }else{
        let frame=core::ptr::addr_of_mut!(FRAME).cast::<u8>();
        draw_bloom_line(frame,(a.0,a.1),(b.0,b.1),color,alpha,glow);
        record_depth_line((a.0,a.1),(b.0,b.1),a.2,b.2);
    }
}

pub(super) fn replay()->bool {
    if unsafe {RECORDING} || !render_workers::parallel_region() {return false;}
    let pointer=unsafe {MEMORY};
    if pointer.is_null() {return false;}
    let tile=render_workers::current_tile();
    let start=tile*TILE_ROWS*WIDTH;
    let pixels=TILE_ROWS.min(HEIGHT-tile*TILE_ROWS)*WIDTH;
    unsafe {
        core::ptr::copy_nonoverlapping(core::ptr::addr_of!((*pointer).rgb).cast::<u8>().add(start*3),
            core::ptr::addr_of_mut!(FRAME).cast::<u8>().add(start*3),pixels*3);
        core::ptr::copy_nonoverlapping(core::ptr::addr_of!((*pointer).depth).cast::<f32>().add(start),
            core::ptr::addr_of_mut!(DEPTH).cast::<f32>().add(start),pixels);
        #[cfg(feature="phase0-audit")] {FIELD_RENDER_STATS=(*pointer).stats;}
        let bin=core::ptr::addr_of!((*pointer).bins).cast::<Bin>().add(tile);
        for i in 0..(*bin).count {
            let index=core::ptr::addr_of!((*bin).indices).cast::<u32>().add(i).read() as usize;
            let e=core::ptr::addr_of!((*pointer).edges).cast::<Edge>().add(index).read();
            edge(e.a,e.b,e.color,e.alpha,e.glow);
        }
    }
    true
}

// Every tile is claimed once. No barrier is needed inside a frame: workers
// write disjoint output rows and report completion through their existing pipe.
pub(super) fn claim()->Option<usize> {
    let tile=unsafe {(*MEMORY).next.fetch_add(1,Ordering::SeqCst)};
    if tile<TILES {Some(tile)}else{None}
}

pub(super) fn publish(tile:usize) {
    let start=tile*TILE_ROWS*WIDTH;
    let pixels=TILE_ROWS.min(HEIGHT-tile*TILE_ROWS)*WIDTH;
    unsafe {
        core::ptr::copy_nonoverlapping(core::ptr::addr_of!(FRAME).cast::<u8>().add(start*3),
            core::ptr::addr_of_mut!((*MEMORY).output_rgb).cast::<u8>().add(start*3),pixels*3);
        core::ptr::copy_nonoverlapping(core::ptr::addr_of!(DEPTH).cast::<f32>().add(start),
            core::ptr::addr_of_mut!((*MEMORY).output_depth).cast::<f32>().add(start),pixels);
    }
}

// Parent only, AFTER every worker's completion response. On any worker error
// this snapshot is discarded and the complete frame is rendered serially.
pub(super) fn collect() {
    unsafe {
        core::ptr::copy_nonoverlapping(core::ptr::addr_of!((*MEMORY).output_rgb).cast::<u8>(),
            core::ptr::addr_of_mut!(FRAME).cast::<u8>(),FRAME_BYTES);
        core::ptr::copy_nonoverlapping(core::ptr::addr_of!((*MEMORY).output_depth).cast::<f32>(),
            core::ptr::addr_of_mut!(DEPTH).cast::<f32>(),PIXELS);
    }
}
