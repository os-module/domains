use alloc::{collections::VecDeque, vec::Vec};
use core::ops::Deref;

use basic::{arch::hart_id, AlienResult};
use common_scheduler::Scheduler;
use rref::RRef;
use task_meta::TaskSchedulingInfo;

#[derive(Debug)]
pub struct FiFoScheduler {
    tasks: VecDeque<RRef<TaskSchedulingInfo>>,
}

impl FiFoScheduler {
    pub const fn new() -> Self {
        Self {
            tasks: VecDeque::new(),
        }
    }
}

impl Scheduler for FiFoScheduler {
    fn add_task(&mut self, task_meta: RRef<TaskSchedulingInfo>) {
        self.tasks.push_back(task_meta);
    }

    fn fetch_task(&mut self) -> Option<RRef<TaskSchedulingInfo>> {
        let hart_id = hart_id();
        let res = self
            .tasks
            .iter()
            .position(|info| info.cpus_allowed & (1 << hart_id) != 0);
        // println!("fetch_task: {:?}, len: {}", res, self.tasks.len());
        if let Some(index) = res {
            return self.tasks.remove(index);
        }
        None
    }

    fn name(&self) -> &'static str {
        "FiFoScheduler"
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
