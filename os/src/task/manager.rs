//!Implementation of [`TaskManager`]
use super::TaskControlBlock;
use crate::sync::UPSafeCell;
use alloc::collections::VecDeque;
use alloc::sync::Arc;
use lazy_static::*;
use crate::config::BIG_STRIDE;
///A array of `TaskControlBlock` that is thread-safe
pub struct TaskManager {
    ready_queue: VecDeque<Arc<TaskControlBlock>>,
}

/// A simple FIFO scheduler.
impl TaskManager {
    ///Creat an empty TaskManager
    pub fn new() -> Self {
        Self {
            ready_queue: VecDeque::new(),
        }
    }
    /// Add process back to ready queue
    pub fn add(&mut self, task: Arc<TaskControlBlock>) {
        let mut innner = task.inner_exclusive_access();
        innner.stride += BIG_STRIDE / innner.priority;
        drop(innner);
        self.ready_queue.push_back(task);
    }
    /// Take a process out of the ready queue
    pub fn fetch(&mut self) -> Option<Arc<TaskControlBlock>> {
        let mut mn : isize = -1;
        let mut cnt = 0;
        let mut _prio = -1;
        for i in &mut self.ready_queue {
            let inner = i.inner_exclusive_access();
            if mn == -1 {
                mn = cnt;
                _prio = inner.stride;
            }
            else {
                if _prio > inner.stride {
                    _prio = inner.stride;
                    mn = cnt;
                }
            }
            cnt += 1;
        }
        if mn == -1 {
            return None;
        }
        let tcb = self.ready_queue[mn as usize].clone();
        self.ready_queue.remove(mn as usize);
        Some(tcb) 
    }
}

lazy_static! {
    /// TASK_MANAGER instance through lazy_static!
    pub static ref TASK_MANAGER: UPSafeCell<TaskManager> =
        unsafe { UPSafeCell::new(TaskManager::new()) };
}

/// Add process to ready queue
pub fn add_task(task: Arc<TaskControlBlock>) {
    //trace!("kernel: TaskManager::add_task");
    TASK_MANAGER.exclusive_access().add(task);
}

/// Take a process out of the ready queue
pub fn fetch_task() -> Option<Arc<TaskControlBlock>> {
    //trace!("kernel: TaskManager::fetch_task");
    TASK_MANAGER.exclusive_access().fetch()
}
