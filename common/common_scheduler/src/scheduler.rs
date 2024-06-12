use alloc::boxed::Box;

use basic::sync::Mutex;
use rref::RRef;
use task_meta::TaskSchedulingInfo;

pub trait Scheduler: Send + Sync {
    fn add_task(&mut self, task_meta: RRef<TaskSchedulingInfo>);
    fn fetch_task(&mut self) -> Option<RRef<TaskSchedulingInfo>>;
    fn name(&self) -> &'static str;
}

pub struct GlobalScheduler {
    scheduler: Option<Box<dyn Scheduler>>,
}

impl GlobalScheduler {
    pub fn set_scheduler(&mut self, scheduler: Box<dyn Scheduler>) {
        self.scheduler = Some(scheduler);
    }
}

impl GlobalScheduler {
    fn add_task(&mut self, task_meta: RRef<TaskSchedulingInfo>) {
        self.scheduler.as_mut().unwrap().add_task(task_meta);
    }

    fn fetch_task(&mut self, mut info: RRef<TaskSchedulingInfo>) -> RRef<TaskSchedulingInfo> {
        let res = self.scheduler.as_mut().unwrap().fetch_task();
        match res {
            Some(task) => task,
            None => {
                info.tid = usize::MAX;
                info
            }
        }
    }
}

static GLOBAL_SCHEDULER: Mutex<GlobalScheduler> = Mutex::new(GlobalScheduler { scheduler: None });

pub fn set_scheduler(scheduler: Box<dyn Scheduler>) {
    GLOBAL_SCHEDULER.lock().set_scheduler(scheduler);
}

pub fn add_task(task_meta: RRef<TaskSchedulingInfo>) {
    // log::info!("<add_task>: {:?}", task_meta.lock().tid());
    GLOBAL_SCHEDULER.lock().add_task(task_meta);
}

pub fn fetch_task(info: RRef<TaskSchedulingInfo>) -> RRef<TaskSchedulingInfo> {
    GLOBAL_SCHEDULER.lock().fetch_task(info)
}
