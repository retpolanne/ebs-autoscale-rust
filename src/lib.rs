use std::io;
use std::error::Error;
use log::{info, trace};
use serde::{Deserialize, Serialize};
use figment::{Figment, providers::{Format, Toml, Serialized}};
use sysinfo::{Disks, Disk};

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
}

impl Default for Config {
    fn default() -> Self {
        Config {
            ensure_ebs_deleted_on_term: true,
            detection_interval: 2
        }
    }
}

pub trait DiskMgr {
    fn new_disks(&mut self);
    fn save_disk_list(&mut self);
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
}

pub struct EBSManager {
    config: Config,
    diskmgr: Box<dyn DiskMgr>,
}

impl EBSManager {
    pub fn new(conf: Config, disks: Box<dyn DiskMgr>) -> Box<EBSManager> {
        Box::new(Self {
            config: conf,
            diskmgr: disks,
        })
    }

    pub fn power_on_self_test(&self) -> Result<bool, io::Error> {
        Ok(true)
    }

    pub fn need_more_space(&self) -> Result<bool, io::Error> {
        let dev_count = self.count_mounted_ebs_devices();
        let threshold = self.calc_threshold(dev_count.unwrap());
        let disk_utilization = self.get_disk_utilization();

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

    fn get_disk_utilization(&self) -> Option<u32> {
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
    }

    fn setup() -> Result<Context, Box<dyn Error>> {
        let config : Config = Figment::from(Serialized::defaults(Config::default()))
            .merge(Toml::file("Test.toml"))
            .extract()?;
        let mock_diskmgr = Box::new(MockDiskMgr {disks: vec!["test".to_string()]});
        Ok(Context {
            ebs_manager: EBSManager::new(config, mock_diskmgr),
        })
    }

    #[test]
    fn test_power_on_self_test() -> Result<(), io::Error> {
        let ctx = setup();
        assert_eq!(ctx.unwrap().ebs_manager.power_on_self_test()?, true);
        Ok(())
    }

    #[test]
    fn test_need_more_space() -> Result<(), io::Error> {
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
    fn test_get_disk_utilization() {
        let ctx = setup();
        assert_eq!(ctx.unwrap().ebs_manager.get_disk_utilization(), Some(10));
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
