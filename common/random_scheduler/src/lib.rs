#![feature(allocator_api)]
#![no_std]
#![forbid(unsafe_code)]
extern crate alloc;
use alloc::{boxed::Box, collections::VecDeque, sync::Arc};
use core::sync::atomic::AtomicBool;

use basic::{arch::hart_id, println, sync::Mutex};
use common_scheduler::{CommonSchedulerDomain, Scheduler};
use interface::SchedulerDomain;
use rref::RRef;
use storage::DataStorageHeap;
use task_meta::TaskSchedulingInfo;

type __TaskList = Mutex<VecDeque<RRef<TaskSchedulingInfo>, DataStorageHeap>>;
type TaskList = Arc<__TaskList, DataStorageHeap>;
#[derive(Debug)]
pub struct RandomScheduler {
    tasks: TaskList,
}

impl RandomScheduler {
    pub fn new() -> Self {
        println!("RandomScheduler: new");
        let task_list = storage::get_data::<__TaskList>("tasks").unwrap();
        let len = task_list.lock().len();
        println!("RandomScheduler: The task list len is {}", len);
        Self { tasks: task_list }
    }
}

impl Scheduler for RandomScheduler {
    fn add_task(&self, task_meta: RRef<TaskSchedulingInfo>) {
        self.tasks.lock().push_back(task_meta);
    }

    fn fetch_task(&self) -> Option<RRef<TaskSchedulingInfo>> {
        let hart_id = hart_id();
        let mut tasks = self.tasks.lock();
        let res = tasks
            .iter()
            .position(|info| info.cpus_allowed & (1 << hart_id) != 0);
        // println!("fetch_task: {:?}, len: {}", res, self.tasks.len());
        static FETCH_MASK: AtomicBool = AtomicBool::new(false);
        if !FETCH_MASK.swap(true, core::sync::atomic::Ordering::Relaxed) {
            println!("fetch_task: {:?}, len: {}", res, tasks.len());
        }
        if let Some(index) = res {
            return tasks.remove(index);
        }
        None
    }

    fn name(&self) -> &'static str {
        "FiFoScheduler"
    }
}

pub fn main() -> Box<dyn SchedulerDomain> {
    Box::new(CommonSchedulerDomain::new(Box::new(RandomScheduler::new())))
}
