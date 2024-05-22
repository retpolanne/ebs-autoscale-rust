use std::fmt;
use std::io;
use std::path::Path;
use std::error::Error;
use log::info;
use serde::{Deserialize, Serialize};
use sysinfo::Disks;

#[derive(Debug, Deserialize, Serialize)]
pub struct Limits {
    initial_utilization_threshold: u32,
    min_ebs_volume_size: u32,
    max_ebs_volume_size: u32,
    max_logical_volume_size: u32,
    max_ebs_volume_count: u32
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    /// Ensure that EBS volumes are deleted on termination
    ///
    /// By default, this is true. If you prefer to keep it safe, turn this config to false
    pub ensure_ebs_deleted_on_term: bool,
    /// Detection interval, in seconds
    ///
    /// Default: 2 seconds
    pub detection_interval: u8,

    pub mountpoint: String,

    pub limits: Limits,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            ensure_ebs_deleted_on_term: true,
            detection_interval: 2,
            mountpoint: "/dev/xvdba".to_string(),
            limits: Limits {
                initial_utilization_threshold: 80,
                min_ebs_volume_size: 10,
                max_ebs_volume_size: 1000,
                max_logical_volume_size: 1000,
                max_ebs_volume_count: 100
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct MountPointNotFoundError;

impl Error for MountPointNotFoundError {

}

impl fmt::Display for MountPointNotFoundError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Could not find mount point")
    }
}

#[derive(Debug, Clone)]
pub struct MaxEBSCountExceededError;

impl Error for MaxEBSCountExceededError {

}

impl fmt::Display for MaxEBSCountExceededError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Maximum number of EBS volumes exceeded")
    }
}

pub trait DiskMgr {
    fn new_disks(&mut self);
    fn save_disk_list(&mut self);
    /// Returns the usage percentage for a mountpoint
    fn disk_usage_percent(&mut self, mountpoint: String) -> Result<u32, MountPointNotFoundError>;
    /// Total size for a mountpoint
    fn disk_size(&mut self, mountpoint: String) -> Result<u64, MountPointNotFoundError>;
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
}

pub trait AWS {
    fn request_ebs_volume(&mut self);
    fn get_attached_ebs_volumes(&mut self);
    fn delete_ebs_volume(&mut self);
}

struct ConcreteAWS {}

impl AWS for ConcreteAWS {
    fn request_ebs_volume(&mut self) {

    }
    fn get_attached_ebs_volumes(&mut self) {

    }
    fn delete_ebs_volume(&mut self) {

    }
}

pub struct EBSManager {
    config: Config,
    diskmgr: Box<dyn DiskMgr>,
    aws: Box<dyn AWS>,
}

impl EBSManager {
    pub fn new(
        conf: Config,
        disks: Box<dyn DiskMgr>,
        aws_cli: Box<dyn AWS>
    ) -> Box<EBSManager> {
        Box::new(Self {
            config: conf,
            diskmgr: disks,
            aws: aws_cli
        })
    }

    pub fn power_on_self_test(&self) -> Result<bool, io::Error> {
        Ok(true)
    }

    pub fn need_more_space(&mut self) -> Result<bool, Box<dyn Error>> {
        let dev_count = self.count_mounted_ebs_devices();
        let threshold = self.calc_threshold(dev_count.unwrap()).unwrap();
        let disk_utilization = self.diskmgr.disk_usage_percent(
            self.config.mountpoint.clone()
        )?;

        if disk_utilization >= threshold {
            info!("Low disk space - adding more disks");
            return Ok(true);
        }
        Ok(false)

    }

    pub fn add_more_space(&mut self, dev_count: u32) -> Result<bool, Box<dyn Error>> {
        if dev_count >= self.config.limits.max_ebs_volume_count {
            return Err(Box::new(MaxEBSCountExceededError));
        }
        let new_size = self.calc_new_size(dev_count);
        let cur_size = self.diskmgr.disk_size(self.config.mountpoint.clone());
        if
        Ok(true)
    }

    fn count_mounted_ebs_devices(&self) -> Option<u32> {
        Some(10)
    }

    fn calc_threshold(&self, dev_count: u32) -> Option<u32> {
        if dev_count >= 4 && dev_count <= 6 {
            return Some(80);
        }
        if dev_count > 6 && dev_count <= 10 {
            return Some(90)
        }
        if dev_count > 10 {
            return Some(90)
        }
        Some(self.config.limits.initial_utilization_threshold)
    }

    fn calc_new_size(&self, dev_count: u32) -> Option<u32> {
        Some(10)
    }

    fn get_next_logical_device(&self) -> Option<String> {
        Some("/dev/xvdb".to_string())
    }

    fn request_new_ebs_volume(&self, size: u32) -> Result<bool, io::Error> {
        Ok(true)
    }

    fn extend_logical_device(&self) -> Result<bool, io::Error> {
        Ok(true)
    }

}


#[cfg(test)]
mod tests {
    use super::*;
    use figment::{Figment, providers::{Format, Toml, Serialized}};

