//! Process management syscalls
use crate::{
    config::{MAX_SYSCALL_NUM, PAGE_SIZE},
    task::{
        change_program_brk, exit_current_and_run_next, suspend_current_and_run_next, TaskStatus, current_user_token, cnt_cursyscall_times, get_cursyscall_times,  get_first_scheduling,
    }, 
    timer::{get_time_us, get_time_ms},
    mm::{translated_ref_mut,mm_sys_mmap, mm_sys_munmap}, 
    syscall::{SYSCALL_EXIT, SYSCALL_YIELD, SYSCALL_GET_TIME, SYSCALL_TASK_INFO, SYSCALL_MMAP, SYSCALL_MUNMAP, SYSCALL_SBRK}
};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// Task information
#[allow(dead_code)]
pub struct TaskInfo {
    /// Task status in it's life cycle
    status: TaskStatus,
    /// The numbers of syscall called by task
    syscall_times: [u32; MAX_SYSCALL_NUM],
    /// Total running time of task
    time: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(_exit_code: i32) -> ! {
    trace!("kernel: sys_exit");
    cnt_cursyscall_times(SYSCALL_EXIT);
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    cnt_cursyscall_times(SYSCALL_YIELD);
    suspend_current_and_run_next();
    0
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(_ts: *mut TimeVal, _tz: usize) -> isize { //根据我的理解，系统调用是在内核空间发生的，也就说需要进行物理地址到逻辑地址的转换吗？
    trace!("kernel: sys_get_time");
    cnt_cursyscall_times(SYSCALL_GET_TIME);
    let us = get_time_us();
    let p_ts : *mut TimeVal = translated_ref_mut(current_user_token(), _ts);
    unsafe {
        *p_ts = TimeVal {
            sec: us / 1_000_000,
            usec: us % 1_000_000,
        };
    }
    0
}

/// YOUR JOB: Finish sys_task_info to pass testcases
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TaskInfo`] is splitted by two pages ?
pub fn sys_task_info(_ti: *mut TaskInfo) -> isize {
    trace!("kernel: sys_task_info NOT IMPLEMENTED YET!");
    cnt_cursyscall_times(SYSCALL_TASK_INFO);
    let p_ti : *mut TaskInfo = translated_ref_mut(current_user_token(), _ti);
    unsafe {
        *p_ti = TaskInfo {
            status: TaskStatus::Running,
            syscall_times: get_cursyscall_times(),
            time: get_time_ms() - get_first_scheduling(),
        }
    }
    0
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    trace!("kernel: sys_mmap NOT IMPLEMENTED YET!");
    cnt_cursyscall_times(SYSCALL_MMAP);
    /*
        start 没有按页大小对齐

        port & !0x7 != 0 (port 其余位必须为0)

        port & 0x7 = 0 (这样的内存无意义)

        [start, start + len) 中存在已经被映射的页

        物理内存不足
     */
    if _start % PAGE_SIZE != 0 || _port & !0x7 != 0 || _port & 0x7 == 0{
        return -1;
    }
    mm_sys_mmap(_start, _len, _port)
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    trace!("kernel: sys_munmap NOT IMPLEMENTED YET!");
    cnt_cursyscall_times(SYSCALL_MUNMAP);
    if _start % PAGE_SIZE != 0 {
        return -1;
    }
    mm_sys_munmap(_start, _len) 
}
/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel: sys_sbrk");
    cnt_cursyscall_times(SYSCALL_SBRK);
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}
