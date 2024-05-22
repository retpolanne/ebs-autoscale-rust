pub mod errors;
pub mod config;
mod fs;
mod aws;
mod disk;

use std::io;
use std::path::Path;
use std::error::Error;
use log::info;
use sysinfo::Disks;

pub trait DiskMgr {
    fn new_disks(&mut self);
    fn save_disk_list(&mut self);
    /// Returns the usage percentage for a mountpoint
    fn disk_usage_percent(&mut self, mountpoint: String) -> Result<u32, errors::MountPointNotFoundError>;
    /// Total size for a mountpoint
    fn disk_size(&mut self, mountpoint: String) -> Result<u64, errors::MountPointNotFoundError>;
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

    fn disk_usage_percent(&mut self, mountpoint: String) -> Result<u32, errors::MountPointNotFoundError> {
        let mt_path = Path::new(&mountpoint);
        for disk in self.disks.list() {
            if disk.mount_point() == mt_path {
                return Ok(
                    (disk.total_space() / disk.available_space() * 100)
                        .try_into().unwrap()
                )
            }
        };
        Err(errors::MountPointNotFoundError)
    }

    fn disk_size(&mut self, mountpoint: String) -> Result<u64, errors::MountPointNotFoundError> {
        let mt_path = Path::new(&mountpoint);
        for disk in self.disks.list() {
            if disk.mount_point() == mt_path {
                return Ok(disk.total_space())
            }
        };
        Err(errors::MountPointNotFoundError)
    }
}

pub trait AWS {
    fn request_ebs_volume(&mut self, size: u64) -> Result<String, Box<errors::GenericAWSError>>;
    fn attach_ebs_volume(&mut self, device: String) -> Result<String, Box<errors::GenericAWSError>>;
    fn get_attached_ebs_volumes(&mut self);
    fn delete_ebs_volume(&mut self);
}

struct ConcreteAWS {}

impl AWS for ConcreteAWS {
    fn request_ebs_volume(&mut self, size: u64) -> Result<String, Box<errors::GenericAWSError>> {
        Ok("/dev/test".to_string())
    }
    fn attach_ebs_volume(&mut self, device: String) -> Result<String, Box<errors::GenericAWSError>> {
        Ok("/dev/test".to_string())
    }
    fn get_attached_ebs_volumes(&mut self) {

    }
    fn delete_ebs_volume(&mut self) {

    }
}

pub struct EBSManager {
    config: config::Config,
    diskmgr: Box<dyn DiskMgr>,
    aws: Box<dyn AWS>,
    fs: Box<dyn fs::FS>,
}

