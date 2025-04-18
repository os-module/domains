#![feature(atomic_from_mut)]
#![no_std]
#![forbid(unsafe_code)]
extern crate alloc;
#[macro_use]
extern crate log;
mod elf;
mod futex;
mod init;
mod kthread;
mod processor;
mod resource;
mod syscall;
mod task;
mod utils;
mod vfs_shim;

use alloc::{boxed::Box, sync::Arc};
use core::ops::Range;

use basic::{println, AlienError, AlienResult};
use interface::{
    define_unwind_for_TaskDomain, Basic, DomainType, InodeID, TaskDomain, TmpHeapInfo,
};
use memory_addr::VirtAddr;
use shared_heap::{DBox, DVec};

use crate::{processor::current_task, vfs_shim::ShimFile};

#[derive(Debug)]
pub struct TaskDomainImpl {}

impl Default for TaskDomainImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskDomainImpl {
    pub fn new() -> Self {
        Self {}
    }
}

impl Basic for TaskDomainImpl {
    fn domain_id(&self) -> u64 {
        shared_heap::domain_id()
    }
}

impl TaskDomain for TaskDomainImpl {
    fn init(&self) -> AlienResult<()> {
        let vfs_domain = basic::get_domain("vfs").unwrap();
        let vfs_domain = match vfs_domain {
            DomainType::VfsDomain(vfs_domain) => vfs_domain,
            _ => panic!("vfs domain not found"),
        };
        vfs_shim::init_vfs_domain(vfs_domain);
        init::init_task();
        println!("Init task domain success");
        Ok(())
    }

    fn satp_with_trap_frame_virt_addr(&self) -> AlienResult<(usize, usize)> {
        let task = current_task().unwrap();
        let addr = task.trap_frame_virt_ptr();
        let token = task.token();
        Ok((token, addr.as_usize()))
    }

    fn trap_frame_phy_addr(&self) -> AlienResult<usize> {
        let task = current_task().unwrap();
        Ok(task.trap_frame_phy_ptr().as_usize())
    }

    fn heap_info(&self, mut tmp_heap_info: DBox<TmpHeapInfo>) -> AlienResult<DBox<TmpHeapInfo>> {
        let task = current_task().unwrap();
        let guard = task.heap.lock();
        *tmp_heap_info = TmpHeapInfo {
            start: guard.start,
            current: guard.current,
        };
        Ok(tmp_heap_info)
    }

    fn get_fd(&self, fd: usize) -> AlienResult<InodeID> {
        let task = current_task().unwrap();
        let file = task.get_file(fd).ok_or(AlienError::EBADF)?;
        Ok(file.inode_id())
    }

    fn add_fd(&self, inode: InodeID) -> AlienResult<usize> {
        let task = current_task().unwrap();
        let file = Arc::new(ShimFile::new(inode));
        let fd = task.add_file(file);
        Ok(fd)
    }

    fn remove_fd(&self, fd: usize) -> AlienResult<InodeID> {
        let task = current_task().unwrap();
        let file = task.remove_file(fd).ok_or(AlienError::EBADF)?;
        Ok(file.inode_id())
    }

    fn fs_info(&self) -> AlienResult<(InodeID, InodeID)> {
        let task = current_task().unwrap();
        let fs_info = task.inner().fs_info.clone();
        Ok((fs_info.root.inode_id(), fs_info.cwd.inode_id()))
    }

    fn set_cwd(&self, inode: InodeID) -> AlienResult<()> {
        let task = current_task().unwrap();
        task.inner().fs_info.cwd = Arc::new(ShimFile::new(inode));
        Ok(())
    }
    fn copy_to_user(&self, dst: usize, buf: &[u8]) -> AlienResult<()> {
        let task = current_task().unwrap();
        task.write_bytes_to_user(VirtAddr::from(dst), buf)
    }

    fn copy_from_user(&self, src: usize, buf: &mut [u8]) -> AlienResult<()> {
        let task = current_task().unwrap();
        task.read_bytes_from_user(VirtAddr::from(src), buf)
    }

    fn read_string_from_user(
        &self,
        src: usize,
        mut buf: DVec<u8>,
    ) -> AlienResult<(DVec<u8>, usize)> {
        let task = current_task().unwrap();
        let str = task.read_string_from_user(VirtAddr::from(src))?;
        let len = str.as_bytes().len();
        let min_len = core::cmp::min(len, buf.len());
        buf.as_mut_slice()[..min_len].copy_from_slice(&str.as_bytes()[..min_len]);
        Ok((buf, min_len))
    }

