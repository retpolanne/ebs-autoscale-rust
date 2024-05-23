use std::path::Path;
use sysinfo::Disks;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct MountPointNotFoundError;

impl Error for MountPointNotFoundError {}

impl fmt::Display for MountPointNotFoundError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Could not find mount point")
    }
}

#[derive(Debug, Clone)]
pub struct NoMoreDeviceNamesAvailableError;

impl Error for NoMoreDeviceNamesAvailableError {}

impl fmt::Display for NoMoreDeviceNamesAvailableError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "No more device names (i.e. /dev/xvdb*) available!")
    }
}

pub trait DiskMgr {
    fn new_disks(&mut self);
    fn save_disk_list(&mut self);
    /// Returns the usage percentage for a mountpoint
    fn disk_usage_percent(&mut self, mountpoint: String) -> Result<u32, MountPointNotFoundError>;
    /// Total size for a mountpoint
    fn disk_size(&mut self, mountpoint: String) -> Result<u64, MountPointNotFoundError>;
    fn get_next_logical_device(&mut self) -> Result<String, NoMoreDeviceNamesAvailableError>;
}

struct ConcreteDiskMgr {
    disks: Disks,
}

impl DiskMgr for ConcreteDiskMgr {
    fn new_disks(&mut self) {
        self.disks = Disks::new()
    }

    fn save_disk_list(&mut self) {
        self.disks.refresh_list();
    }

    fn disk_usage_percent(&mut self, mountpoint: String) -> Result<u32, MountPointNotFoundError> {
        let mt_path = Path::new(&mountpoint);
        for disk in self.disks.list() {
            if disk.mount_point() == mt_path {
                return Ok(
                    (disk.total_space() / disk.available_space() * 100)
                        .try_into().unwrap()
                )
            }
        };
        Err(MountPointNotFoundError)
    }

    fn disk_size(&mut self, mountpoint: String) -> Result<u64, MountPointNotFoundError> {
        let mt_path = Path::new(&mountpoint);
        for disk in self.disks.list() {
            if disk.mount_point() == mt_path {
                return Ok(disk.total_space())
            }
        };
        Err(MountPointNotFoundError)
    }

    fn get_next_logical_device(&mut self) -> Result<String, NoMoreDeviceNamesAvailableError> {
        Ok("/dev/test".to_string())
    }
}

pub struct MockDiskMgr {
    pub disks: Vec<String>,
    pub utilization_percentage: u32,
    pub total_disk_size: u64,
    pub sim_no_more_device_names: bool
}

impl Default for MockDiskMgr {
    fn default() -> MockDiskMgr {
        MockDiskMgr {
            disks: vec!["test".to_string()],
            utilization_percentage: 10,
            total_disk_size: 100,
            sim_no_more_device_names: false,
        }
    }
}

impl DiskMgr for MockDiskMgr {
    fn new_disks(&mut self) {
        self.disks = Vec::new()
    }

    fn save_disk_list(&mut self) {
        self.disks = vec!["test".to_string()];
    }

    fn disk_usage_percent(&mut self, _mountpoint: String) -> Result<u32, MountPointNotFoundError> {
        Ok(self.utilization_percentage)
    }

    fn disk_size(&mut self, _mountpoint: String) -> Result<u64, MountPointNotFoundError> {
        Ok(self.total_disk_size)
    }

    fn get_next_logical_device(&mut self) -> Result<String, NoMoreDeviceNamesAvailableError> {
        if self.sim_no_more_device_names {
            return Err(NoMoreDeviceNamesAvailableError)
        }
        Ok("/dev/test".to_string())
    }
}