impl EBSManager {
    pub fn new(
        conf: config::Config,
        disks: Box<dyn DiskMgr>,
        aws_cli: Box<dyn AWS>,
        fs_lib: Box<dyn fs::FS>
    ) -> Box<EBSManager> {
        Box::new(Self {
            config: conf,
            diskmgr: disks,
            aws: aws_cli,
            fs: fs_lib,
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
            return Err(Box::new(errors::MaxEBSCountExceededError));
        }
        let cur_size = self.diskmgr.disk_size(self.config.mountpoint.clone())?;
        if cur_size >= self.config.limits.max_logical_volume_size.into() {
           return Err(Box::new(errors::MaxLogicalVolumeSizeExceededError));
        }
        let new_size = self.calc_new_size(dev_count).unwrap();
        info!(
            "Will extend volume {} by {}GB",
            self.config.mountpoint.clone(),
            new_size.clone()
        );
        Ok(self.aws.request_ebs_volume(new_size.into())
            .and_then(|dev| self.aws.attach_ebs_volume(dev))
            .and_then(
                |dev|
                self.fs.expand_volume(dev)
                    .map_err(|_e| Box::new(errors::GenericAWSError))
            )?)
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

        fn disk_usage_percent(&mut self, mountpoint: String) -> Result<u32, errors::MountPointNotFoundError> {
            Ok(self.utilization_percentage)
        }

        fn disk_size(&mut self, mountpoint: String) -> Result<u64, errors::MountPointNotFoundError> {
            Ok(self.total_disk_size)
        }
    }

    struct MockAWS {
        simulate_aws_err: bool,
    }

    impl AWS for MockAWS {
        fn request_ebs_volume(&mut self, _size: u64) -> Result<String, Box<errors::GenericAWSError>>{
            if self.simulate_aws_err {
                return Err(Box::new(errors::GenericAWSError))
            }
            Ok("/dev/test".to_string())
        }
        fn attach_ebs_volume(&mut self, _device: String) -> Result<String, Box<errors::GenericAWSError>>{
            if self.simulate_aws_err {
                return Err(Box::new(errors::GenericAWSError))
            }
            Ok("/dev/test".to_string())
        }
        fn get_attached_ebs_volumes(&mut self) {

        }
        fn delete_ebs_volume(&mut self) {

        }
    }


    fn setup(
        mock_diskmgr: MockDiskMgr,
        simulate_aws_err: bool,
        simulate_fs_err: bool
    ) -> Result<Context, Box<dyn Error>> {
        let config : config::Config = Figment::from(Serialized::defaults(config::Config::default()))
            .merge(Toml::file("Test.toml"))
            .extract()?;
        let mock_diskmgr : Box<dyn DiskMgr> = Box::new(mock_diskmgr);
        let mock_aws = Box::new(MockAWS {simulate_aws_err});
        let mock_fs = Box::new(fs::MockFS {simulate_fs_err});
        Ok(Context {
            ebs_manager: EBSManager::new(
                config,
                mock_diskmgr,
                mock_aws,
                mock_fs
            ),
        })
    }

    #[test]
    fn test_power_on_self_test() -> Result<(), io::Error> {
        let ctx = setup(MockDiskMgr::default(), false, false).unwrap();
        assert_eq!(ctx.ebs_manager.power_on_self_test()?, true);
        Ok(())
    }

    #[test]
    fn test_need_more_space_false() -> Result<(), Box<dyn Error>> {
        let mut ctx = setup(MockDiskMgr::default(), false, false).unwrap();
        assert_eq!(ctx.ebs_manager.need_more_space()?, false);
        Ok(())
    }

    #[test]
    fn test_need_more_space_true() -> Result<(), Box<dyn Error>> {
        let mut ctx = setup(MockDiskMgr {
            disks: vec!["test".to_string()],
            utilization_percentage: 95,
            total_disk_size: 100,
        }, false, false).unwrap();
        assert_eq!(ctx.ebs_manager.need_more_space()?, true);
        Ok(())
    }

    #[test]
    fn test_add_more_space_max_ebs_count() -> Result<(), Box<dyn Error>> {
        let mut ctx = setup(MockDiskMgr::default(), false, false).unwrap();
        assert!(ctx.ebs_manager.add_more_space(101).is_err());
        Ok(())
    }

    #[test]
    fn test_add_more_space_max_logical_size() -> Result<(), Box<dyn Error>> {
        let mut ctx = setup(MockDiskMgr {
            disks: vec!["test".to_string()],
            utilization_percentage: 10,
            total_disk_size: 1000,
        }, false, false).unwrap();
        assert!(ctx.ebs_manager.add_more_space(10).is_err());
        Ok(())
    }

    #[test]
    fn test_add_more_space_aws_err() -> Result<(), Box<dyn Error>> {
        let mut ctx = setup(MockDiskMgr::default(), true, false).unwrap();
        assert!(ctx.ebs_manager.add_more_space(1).is_err());
        Ok(())
    }

    #[test]
    fn test_add_more_space_logical_volume_err() -> Result<(), Box<dyn Error>> {
        let mut ctx = setup(MockDiskMgr::default(), false, true).unwrap();
        assert!(ctx.ebs_manager.add_more_space(10).is_err());
        Ok(())
    }

    #[test]
    fn test_add_more_space() -> Result<(), Box<dyn Error>> {
        let mut ctx = setup(MockDiskMgr::default(), false, false).unwrap();
        assert!(ctx.ebs_manager.add_more_space(10).is_ok());
        Ok(())
    }

    #[test]
    fn test_count_mounted_ebs_devices() {
        let ctx = setup(MockDiskMgr::default(), false, false).unwrap();
        assert_eq!(ctx.ebs_manager.count_mounted_ebs_devices(), Some(10));
    }

    #[test]
    fn test_calc_threshold() {
        let ctx = setup(MockDiskMgr::default(), false, false).unwrap();
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
        let ctx = setup(MockDiskMgr::default(), false, false).unwrap();
        assert_eq!(ctx.ebs_manager.calc_new_size(10), Some(10));
    }

    #[test]
    fn test_get_next_logical_device() {
        let ctx = setup(MockDiskMgr::default(), false, false).unwrap();
        assert_eq!(ctx.ebs_manager.get_next_logical_device(), Some("/dev/xvdb".to_string()));
    }
}
