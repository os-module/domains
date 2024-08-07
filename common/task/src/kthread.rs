use alloc::{collections::BTreeMap, string::ToString, sync::Arc};

use basic::{
    constants::signal::{SignalHandlers, SignalReceivers, SignalStack},
    println,
    sync::Mutex,
    task::{TaskContext, TaskContextExt},
    AlienResult,
};
use interface::VFS_ROOT_ID;
use memory_addr::VirtAddr;
use ptable::VmSpace;
use small_index::IndexAllocator;
use task_meta::{TaskBasicInfo, TaskMeta, TaskSchedulingInfo, TaskStatus};

use crate::{
    elf::VmmPageAllocator,
    processor::add_task,
    resource::{FdManager, HeapInfo, MMapInfo, ResourceLimits, TidHandle},
    task::{FsContext, Task, TaskInner},
    vfs_shim::{STDIN, STDOUT},
};

pub fn ktread_create(func: fn(), name: &str) -> AlienResult<()> {
    let tid = Arc::new(TidHandle::new().unwrap());
    let pid = tid.clone();

    let context = TaskContext::new_kernel(func as _, VirtAddr::from(0));
    let task_basic_info = TaskBasicInfo::new(tid.raw(), context);
    let scheduling_info = TaskSchedulingInfo::new(tid.raw(), 0, usize::MAX);
    let task_meta = TaskMeta::new(task_basic_info, scheduling_info);

    let k_stack_top = basic::add_one_task(task_meta)?;

    // fake kspace
    let kspace = VmSpace::<VmmPageAllocator>::new();
    let task = Task {
        tid,
        kernel_stack: k_stack_top,
        pid,
        address_space: Arc::new(Mutex::new(kspace)),
        fd_table: {
            let mut fd_table = FdManager::new();
            fd_table.insert(STDIN.clone());
            fd_table.insert(STDOUT.clone());
            fd_table.insert(STDOUT.clone());
            Arc::new(Mutex::new(fd_table))
        },
        threads: Arc::new(Mutex::new(IndexAllocator::new())),
        heap: Arc::new(Mutex::new(HeapInfo::new(0, 0))),
        inner: Mutex::new(TaskInner {
            name: name.to_string(),
            thread_number: 0,
            status: TaskStatus::Ready,
            parent: None,
            children: BTreeMap::new(),
            fs_info: FsContext::new(VFS_ROOT_ID, VFS_ROOT_ID),
            exit_code: 0,
            clear_child_tid: 0,
            // user mode stack info
            stack: 0..0,
            resource_limits: Mutex::new(ResourceLimits::default()),
            ss_stack: SignalStack {
                ss_sp: 0,
                ss_flags: 0x2,
                ss_size: 0,
            },
        }),
        send_sigchld_when_exit: false,
        mmap: Arc::new(Mutex::new(MMapInfo::new())),
        signal_handlers: Arc::new(Mutex::new(SignalHandlers::new())),
        signal_receivers: Arc::new(Mutex::new(SignalReceivers::new())),
    };
    let task = Arc::new(task);
    add_task(task);
    Ok(())
}

#[allow(unused)]
pub fn ktrhead_exit() {
    println!("kthread_exit");
}