    struct Context {
        ebs_manager: Box<EBSManager>,
    }

    struct MockDiskMgr {
        disks: Vec<String>,
        utilization_percentage: u32,
        total_disk_size: u64,
    }

    impl Default for MockDiskMgr {
        fn default() -> MockDiskMgr {
            MockDiskMgr {
                disks: vec!["test".to_string()],
                utilization_percentage: 10,
                total_disk_size: 100,
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

        fn disk_usage_percent(&mut self, mountpoint: String) -> Result<u32, MountPointNotFoundError> {
            Ok(self.utilization_percentage)
        }

        fn disk_size(&mut self, mountpoint: String) -> Result<u64, MountPointNotFoundError> {
            Ok(self.total_disk_size)
        }
    }

    struct MockAWS {}

    impl AWS for MockAWS {
        fn request_ebs_volume(&mut self) {

        }
        fn get_attached_ebs_volumes(&mut self) {

        }
        fn delete_ebs_volume(&mut self) {

        }
    }

    fn setup(mock_diskmgr: MockDiskMgr) -> Result<Context, Box<dyn Error>> {
        let config : Config = Figment::from(Serialized::defaults(Config::default()))
            .merge(Toml::file("Test.toml"))
            .extract()?;
        let mock_diskmgr : Box<dyn DiskMgr> = Box::new(mock_diskmgr);
        let mock_aws = Box::new(MockAWS {});
        Ok(Context {
            ebs_manager: EBSManager::new(
                config,
                mock_diskmgr,
                mock_aws
            ),
        })
    }

    #[test]
    fn test_power_on_self_test() -> Result<(), io::Error> {
        let ctx = setup(MockDiskMgr::default()).unwrap();
        assert_eq!(ctx.ebs_manager.power_on_self_test()?, true);
        Ok(())
    }

    #[test]
    fn test_need_more_space_false() -> Result<(), Box<dyn Error>> {
        let mut ctx = setup(MockDiskMgr::default()).unwrap();
        assert_eq!(ctx.ebs_manager.need_more_space()?, false);
        Ok(())
    }

    #[test]
    fn test_need_more_space_true() -> Result<(), Box<dyn Error>> {
        let mut ctx = setup(MockDiskMgr {
            disks: vec!["test".to_string()],
            utilization_percentage: 95,
            total_disk_size: 100,
        }).unwrap();
        assert_eq!(ctx.ebs_manager.need_more_space()?, true);
        Ok(())
    }

    #[test]
    fn test_add_more_space_max_ebs_count() -> Result<(), Box<dyn Error>> {
        let mut ctx = setup(MockDiskMgr::default()).unwrap();
        assert!(ctx.ebs_manager.add_more_space(101).is_err());
        Ok(())
    }

    #[test]
    fn test_add_more_space_max_logical_size() -> Result<(), Box<dyn Error>> {
        /// TODO
        let mut ctx = setup(MockDiskMgr::default()).unwrap();
        assert!(ctx.ebs_manager.add_more_space(101).is_err());
        Ok(())
    }

    #[test]
    fn test_request_new_ebs_volume() -> Result<(), io::Error> {
        let ctx = setup(MockDiskMgr::default()).unwrap();
        assert_eq!(ctx.ebs_manager.request_new_ebs_volume(32)?, true);
        Ok(())
    }

    #[test]
    fn test_count_mounted_ebs_devices() {
        let ctx = setup(MockDiskMgr::default()).unwrap();
        assert_eq!(ctx.ebs_manager.count_mounted_ebs_devices(), Some(10));
    }

    #[test]
    fn test_calc_threshold() {
        let ctx = setup(MockDiskMgr::default()).unwrap();
        assert_eq!(ctx.ebs_manager.calc_threshold(1),
                   Some(ctx.ebs_manager.config.limits.initial_utilization_threshold)
        );
        for dev_count in [4, 5, 6] {
            assert_eq!(ctx.ebs_manager.calc_threshold(dev_count), Some(80));
        }
        for dev_count in [7, 8, 9, 10] {
            assert_eq!(ctx.ebs_manager.calc_threshold(dev_count), Some(90));
        }
        assert_eq!(ctx.ebs_manager.calc_threshold(11), Some(90));
    }

    #[test]
    fn test_calc_new_size() {
        let ctx = setup(MockDiskMgr::default()).unwrap();
        assert_eq!(ctx.ebs_manager.calc_new_size(10), Some(10));
    }

    #[test]
    fn test_get_next_logical_device() {
        let ctx = setup(MockDiskMgr::default()).unwrap();
        assert_eq!(ctx.ebs_manager.get_next_logical_device(), Some("/dev/xvdb".to_string()));
    }

    #[test]
    fn test_extend_logical_device() -> Result<(), io::Error> {
        let ctx = setup(MockDiskMgr::default()).unwrap();
        assert_eq!(ctx.ebs_manager.extend_logical_device()?, true);
        Ok(())
    }
}
