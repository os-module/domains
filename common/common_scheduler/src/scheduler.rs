use alloc::{boxed::Box, vec::Vec};

use basic::{sync::Mutex, AlienResult};
use rref::RRef;
use task_meta::TaskSchedulingInfo;

pub trait Scheduler: Send + Sync {
    fn add_task(&mut self, task_meta: RRef<TaskSchedulingInfo>);
    fn fetch_task(&mut self) -> Option<RRef<TaskSchedulingInfo>>;
    fn name(&self) -> &'static str;
    fn dump_meta_data(&mut self) -> AlienResult<Vec<RRef<TaskSchedulingInfo>>>;
    fn rebuild_from_meta_data(
        &mut self,
        meta_data: &mut Vec<RRef<TaskSchedulingInfo>>,
    ) -> AlienResult<()>;
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
    fn dump_meta_data(&mut self) -> AlienResult<Vec<RRef<TaskSchedulingInfo>>> {
        self.scheduler.as_mut().unwrap().dump_meta_data()
    }

    fn rebuild_from_meta_data(
        &mut self,
        meta_data: &mut Vec<RRef<TaskSchedulingInfo>>,
    ) -> AlienResult<()> {
        self.scheduler
            .as_mut()
            .unwrap()
            .rebuild_from_meta_data(meta_data)
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

pub fn dump_meta_data() -> AlienResult<Vec<RRef<TaskSchedulingInfo>>> {
    GLOBAL_SCHEDULER.lock().dump_meta_data()
}

pub fn rebuild_from_meta_data(meta_data: &mut Vec<RRef<TaskSchedulingInfo>>) -> AlienResult<()> {
    GLOBAL_SCHEDULER.lock().rebuild_from_meta_data(meta_data)
}
