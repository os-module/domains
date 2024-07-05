use alloc::{
    string::{String, ToString},
    sync::Arc,
    vec::Vec,
};

use basic::sync::Mutex;
use vfscore::{
    error::VfsError,
    file::VfsFile,
    inode::VfsInode,
    utils::{
        VfsDirEntry, VfsFileStat, VfsInodeMode, VfsNodePerm, VfsNodeType, VfsTime, VfsTimeSpec,
    },
    VfsResult,
};

#[derive(Default)]
pub struct CustomRootInode {
    children: Mutex<Vec<(String, Arc<dyn VfsInode>)>>,
}

impl CustomRootInode {
    pub fn new() -> Self {
        Self {
            children: Mutex::new(Vec::new()),
        }
    }
    pub fn insert_inode(&self, name: String, inode: Arc<dyn VfsInode>) {
        if self.children.lock().iter().find(|x| x.0 == name).is_none() {
            self.children.lock().push((name.to_string(), inode));
        }
    }
}

impl VfsFile for CustomRootInode {
    fn readdir(&self, start_index: usize) -> VfsResult<Option<VfsDirEntry>> {
        let children = self.children.lock();
        if start_index >= children.len() {
            return Ok(None);
        }
        let (name, inode) = &children[start_index];
        Ok(Some(VfsDirEntry {
            ino: 0,
            ty: inode.inode_type(),
            name: name.clone(),
        }))
    }
}

impl VfsInode for CustomRootInode {
    fn node_perm(&self) -> VfsNodePerm {
        VfsNodePerm::from_bits_truncate(0o644)
    }
    fn lookup(&self, name: &str) -> VfsResult<Arc<dyn VfsInode>> {
        let res = self
            .children
            .lock()
            .iter()
            .find(|(f_name, _)| f_name == name)
            .map(|(_, inode)| inode.clone());
        match res {
            Some(inode) => Ok(inode),
            None => Err(VfsError::NoEntry),
        }
    }
    fn get_attr(&self) -> VfsResult<VfsFileStat> {
        let mode = VfsInodeMode::from(VfsNodePerm::from_bits_truncate(0o644), VfsNodeType::Dir);
        Ok(VfsFileStat {
            st_dev: 0,
            st_ino: 1,
            st_mode: mode.bits(),
            st_nlink: 1,
            st_uid: 0,
            st_gid: 0,
            st_rdev: 0,
            __pad: 0,
            st_size: 4096,
            st_blksize: 512,
            __pad2: 0,
            st_blocks: 0,
            st_atime: VfsTimeSpec::default(),
            st_mtime: VfsTimeSpec::default(),
            st_ctime: VfsTimeSpec::default(),
            unused: 0,
        })
    }

    fn inode_type(&self) -> VfsNodeType {
        VfsNodeType::Dir
    }

    fn update_time(&self, _time: VfsTime, _now: VfsTimeSpec) -> VfsResult<()> {
        Ok(())
    }
}
