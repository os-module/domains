use alloc::collections::VecDeque;

use basic::arch::hart_id;
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
}
