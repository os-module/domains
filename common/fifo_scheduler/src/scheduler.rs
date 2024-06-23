use alloc::{collections::VecDeque, sync::Arc};

use basic::{arch::hart_id, sync::Mutex};
use common_scheduler::Scheduler;
use rref::RRef;
use storage::DataStorageHeap;
use task_meta::TaskSchedulingInfo;

type __TaskList = Mutex<VecDeque<RRef<TaskSchedulingInfo>, DataStorageHeap>>;
type TaskList = Arc<__TaskList, DataStorageHeap>;
#[derive(Debug)]
pub struct CustomFiFoScheduler {
    tasks: TaskList,
}

impl CustomFiFoScheduler {
    pub fn new() -> Self {
        let task_list = storage::get_or_insert_with_data::<__TaskList, _>("tasks", || {
            __TaskList::new(VecDeque::new_in(DataStorageHeap))
        });
        Self { tasks: task_list }
    }
}

impl Scheduler for CustomFiFoScheduler {
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
        if let Some(index) = res {
            return tasks.remove(index);
        }
        None
    }
    fn name(&self) -> &'static str {
        "FiFoScheduler"
    }
}
