use alloc::sync::Arc;

use basic::{constants::DeviceId, sync::Mutex};
use spin::Lazy;
use storage::DataStorageHeap;
use vfs_common::id::DeviceIdManager;
use vfscore::utils::VfsNodeType;

type DeviceIdManagerType = Arc<Mutex<DeviceIdManager>, DataStorageHeap>;
static DEVICE_ID_MANAGER: Lazy<DeviceIdManagerType> =
    Lazy::new(|| {
        let res = storage::get_or_insert_with_data("device_id_manager", || {
            Mutex::new(DeviceIdManager::new())
        });
        res
    });
pub fn alloc_device_id(inode_type: VfsNodeType) -> DeviceId {
    DEVICE_ID_MANAGER.lock().alloc(inode_type)
}
