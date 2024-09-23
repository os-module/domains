use alloc::boxed::Box;

use rref::RRef;
use spin::Once;
use task_meta::TaskSchedulingInfo;

pub trait Scheduler: Send + Sync {
    fn add_task(&self, task_meta: RRef<TaskSchedulingInfo>);
    fn fetch_task(&self) -> Option<RRef<TaskSchedulingInfo>>;
    fn name(&self) -> &'static str;
}

pub struct GlobalScheduler {
    scheduler: Box<dyn Scheduler>,
}

impl GlobalScheduler {
    pub fn new(scheduler: Box<dyn Scheduler>) -> Self {
        Self { scheduler }
    }
}

impl GlobalScheduler {
    fn add_task(&self, task_meta: RRef<TaskSchedulingInfo>) {
        self.scheduler.add_task(task_meta);
    }

    fn fetch_task(&self, mut info: RRef<TaskSchedulingInfo>) -> RRef<TaskSchedulingInfo> {
        let res = self.scheduler.fetch_task();
        match res {
            Some(task) => task,
            None => {
                info.tid = usize::MAX;
                info
            }
        }
    }
}

static GLOBAL_SCHEDULER: Once<GlobalScheduler> = Once::new();

pub fn set_scheduler(scheduler: Box<dyn Scheduler>) {
    GLOBAL_SCHEDULER.call_once(|| GlobalScheduler::new(scheduler));
}

pub fn add_task(task_meta: RRef<TaskSchedulingInfo>) {
    // log::info!("<add_task>: {:?}", task_meta.lock().tid());
    GLOBAL_SCHEDULER.get().unwrap().add_task(task_meta);
}

pub fn fetch_task(info: RRef<TaskSchedulingInfo>) -> RRef<TaskSchedulingInfo> {
    GLOBAL_SCHEDULER.get().unwrap().fetch_task(info)
}
