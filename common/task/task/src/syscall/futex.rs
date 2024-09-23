use alloc::sync::Arc;

use basic::{
    constants::{ipc::FutexOp, time::TimeSpec},
    sync::Mutex,
    time::{TimeNow, ToClock},
    AlienError, AlienResult,
};
use memory_addr::VirtAddr;
use ptable::VmIo;

use crate::{
    futex::{FutexWaitManager, FutexWaiter},
    processor::current_task,
};

pub static FUTEX_WAITER: Mutex<FutexWaitManager> = Mutex::new(FutexWaitManager::new());

pub fn futex(
    uaddr: usize,
    futex_op: u32,
    val: u32,
    val2: usize,
    uaddr2: usize,
    val3: u32,
) -> AlienResult<isize> {
    let futex_op = FutexOp::try_from(futex_op).unwrap();
    let task = current_task().unwrap();
    trace!(
        "futex: {:#x?} {:?} {:?} {:?} {:?} {:?}",
        uaddr,
        futex_op,
        val,
        val2,
        uaddr2,
        val3
    );
    match futex_op {
        FutexOp::FutexWaitPrivate | FutexOp::FutexWait => {
            let u_value = task
                .address_space
                .lock()
                .read_value_atomic(VirtAddr::from(uaddr))
                .unwrap();
            if u_value != val as usize {
                return Err(AlienError::EAGAIN);
            }
            let wait_time = if val2 != 0 {
                let time_spec = task.read_val_from_user::<TimeSpec>(VirtAddr::from(val2))?;
                Some(time_spec.to_clock() + TimeSpec::now().to_clock())
            } else {
                // wait forever
                None
            };
            warn!("Futex wait time: {:?}", wait_time);
            let timeout_flag = Arc::new(Mutex::new(false));
            let tid = task.tid();
            let waiter = FutexWaiter::new(tid, wait_time, timeout_flag.clone());
            FUTEX_WAITER.lock().add_waiter(uaddr, waiter);
            // switch to other task
            basic::wait_now()?;
            warn!("Because of futex, we switch to other task");
            // checkout the timeout flag
            let timeout_flag = timeout_flag.lock();
            if *timeout_flag {
                return Ok(0);
            }
        }
        FutexOp::FutexCmpRequeuePiPrivate => {
            let u_value = task
                .address_space
                .lock()
                .read_value_atomic(VirtAddr::from(uaddr))
                .unwrap();
            if u_value != val3 as usize {
                error!("FutexRequeuePrivate: uaddr_ref != val");
                return Err(AlienError::EAGAIN);
            }
            // wake val tasks
            let res = FUTEX_WAITER.lock().wake(uaddr, val as usize)?;
            // requeue val2 tasks to uaddr2
            let res2 = FUTEX_WAITER.lock().requeue(uaddr2, val2, uaddr)?;
            return Ok(res2 as isize + res as isize);
        }
        FutexOp::FutexRequeuePrivate => {
            // wake val tasks
            let res = FUTEX_WAITER.lock().wake(uaddr, val as usize)?;
            // requeue val2 tasks to uaddr2
            let res2 = FUTEX_WAITER.lock().requeue(uaddr2, val2, uaddr)?;
            return Ok(res2 as isize + res as isize);
        }
        FutexOp::FutexWakePrivate | FutexOp::FutexWake => {
            let res = FUTEX_WAITER.lock().wake(uaddr, val as usize)?;
            return Ok(res as isize);
        }
        _ => {
            panic!("futex: unimplemented futex_op: {:?}", futex_op);
        }
    }
    Ok(0)
}
