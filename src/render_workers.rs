//! Pixel-exact parallel execution of the ONE production renderer.
//!
//! Persistent Linux workers inherit the immutable world and have private scratch
//! rows/caches. Jobs contain only the same playback time and aspect ratio.
//! A shared queue balances indivisible row tiles without resampling.
//! The parent assembles both RGB and depth, preserving each pixel's draw order.
use super::*;

#[derive(Clone,Copy)]
struct Worker {pid:usize,command:usize,response:usize}
const EMPTY:Worker=Worker{pid:0,command:0,response:0};
static mut WORKERS:[Worker;3]=[EMPTY;3];
static mut COUNT:usize=0;
static mut READY:bool=false;
static mut CHILD:bool=false;
static mut ACTIVE:bool=false;
static mut TILE:usize=0;
#[cfg(feature="phase0-audit")] pub(super) static mut FRAME_CPU_NS:u64=0;

// Restrict raster ownership to the claimed tile; never move sample sites.
#[inline]
pub(super) fn next_row(y:i32)->i32 {
    let y=y.max(0);
    if !unsafe {ACTIVE} {return y;}
    let start=(unsafe {TILE}*field_snapshot::TILE_ROWS) as i32;
    let end=(start+field_snapshot::TILE_ROWS as i32).min(HEIGHT as i32);
    if y>=end {HEIGHT as i32}else{y.max(start)}
}

pub(super) fn parallel_region()->bool {unsafe {ACTIVE}}
pub(super) fn current_tile()->usize {unsafe {TILE}}

fn close(fd:usize) {syscall2(3,fd,0);}
fn transfer(fd:usize,pointer:*mut u8,length:usize,writing:bool)->bool {
    let mut used=0;
    while used<length {
        let n=syscall3(if writing {SYS_WRITE}else{SYS_READ},fd,pointer as usize+used,length-used);
        if n == -4 {continue;} // EINTR
        if n<=0 {return false;}
        used+=n as usize;
    }
    true
}
fn pipe()->Option<[usize;2]> {
    let mut fds=[0i32;2];
    if syscall2(22,fds.as_mut_ptr() as usize,0)<0 {None}
    else {Some([fds[0] as usize,fds[1] as usize])}
}
fn reap(pid:usize)->isize {
    loop {
        let result:isize;
        unsafe {asm!("syscall",inlateout("rax") 61isize=>result,
            in("rdi") pid,in("rsi") 0usize,in("rdx") 0usize,in("r10") 0usize,
            lateout("rcx") _,lateout("r11") _);}
        if result != -4 {return result;}
    }
}

pub(super) fn shutdown() {
    if unsafe {CHILD} {return;}
    let count=unsafe {COUNT};
    for i in 0..count {
        let w=unsafe {WORKERS[i]};
        // Closing response readers also releases a failed/blocked writer.
        close(w.command);close(w.response);
    }
    for i in 0..count {reap(unsafe {WORKERS[i].pid});}
    field_snapshot::shutdown();
    unsafe {COUNT=0;ACTIVE=false;TILE=0;}
}

fn initialize() {
    if unsafe {READY} {return;}
    unsafe {READY=true;}
    // Broken worker channels return EPIPE, allowing exact serial fallback.
    // Linux's rt_sigaction ABI uses an eight-byte kernel signal mask.
    let ignore=[1usize,0,0,0];
    let signal_result:isize;
    unsafe {asm!("syscall",inlateout("rax") 13isize=>signal_result,in("rdi") 13usize,
        in("rsi") ignore.as_ptr(),in("rdx") 0usize,in("r10") 8usize,
        lateout("rcx") _,lateout("r11") _);}
    if signal_result<0 {return;}
    let mut affinity=[0u8;128];
    let n=syscall3(204,0,affinity.len(),affinity.as_mut_ptr() as usize);
    let cpus=if n>0 {affinity.iter().map(|x|x.count_ones() as usize).sum()}else{1};
    let lanes=if cpus>=4 {4}else if cpus>=2 {2}else{1};
    if lanes>1 && !field_snapshot::initialize() {return;}
    for _ in 1..lanes {
        let Some(command)=pipe() else {shutdown();return;};
        let Some(response)=pipe() else {close(command[0]);close(command[1]);shutdown();return;};
        let pid=syscall2(57,0,0);
        if pid<0 {
            for fd in [command[0],command[1],response[0],response[1]] {close(fd);}
            shutdown();return;
        }
        if pid==0 {
            unsafe {CHILD=true;}
            close(command[1]);close(response[0]);
            for i in 0..unsafe {COUNT} {
                let old=unsafe {WORKERS[i]};close(old.command);close(old.response);
            }
            unsafe {COUNT=0;}
            worker_loop(command[0],response[1]);
        }
        close(command[0]);close(response[1]);
        unsafe {WORKERS[COUNT]=Worker{pid:pid as usize,command:command[1],response:response[0]};COUNT+=1;}
    }
}

fn measure(t:f32,aspect:f32)->[u64;7] {
    #[cfg(feature="phase0-audit")]
    unsafe {LANDSCAPE_HEIGHT_EVALUATIONS=0;SURFACE_CACHE_MISSES=0;}
    #[cfg(feature="phase0-audit")] let start=process_cpu_ns();
    render_frame_region(t,aspect);
    #[cfg(feature="phase0-audit")]
    return unsafe {[process_cpu_ns()-start,FRAME_STAGE_NS[0],FRAME_STAGE_NS[1],
        FRAME_STAGE_NS[2],FRAME_STAGE_NS[3],LANDSCAPE_HEIGHT_EVALUATIONS,SURFACE_CACHE_MISSES]};
    #[cfg(not(feature="phase0-audit"))] [0;7]
}

