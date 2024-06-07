#![no_std]
#![forbid(unsafe_code)]
extern crate alloc;
use alloc::{boxed::Box, collections::VecDeque};

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
        if self.fetch_mask {
            self.fetch_mask = false;
            self.tasks.pop_front()
        } else {
            self.fetch_mask = true;
            self.tasks.pop_back()
        }
    }

    fn name(&self) -> &'static str {
        "RandomScheduler"
    }
}

pub fn main() -> Box<dyn SchedulerDomain> {
    Box::new(CommonSchedulerDomain::new(Box::new(RandomScheduler::new())))
}
