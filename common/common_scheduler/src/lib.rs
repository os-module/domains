#![no_std]
#![forbid(unsafe_code)]

mod scheduler;

extern crate alloc;

use alloc::{boxed::Box, vec::Vec};

use basic::{println, AlienResult};
use interface::{Basic, SchedulerDomain};
use rref::RRef;
pub use scheduler::Scheduler;
use task_meta::TaskSchedulingInfo;

#[derive(Debug)]
pub struct CommonSchedulerDomain {
    name: &'static str,
}

impl CommonSchedulerDomain {
    pub fn new(global_scheduler: Box<dyn Scheduler>) -> Self {
        let name = global_scheduler.name();
        scheduler::set_scheduler(global_scheduler);
        Self { name }
    }
}

impl Basic for CommonSchedulerDomain {
    fn domain_id(&self) -> u64 {
        rref::domain_id()
    }
}

impl SchedulerDomain for CommonSchedulerDomain {
    fn init(&self) -> AlienResult<()> {
        println!("SchedulerDomain init, name: {}", self.name);
        Ok(())
    }

    fn add_task(&self, scheduling_info: RRef<TaskSchedulingInfo>) -> AlienResult<()> {
        scheduler::add_task(scheduling_info);
        Ok(())
    }

    fn fetch_task(&self, info: RRef<TaskSchedulingInfo>) -> AlienResult<RRef<TaskSchedulingInfo>> {
        Ok(scheduler::fetch_task(info))
    }

    fn dump_meta_data(&self) -> AlienResult<Vec<RRef<TaskSchedulingInfo>>> {
        scheduler::dump_meta_data()
    }
    fn rebuild_from_meta_data(
        &self,
        meta_data: &mut Vec<RRef<TaskSchedulingInfo>>,
    ) -> AlienResult<()> {
        scheduler::rebuild_from_meta_data(meta_data)
    }
}
