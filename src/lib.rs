use std::fmt;
use std::io;
use std::path::Path;
use std::error::Error;
use log::info;
use serde::{Deserialize, Serialize};
use sysinfo::{Disks};

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

    pub mountpoint : String
}

impl Default for Config {
    fn default() -> Self {
        Config {
            ensure_ebs_deleted_on_term: true,
            detection_interval: 2,
            mountpoint: "/dev/xvdba".to_string()
        }
    }
}

#[derive(Debug, Clone)]
struct MountPointNotFoundError;

impl Error for MountPointNotFoundError {

}

impl fmt::Display for MountPointNotFoundError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Could not find mount point")
    }
}

pub trait DiskMgr {
    fn new_disks(&mut self);
    fn save_disk_list(&mut self);
    /// Returns the usage percentage for a mountpoint
    fn disk_usage_percent(&mut self, mountpoint: String) -> Result<u32, MountPointNotFoundError>;
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
            self.add_more_space(dev_count.unwrap())?;
        }
        Ok(true)

    }

    pub fn add_more_space(&self, dev_count: u32) -> Result<bool, io::Error> {
        Ok(true)
    }

    fn count_mounted_ebs_devices(&self) -> Option<u32> {
        Some(10)
    }

    fn calc_threshold(&self, dev_count: u32) -> Option<u32> {
        Some(dev_count)
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
        disks: Vec<String>
    }

    impl DiskMgr for MockDiskMgr {
        fn new_disks(&mut self) {
            self.disks = Vec::new()
        }

        fn save_disk_list(&mut self) {
            self.disks = vec!["test".to_string()];
        }

        fn disk_usage_percent(&mut self, mountpoint: String) -> Result<u32, MountPointNotFoundError> {
            Ok(85)
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

    fn setup() -> Result<Context, Box<dyn Error>> {
        let config : Config = Figment::from(Serialized::defaults(Config::default()))
            .merge(Toml::file("Test.toml"))
            .extract()?;
        let mock_diskmgr : Box<dyn DiskMgr> = Box::new(MockDiskMgr {disks: vec!["test".to_string()]});
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
        let ctx = setup();
        assert_eq!(ctx.unwrap().ebs_manager.power_on_self_test()?, true);
        Ok(())
    }

    #[test]
    fn test_need_more_space_false() -> Result<(), Box<dyn Error>> {
        let ctx = setup();
        assert_eq!(ctx.unwrap().ebs_manager.need_more_space()?, true);
        Ok(())
    }

    #[test]
    fn test_request_new_ebs_volume() -> Result<(), io::Error> {
        let ctx = setup();
        assert_eq!(ctx.unwrap().ebs_manager.request_new_ebs_volume(32)?, true);
        Ok(())
    }

    #[test]
    fn test_count_mounted_ebs_devices() {
        let ctx = setup();
        assert_eq!(ctx.unwrap().ebs_manager.count_mounted_ebs_devices(), Some(10));
    }

    #[test]
    fn test_calc_threshold() {
        let ctx = setup();
        assert_eq!(ctx.unwrap().ebs_manager.calc_threshold(10), Some(10));
    }

    #[test]
    fn test_calc_new_size() {
        let ctx = setup();
        assert_eq!(ctx.unwrap().ebs_manager.calc_new_size(10), Some(10));
    }

    #[test]
    fn test_get_next_logical_device() {
        let ctx = setup();
        assert_eq!(ctx.unwrap().ebs_manager.get_next_logical_device(), Some("/dev/xvdb".to_string()));
    }

    #[test]
    fn test_extend_logical_device() -> Result<(), io::Error> {
        let ctx = setup();
        assert_eq!(ctx.unwrap().ebs_manager.extend_logical_device()?, true);
        Ok(())
    }
}
