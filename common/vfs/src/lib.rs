#![no_std]
#![forbid(unsafe_code)]
#![feature(trait_upcasting)]
#![feature(btreemap_alloc)]
#![feature(allocator_api)]
extern crate alloc;
extern crate basic;

use alloc::{boxed::Box, collections::BTreeMap, sync::Arc};
use core::sync::atomic::{AtomicBool, AtomicU64};

use basic::{
    constants::{
        io::{Fcntl64Cmd, OpenFlags, PollEvents, SeekFrom},
        time::TimeSpec,
    },
    sync::{Mutex, RwLock},
    time::TimeNow,
    *,
};
use interface::{Basic, DomainType, InodeID, NetDomain, SocketID, VfsDomain};
use log::debug;
use rref::{RRef, RRefVec};
use spin::{Lazy, Once};
use storage::{DataStorageHeap, StorageBuilder};
use vfscore::{
    dentry::VfsDentry,
    path::VfsPath,
    utils::{VfsFileStat, VfsInodeMode, VfsNodeType, VfsPollEvents, VfsTime, VfsTimeSpec},
};

use crate::{
    kfile::{File, KernelFile},
    socket::SocketFile,
    tree::system_root_fs,
};

mod devfs;
mod initrd;
mod kfile;
mod pipe;
mod pipefs;
mod procfs;
mod ramfs;
mod shim;
mod socket;
mod sys;
mod tree;
static NET_STACK_DOMAIN: Once<Arc<dyn NetDomain>> = Once::new();
static VFS_MAP: RwLock<BTreeMap<InodeID, Arc<dyn File>>> = RwLock::new(BTreeMap::new());

static INODE_ID: Lazy<Arc<AtomicU64, DataStorageHeap>> =
    Lazy::new(|| storage::get_or_insert_with_data("inode_id", || AtomicU64::new(4)));

static VFS_INIT: Lazy<Arc<AtomicBool, DataStorageHeap>> =
    Lazy::new(|| storage::get_or_insert_with_data("vfs_init", || AtomicBool::new(false)));

type DataType = Arc<Mutex<BTreeMap<InodeID, (), DataStorageHeap>>, DataStorageHeap>;
static VFS_MAP_SHADOW: Lazy<DataType> = Lazy::new(|| {
    storage::get_or_insert_with_data("inode2inode", || {
        Mutex::new(BTreeMap::new_in(DataStorageHeap::build()))
    })
});

fn insert_dentry(dentry: Arc<dyn VfsDentry>, open_flags: OpenFlags) -> InodeID {
    let id = INODE_ID.fetch_add(1, core::sync::atomic::Ordering::SeqCst);
    let file = KernelFile::new(dentry, open_flags, id);
    VFS_MAP.write().insert(id, Arc::new(file));
    VFS_MAP_SHADOW.lock().insert(id, ());
    id
}

fn insert_special_file(file: Arc<dyn File>) -> InodeID {
    let id = INODE_ID.fetch_add(1, core::sync::atomic::Ordering::SeqCst);
    VFS_MAP.write().insert(id, file);
    id
}

fn remove_file(inode: InodeID) {
    if (0..4).contains(&inode) {
        debug!("<remove_file>, InodeID: {} is not correct", inode);
    } else {
        VFS_MAP.write().remove(&inode);
        VFS_MAP_SHADOW.lock().remove(&inode);
    }
}

fn get_file(inode: InodeID) -> Option<Arc<dyn File>> {
    VFS_MAP.read().get(&inode).cloned()
}

#[derive(Debug)]
struct VfsDomainImpl;

impl Basic for VfsDomainImpl {
    fn domain_id(&self) -> u64 {
        rref::domain_id()
    }
}

impl VfsDomain for VfsDomainImpl {
    fn init(&self, initrd: &[u8]) -> AlienResult<()> {
        let is_init_done = VFS_INIT.load(core::sync::atomic::Ordering::SeqCst);
        tree::init_filesystem(initrd, is_init_done).unwrap();
        let net_stack_domain = basic::get_domain("net_stack").unwrap();
        match net_stack_domain {
            DomainType::NetDomain(net_stack_domain) => {
                NET_STACK_DOMAIN.call_once(|| net_stack_domain);
            }
            _ => panic!("net_stack domain not found"),
        };
        VFS_INIT.store(true, core::sync::atomic::Ordering::SeqCst);
        println!("vfs init success");
        Ok(())
    }