    fn current_pid(&self) -> AlienResult<usize> {
        let task = current_task().unwrap();
        Ok(task.pid.raw())
    }
    fn current_ppid(&self) -> AlienResult<usize> {
        let task = current_task().unwrap();
        let p = task.inner().parent.clone();
        if p.is_none() {
            Ok(0)
        } else {
            let p = p.unwrap().upgrade().unwrap();
            Ok(p.pid() as _)
        }
    }
    fn do_brk(&self, addr: usize) -> AlienResult<isize> {
        let task = current_task().unwrap();
        let new_addr = task.extend_heap(addr);
        Ok(new_addr as isize)
    }

    fn do_clone(
        &self,
        flags: usize,
        stack: usize,
        ptid: usize,
        tls: usize,
        ctid: usize,
    ) -> AlienResult<isize> {
        syscall::clone::do_clone(flags, stack, ptid, tls, ctid)
    }

    fn do_wait4(
        &self,
        pid: isize,
        exit_code_ptr: usize,
        options: u32,
        _rusage: usize,
    ) -> AlienResult<isize> {
        syscall::wait::do_wait4(pid, exit_code_ptr, options, _rusage)
    }

    fn do_execve(
        &self,
        filename_ptr: usize,
        argv_ptr: usize,
        envp_ptr: usize,
    ) -> AlienResult<isize> {
        syscall::execve::do_execve(
            VirtAddr::from(filename_ptr),
            argv_ptr.into(),
            envp_ptr.into(),
        )
    }

    fn do_set_tid_address(&self, tidptr: usize) -> AlienResult<isize> {
        let task = current_task().unwrap();
        task.set_tid_address(tidptr);
        Ok(task.tid() as _)
    }
    fn do_mmap(
        &self,
        start: usize,
        len: usize,
        prot: u32,
        flags: u32,
        fd: usize,
        offset: usize,
    ) -> AlienResult<isize> {
        syscall::mmap::do_mmap(start, len, prot, flags, fd, offset)
    }

    fn do_munmap(&self, start: usize, len: usize) -> AlienResult<isize> {
        syscall::mmap::do_munmap(start, len)
    }

    fn do_sigaction(&self, signum: u8, act: usize, oldact: usize) -> AlienResult<isize> {
        syscall::signal::do_sigaction(signum, act, oldact)
    }
    fn do_sigprocmask(&self, how: usize, set: usize, oldset: usize) -> AlienResult<isize> {
        syscall::signal::do_sigprocmask(how, set, oldset)
    }
    fn do_fcntl(&self, fd: usize, cmd: usize) -> AlienResult<(InodeID, usize)> {
        syscall::fs::do_fcntl(fd, cmd)
    }
    fn do_prlimit(
        &self,
        pid: usize,
        resource: usize,
        new_limit: usize,
        old_limit: usize,
    ) -> AlienResult<isize> {
        syscall::prlimit::do_prlimit(pid, resource, new_limit, old_limit)
    }
    fn do_dup(&self, old_fd: usize, new_fd: Option<usize>) -> AlienResult<isize> {
        syscall::fs::do_dup(old_fd, new_fd)
    }

    fn do_pipe2(&self, r: InodeID, w: InodeID, pipe: usize) -> AlienResult<isize> {
        syscall::fs::do_pipe2(r, w, pipe)
    }

    fn do_exit(&self, exit_code: isize) -> AlienResult<isize> {
        syscall::exit::do_exit(exit_code as i32)
    }

    fn do_mmap_device(&self, phy_addr_range: Range<usize>) -> AlienResult<isize> {
        syscall::mmap::do_mmap_device(phy_addr_range)
    }
    fn do_set_priority(&self, which: i32, who: u32, priority: i32) -> AlienResult<()> {
        syscall::priority::do_set_priority(which, who, priority)
    }
    fn do_get_priority(&self, which: i32, who: u32) -> AlienResult<i32> {
        syscall::priority::do_get_priority(which, who)
    }
    fn do_signal_stack(&self, ss: usize, oss: usize) -> AlienResult<isize> {
        syscall::signal::do_signal_stack(ss, oss)
    }
    fn do_mprotect(&self, addr: usize, len: usize, prot: u32) -> AlienResult<isize> {
        syscall::mmap::do_mprotect(addr, len, prot)
    }
    fn do_load_page_fault(&self, addr: usize) -> AlienResult<()> {
        syscall::mmap::do_load_page_fault(addr)
    }
    fn do_futex(
        &self,
        uaddr: usize,
        futex_op: u32,
        val: u32,
        timeout: usize,
        uaddr2: usize,
        val3: u32,
    ) -> AlienResult<isize> {
        syscall::futex::futex(uaddr, futex_op, val, timeout, uaddr2, val3)
    }
}
define_unwind_for_TaskDomain!(TaskDomainImpl);
pub fn main() -> Box<dyn TaskDomain> {
    Box::new(UnwindWrap::new(TaskDomainImpl::new()))
}
