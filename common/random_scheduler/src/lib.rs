#![no_std]
#![forbid(unsafe_code)]
extern crate alloc;
use alloc::{boxed::Box, collections::VecDeque, vec::Vec};
use core::{ops::Deref, sync::atomic::AtomicBool};

use basic::{arch::hart_id, println, AlienResult};
use common_scheduler::{CommonSchedulerDomain, Scheduler};
use interface::SchedulerDomain;
use rref::RRef;
use task_meta::TaskSchedulingInfo;

#[derive(Debug)]
pub struct RandomScheduler {
    fetch_mask: bool,
    tasks: VecDeque<RRef<TaskSchedulingInfo>>,
}

impl RandomScheduler {
    pub const fn new() -> Self {
        Self {
            fetch_mask: false,
            tasks: VecDeque::new(),
        }
    }
}

impl Scheduler for RandomScheduler {
    fn add_task(&mut self, task_meta: RRef<TaskSchedulingInfo>) {
        self.tasks.push_back(task_meta);
    }

    fn fetch_task(&mut self) -> Option<RRef<TaskSchedulingInfo>> {
        let hart_id = hart_id();
        let res = self
            .tasks
            .iter()
            .position(|info| info.cpus_allowed & (1 << hart_id) != 0);
        static FETCH_MASK: AtomicBool = AtomicBool::new(false);
        if !FETCH_MASK.swap(true, core::sync::atomic::Ordering::Relaxed) {
            println!("fetch_task: {:?}, len: {}", res, self.tasks.len());
        }
        if let Some(index) = res {
            return self.tasks.remove(index);
        }
        None
    }

    fn name(&self) -> &'static str {
        "RandomScheduler"
    }

    fn dump_meta_data(&mut self) -> AlienResult<Vec<RRef<TaskSchedulingInfo>>> {
        let mut res = Vec::new();
        while let Some(task) = self.tasks.pop_front() {
            res.push(task);
        }
        Ok(res)
    }

    fn rebuild_from_meta_data(
        &mut self,
        meta_data: &mut Vec<RRef<TaskSchedulingInfo>>,
    ) -> AlienResult<()> {
        meta_data.iter().for_each(|task_meta_data| {
            let new_task = task_meta_data.deref().clone();
            self.tasks.push_back(RRef::new(new_task));
        });
        Ok(())
    }
}

pub fn main() -> Box<dyn SchedulerDomain> {
    Box::new(CommonSchedulerDomain::new(Box::new(RandomScheduler::new())))
}