    fn vfs_poll(&self, inode: InodeID, events: VfsPollEvents) -> AlienResult<VfsPollEvents> {
        let file = get_file(inode).unwrap();
        let res = file.poll(PollEvents::from_bits_truncate(events.bits()))?;
        Ok(VfsPollEvents::from_bits_truncate(res.bits()))
    }

    fn vfs_ioctl(&self, inode: InodeID, cmd: u32, arg: usize) -> AlienResult<usize> {
        let file = get_file(inode).unwrap();
        let res = file.ioctl(cmd, arg)?;
        Ok(res)
    }

    fn vfs_open(
        &self,
        root: InodeID,
        path: &RRefVec<u8>,
        path_len: usize,
        mode: u32,
        open_flags: usize,
    ) -> AlienResult<InodeID> {
        let start = get_file(root).ok_or(AlienError::EINVAL)?;
        let root = system_root_fs();
        let path_name = core::str::from_utf8(&path.as_slice()[..path_len]).unwrap();
        let open_flags = OpenFlags::from_bits_truncate(open_flags);
        let mode = if open_flags.contains(OpenFlags::O_CREAT) {
            Some(VfsInodeMode::from_bits_truncate(mode))
        } else {
            None
        };
        // println_color!(31,"vfs_open: path_name: {}, mode: {:?}", path_name, mode);
        let path = VfsPath::new(root, start.dentry())
            .join(path_name)?
            .open(mode)?;
        let id = insert_dentry(path, open_flags);
        // println_color!(31,"vfs_open: path_name: {} with id: {}", path_name,id);
        Ok(id)
    }
    fn vfs_close(&self, inode: InodeID) -> AlienResult<()> {
        remove_file(inode);
        Ok(())
    }

    fn vfs_getattr(
        &self,
        inode: InodeID,
        mut attr: RRef<VfsFileStat>,
    ) -> AlienResult<RRef<VfsFileStat>> {
        let vfs_attr = get_file(inode).unwrap().get_attr()?;
        *attr = vfs_attr;
        Ok(attr)
    }
    fn vfs_read_at(
        &self,
        inode: InodeID,
        offset: u64,
        mut buf: RRefVec<u8>,
    ) -> AlienResult<(RRefVec<u8>, usize)> {
        let file = get_file(inode).unwrap();
        let res = file.read_at(offset, buf.as_mut_slice())?;
        Ok((buf, res))
    }
    fn vfs_read(&self, inode: InodeID, mut buf: RRefVec<u8>) -> AlienResult<(RRefVec<u8>, usize)> {
        let file = get_file(inode).unwrap();
        let res = file.read(buf.as_mut_slice())?;
        Ok((buf, res))
    }
    fn vfs_write_at(
        &self,
        inode: InodeID,
        offset: u64,
        buf: RRefVec<u8>,
    ) -> AlienResult<(RRefVec<u8>, usize)> {
        let file = get_file(inode).unwrap();
        let res = file.write_at(offset, buf.as_slice())?;
        Ok((buf, res))
    }
    fn vfs_write(&self, inode: InodeID, buf: &RRefVec<u8>, w_len: usize) -> AlienResult<usize> {
        let file = get_file(inode).unwrap();
        let res = file.write(&buf.as_slice()[..w_len])?;
        Ok(res)
    }
    fn vfs_flush(&self, inode: InodeID) -> AlienResult<()> {
        let file = get_file(inode).unwrap();
        file.flush()?;
        Ok(())
    }
    fn vfs_fsync(&self, inode: InodeID) -> AlienResult<()> {
        let file = get_file(inode).unwrap();
        file.fsync()?;
        Ok(())
    }
    fn vfs_lseek(&self, inode: InodeID, seek: SeekFrom) -> AlienResult<u64> {
        let file = get_file(inode).unwrap();
        file.seek(seek)
    }
    fn vfs_inode_type(&self, inode: InodeID) -> AlienResult<VfsNodeType> {
        let file = get_file(inode).unwrap();
        let res = file.inode().inode_type();
        Ok(res)
    }
    fn vfs_readdir(
        &self,
        inode: InodeID,
        mut buf: RRefVec<u8>,
    ) -> AlienResult<(RRefVec<u8>, usize)> {
        let file = get_file(inode).unwrap();
        let res = file.readdir(buf.as_mut_slice())?;
        Ok((buf, res))
    }