fn worker_loop(command:usize,response:usize)->! {
    let mut job=[0u8;8];
    while transfer(command,job.as_mut_ptr(),job.len(),false) {
        let t=f32::from_le_bytes([job[0],job[1],job[2],job[3]]);
        let aspect=f32::from_le_bytes([job[4],job[5],job[6],job[7]]);
        if !t.is_finite() || !aspect.is_finite() || aspect<=0.0 {break;}
        let mut stats=work_tiles(t,aspect);
        if !transfer(response,stats.as_mut_ptr().cast(),56,true) {break;}
    }
    close(command);close(response);exit(0)
}

fn work_tiles(t:f32,aspect:f32)->[u64;7] {
    let mut total=[0u64;7];
    while let Some(tile)=field_snapshot::claim() {
        unsafe {ACTIVE=true;TILE=tile;}
        let stats=measure(t,aspect);
        field_snapshot::publish(tile);
        for i in 0..7 {total[i]+=stats[i];}
    }
    unsafe {ACTIVE=false;}
    total
}

pub(super) fn render(t:f32,aspect:f32) {
    initialize();
    let count=unsafe {COUNT};
    #[cfg(feature="phase0-audit")]
    unsafe {LANDSCAPE_HEIGHT_EVALUATIONS=0;SURFACE_CACHE_MISSES=0;}
    #[cfg(feature="phase0-audit")] let prepare_start=process_cpu_ns();
    if count>0 && !field_snapshot::prepare(t,aspect) {
        shutdown();render(t,aspect);return;
    }
    #[cfg(feature="phase0-audit")] let prepare_cpu=process_cpu_ns()-prepare_start;
    #[cfg(feature="phase0-audit")] let prepare_queries=unsafe {LANDSCAPE_HEIGHT_EVALUATIONS};
    #[cfg(feature="phase0-audit")] let prepare_misses=unsafe {SURFACE_CACHE_MISSES};
    let mut job=[0u8;8];job[..4].copy_from_slice(&t.to_le_bytes());job[4..].copy_from_slice(&aspect.to_le_bytes());
    for i in 0..count {
        if !transfer(unsafe {WORKERS[i].command},job.as_mut_ptr(),8,true) {
            shutdown();render(t,aspect);return;
        }
    }
    let mut total=if count>0 {work_tiles(t,aspect)}else{measure(t,aspect)};
    #[cfg(feature="phase0-audit")] {
        total[0]+=prepare_cpu;total[1]+=prepare_cpu;
        total[5]+=prepare_queries;total[6]+=prepare_misses;
    }
    for i in 0..count {
        let response=unsafe {WORKERS[i].response};
        let mut stats=[0u64;7];
        if !transfer(response,stats.as_mut_ptr().cast(),56,false) {
            shutdown();render(t,aspect);return;
        }
        for k in 0..7 {total[k]+=stats[k];}
    }
    if count>0 {field_snapshot::collect();}
    #[cfg(feature="phase0-audit")]
    unsafe {FRAME_CPU_NS=total[0];
        core::ptr::copy_nonoverlapping(total.as_ptr().add(1),core::ptr::addr_of_mut!(FRAME_STAGE_NS).cast::<u64>(),4);
        LANDSCAPE_HEIGHT_EVALUATIONS=total[5];SURFACE_CACHE_MISSES=total[6];}
}

#[cfg(feature="phase0-audit")]
pub(super) fn serial(t:f32,aspect:f32) {
    unsafe {ACTIVE=false;}
    let stats=measure(t,aspect);
    unsafe {FRAME_CPU_NS=stats[0];}
}

#[cfg(feature="phase0-audit")]
pub(super) fn check_ownership_and_shutdown() {
    let workers=unsafe {WORKERS};let count=unsafe {COUNT};
    shutdown();
    // ECHILD proves these particular children were reaped, not left zombies.
    for worker in workers.iter().take(count) {assert!(reap(worker.pid)==-10);}
    let mut visits=[0u8;HEIGHT];
    for tile in 0..field_snapshot::TILES {
        unsafe {ACTIVE=true;TILE=tile;}
        let mut y=next_row(0);
        while y<HEIGHT as i32 {visits[y as usize]+=1;y=next_row(y+1);}
        // Atmosphere tiles are indivisible, and use the same partition.
        let mut y=next_row(0);
        while y<HEIGHT as i32 {
            assert!(y%4==0);
            for offset in 0..4 {assert!(next_row(y+offset)==y+offset);}
            y=next_row(y+4);
        }
    }
    assert!(visits.iter().all(|count|*count==1));
    unsafe {ACTIVE=false;TILE=0;READY=false;}
}

// Exercise real EPIPE/EOF fallback, not just an audit-only serial switch.
// Only terminates and reaps the audit's own private idle renderer child.
#[cfg(feature="phase0-audit")]
pub(super) fn fail_one_worker()->usize {
    initialize();
    if unsafe {COUNT}==0 {return 0;}
    let pid=unsafe {WORKERS[0].pid};
    assert!(syscall2(62,pid,9)==0); // kill(our_child, SIGKILL)
    assert!(reap(pid)==pid as isize);
    pid
}

#[cfg(feature="phase0-audit")]
pub(super) fn is_serial()->bool {unsafe {COUNT==0 && !ACTIVE}}
