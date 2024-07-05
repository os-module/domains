use alloc::{
    format,
    string::{String, ToString},
    sync::Arc,
};
use core::cmp::min;

use interface::DomainTypeRaw;
use vfscore::{
    error::VfsError,
    file::VfsFile,
    inode::VfsInode,
    utils::{VfsDirEntry, VfsFileStat, VfsInodeMode, VfsNodePerm, VfsNodeType, VfsTimeSpec},
    VfsResult,
};

use crate::{custom_inode::CustomRootInode, DOMAIN_INFO};

pub fn domain_fs_root() -> Arc<dyn VfsInode> {
    let root = CustomRootInode::new();
    root.insert_inode("domain-type".to_string(), Arc::new(DomainTyInfoDir));
    root.insert_inode("domains".to_string(), Arc::new(DomainFileInfoDir));
    Arc::new(root)
}

pub struct DomainTyInfoDir;

fn __readdir_from_ty(start_index: usize) -> VfsResult<Option<VfsDirEntry>> {
    let domain_info = DOMAIN_INFO.get();
    if let Some(domain_info) = domain_info {
        let guard = domain_info.lock();
        let tys = &guard.ty_list;
        let entry = tys.iter().skip(start_index).next().map(|(ty, _)| {
            let name = format!("{}", ty);
            VfsDirEntry {
                ino: 0,
                ty: VfsNodeType::File,
                name,
            }
        });
        Ok(entry)
    } else {
        Ok(None)
    }
}
fn __readdir_from_domain_list(start_index: usize) -> VfsResult<Option<VfsDirEntry>> {
    let domain_info = DOMAIN_INFO.get();
    if let Some(domain_info) = domain_info {
        let guard = domain_info.lock();
        let domains = &guard.domain_list;
        let entry = domains
            .iter()
            .skip(start_index)
            .next()
            .map(|(name, _)| VfsDirEntry {
                ino: 0,
                ty: VfsNodeType::File,
                name: name.to_string(),
            });
        Ok(entry)
    } else {
        Ok(None)
    }
}

impl VfsFile for DomainTyInfoDir {
    fn readdir(&self, start_index: usize) -> VfsResult<Option<VfsDirEntry>> {
        return __readdir_from_ty(start_index);
    }
}

impl VfsInode for DomainTyInfoDir {
    fn lookup(&self, name: &str) -> VfsResult<Arc<dyn VfsInode>> {
        let domain_info = DOMAIN_INFO.get();
        if let Some(domain_info) = domain_info {
            let guard = domain_info.lock();
            let tys = &guard.ty_list;
            let ty = domain_type_from_str(name).ok_or(VfsError::NoEntry)?;
            let infos = tys.get(&ty).ok_or(VfsError::NoEntry)?;
            let data = format!("type: {}\nfile: {:#?}", ty, infos);
            return Ok(Arc::new(DomainInfoFile::new(data)));
        }
        Err(VfsError::NoEntry)
    }
    fn get_attr(&self) -> VfsResult<VfsFileStat> {
        let mode = VfsInodeMode::from(VfsNodePerm::from_bits_truncate(0o644), VfsNodeType::Dir);
        Ok(VfsFileStat {
            st_dev: 0,
            st_ino: 0,
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
}

pub struct DomainFileInfoDir;

impl VfsFile for DomainFileInfoDir {
    fn readdir(&self, start_index: usize) -> VfsResult<Option<VfsDirEntry>> {
        __readdir_from_domain_list(start_index)
    }
}

impl VfsInode for DomainFileInfoDir {
    fn lookup(&self, name: &str) -> VfsResult<Arc<dyn VfsInode>> {
        let domain_info = DOMAIN_INFO.get();
        if let Some(domain_info) = domain_info {
            let guard = domain_info.lock();
            let domain_list = &guard.domain_list;
            let info = domain_list.get(name).ok_or(VfsError::NoEntry)?;
            let data = format!("DomainName: {}\nInformation: {:#?}", name, info);
            return Ok(Arc::new(DomainInfoFile::new(data)));
        }
        Err(VfsError::NoEntry)
    }
    fn get_attr(&self) -> VfsResult<VfsFileStat> {
        let mode = VfsInodeMode::from(VfsNodePerm::from_bits_truncate(0o644), VfsNodeType::Dir);
        Ok(VfsFileStat {
            st_dev: 0,
            st_ino: 0,
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
}

fn domain_type_from_str(name: &str) -> Option<DomainTypeRaw> {
    match name {
        "FsDomain" => Some(DomainTypeRaw::FsDomain),
        "BlkDeviceDomain" => Some(DomainTypeRaw::BlkDeviceDomain),
        "CacheBlkDeviceDomain" => Some(DomainTypeRaw::CacheBlkDeviceDomain),
        "RtcDomain" => Some(DomainTypeRaw::RtcDomain),
        "GpuDomain" => Some(DomainTypeRaw::GpuDomain),
        "InputDomain" => Some(DomainTypeRaw::InputDomain),
        "VfsDomain" => Some(DomainTypeRaw::VfsDomain),
        "UartDomain" => Some(DomainTypeRaw::UartDomain),
        "PLICDomain" => Some(DomainTypeRaw::PLICDomain),
        "TaskDomain" => Some(DomainTypeRaw::TaskDomain),
        "SysCallDomain" => Some(DomainTypeRaw::SysCallDomain),
        "ShadowBlockDomain" => Some(DomainTypeRaw::ShadowBlockDomain),
        "BufUartDomain" => Some(DomainTypeRaw::BufUartDomain),
        "NetDeviceDomain" => Some(DomainTypeRaw::NetDeviceDomain),
        "BufInputDomain" => Some(DomainTypeRaw::BufInputDomain),
        "EmptyDeviceDomain" => Some(DomainTypeRaw::EmptyDeviceDomain),
        "DevFsDomain" => Some(DomainTypeRaw::DevFsDomain),
        "SchedulerDomain" => Some(DomainTypeRaw::SchedulerDomain),
        "LogDomain" => Some(DomainTypeRaw::LogDomain),
        "NetDomain" => Some(DomainTypeRaw::NetDomain),
        _ => None,
    }
}

pub struct DomainInfoFile {
    data: String,
}

impl DomainInfoFile {
    pub fn new(data: String) -> Self {
        Self { data }
    }
}

impl VfsFile for DomainInfoFile {
    fn read_at(&self, offset: u64, buf: &mut [u8]) -> VfsResult<usize> {
        if offset as usize >= self.data.len() {
            return Ok(0);
        }
        let copy_start = min(offset as usize, self.data.len());
        let copy_end = min(copy_start + buf.len(), self.data.len());
        let copied = copy_end - copy_start;
        buf[..copied].copy_from_slice(&self.data.as_bytes()[copy_start..copy_end]);
        Ok(copied)
    }
}

impl VfsInode for DomainInfoFile {
    fn get_attr(&self) -> VfsResult<VfsFileStat> {
        let mode = VfsInodeMode::from(VfsNodePerm::from_bits_truncate(0o644), VfsNodeType::File);
        Ok(VfsFileStat {
            st_dev: 0,
            st_ino: 0,
            st_mode: mode.bits(),
            st_nlink: 1,
            st_uid: 0,
            st_gid: 0,
            st_rdev: 0,
            __pad: 0,
            st_size: self.data.len() as u64,
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
        VfsNodeType::File
    }
}