    fn vfs_get_path(
        &self,
        inode: InodeID,
        mut buf: RRefVec<u8>,
    ) -> AlienResult<(RRefVec<u8>, usize)> {
        let file = get_file(inode).unwrap();
        let path = file.dentry().path();
        let path = path.as_bytes();
        let copy_len = core::cmp::min(path.len(), buf.len());
        buf.as_mut_slice()[..copy_len].copy_from_slice(&path[..copy_len]);
        Ok((buf, copy_len))
    }
    fn vfs_ftruncate(&self, inode: InodeID, len: u64) -> AlienResult<()> {
        let file = get_file(inode).unwrap();
        file.truncate(len)?;
        Ok(())
    }

    fn vfs_update_atime(&self, inode: InodeID, atime_sec: u64, atime_nano: u64) -> AlienResult<()> {
        let file = get_file(inode).unwrap();
        let time = VfsTimeSpec::new(atime_sec, atime_nano);
        let now = TimeSpec::now();
        let now = VfsTimeSpec::new(now.tv_sec as u64, now.tv_nsec as u64);
        file.dentry()
            .inode()?
            .update_time(VfsTime::AccessTime(time), now)?;
        Ok(())
    }

    fn vfs_update_mtime(&self, inode: InodeID, mtime_sec: u64, mtime_nano: u64) -> AlienResult<()> {
        let file = get_file(inode).unwrap();
        let time = VfsTimeSpec::new(mtime_sec, mtime_nano);
        let now = TimeSpec::now();
        let now = VfsTimeSpec::new(now.tv_sec as u64, now.tv_nsec as u64);
        file.dentry()
            .inode()?
            .update_time(VfsTime::ModifiedTime(time), now)?;
        Ok(())
    }

    fn do_fcntl(&self, inode: InodeID, cmd: usize, args: usize) -> AlienResult<isize> {
        const FD_CLOEXEC: usize = 1;
        let cmd = Fcntl64Cmd::try_from(cmd as u32).unwrap();
        let file = get_file(inode).unwrap();
        match cmd {
            Fcntl64Cmd::F_DUPFD_CLOEXEC => {
                file.set_open_flag(file.get_open_flag() | OpenFlags::O_CLOEXEC);
                Ok(0)
            }
            Fcntl64Cmd::F_GETFD => {
                if file.get_open_flag().contains(OpenFlags::O_CLOEXEC) {
                    Ok(1)
                } else {
                    Ok(0)
                }
            }
            Fcntl64Cmd::F_SETFD => {
                debug!("fcntl: F_SETFD :{:?}", args & FD_CLOEXEC);
                let open_flag = file.get_open_flag();
                if args & FD_CLOEXEC == 0 {
                    file.set_open_flag(open_flag & !OpenFlags::O_CLOEXEC);
                } else {
                    file.set_open_flag(open_flag | OpenFlags::O_CLOEXEC);
                }
                Ok(0)
            }
            Fcntl64Cmd::F_GETFL => Ok(file.get_open_flag().bits() as isize),
            Fcntl64Cmd::F_SETFL => {
                let flag = OpenFlags::from_bits_truncate(args);
                debug!("fcntl: F_SETFL :{:?}", flag,);
                file.set_open_flag(flag);
                Ok(0)
            }
            _ => Err(AlienError::EINVAL),
        }
    }
    fn do_pipe2(&self, _flags: usize) -> AlienResult<(InodeID, InodeID)> {
        let (reader, writer) = pipe::make_pipe_file();
        let r = insert_special_file(reader);
        let w = insert_special_file(writer);
        Ok((r, w))
    }
    fn do_socket(&self, socket_id: SocketID) -> AlienResult<InodeID> {
        let socket_file = SocketFile::new(
            NET_STACK_DOMAIN.get().unwrap().clone(),
            socket_id,
            OpenFlags::O_RDWR,
        );
        let inode_id = insert_special_file(Arc::new(socket_file));
        Ok(inode_id)
    }
    fn socket_id(&self, inode: InodeID) -> AlienResult<SocketID> {
        let file = get_file(inode).unwrap();
        let socket_file = file.downcast_arc::<SocketFile>().unwrap();
        Ok(socket_file.socket_id())
    }
}

pub fn main() -> Box<dyn VfsDomain> {
    Box::new(VfsDomainImpl)
}
