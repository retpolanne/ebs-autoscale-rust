pub mod config;
mod fs;
mod aws;
mod disk;

use std::io;
use std::error::Error;
use log::info;

use std::fmt;

#[derive(Debug, Clone)]
pub struct MaxEBSCountExceededError;

impl Error for MaxEBSCountExceededError {}

impl fmt::Display for MaxEBSCountExceededError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Maximum number of EBS volumes exceeded")
    }
}

#[derive(Debug, Clone)]
pub struct MaxLogicalVolumeSizeExceededError;

impl Error for MaxLogicalVolumeSizeExceededError {}

impl fmt::Display for MaxLogicalVolumeSizeExceededError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Maximum logical volume size exceeded")
    }
}

pub struct EBSManager {
    config: config::Config,
    diskmgr: Box<dyn disk::DiskMgr>,
    aws: Box<dyn aws::AWS>,
    fs: Box<dyn fs::FS>,
}

impl EBSManager {
    pub fn new(
        conf: config::Config,
        disks: Box<dyn disk::DiskMgr>,
        aws_cli: Box<dyn aws::AWS>,
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
        let dev_count = self.aws.count_mounted_ebs_volumes();
        let threshold = self.calc_threshold(dev_count).unwrap();
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
        let cur_size = self.diskmgr.disk_size(self.config.mountpoint.clone())?;
        if cur_size >= self.config.limits.max_logical_volume_size.into() {
           return Err(Box::new(MaxLogicalVolumeSizeExceededError));
        }
        let new_size = self.calc_new_size(dev_count).unwrap();
        let created_volumes = self.aws.get_managed_ebs_volumes();
        // TODO - check AWS payload to filter by this
        // https://github.com/awslabs/amazon-ebs-autoscale/blob/8c8ac28914bf9302e4f5821ba99daac6e7fc05eb/bin/create-ebs-volume#L192
        let attached_volumes_count = created_volumes
            .filter(
                |ebs_data|
                ebs_data.as_ref()
                        .map_or(false, true)
            );
        // TODO - same here
        // https://github.com/awslabs/amazon-ebs-autoscale/blob/8c8ac28914bf9302e4f5821ba99daac6e7fc05eb/bin/create-ebs-volume#L223
        // https://blog.malware.re/2022/12/23/Rust-filter-map/index.html
        let total_created_volumes_size = created_volumes
            .map();
        if total_created_volumes_size >= self.config.limits.max_logical_volume_size {
            return Err(Box::new(MaxLogicalVolumeSizeExceededError));
        }
        if attached_volumes_count >= self.config.limits.max_ebs_volume_count ||
            created_volumes.len() >= self.config.limits.max_ebs_volume_count
        {
            return Err(Box::new(MaxEBSCountExceededError))
        }
        info!(
            "Will extend volume {} by {}GB",
            self.config.mountpoint.clone(),
            new_size.clone()
        );
        Ok(self.aws.request_ebs_volume(
            new_size.into(),
            self.config.volume.vol_type,
            self.config.volume.encrypted,
            self.config.volume.throughput,
        )
            .and_then(
                |_ret|
                self.diskmgr.get_next_logical_device()
                    .map_err(|_e| Box::new(aws::GenericAWSError))
            )
            .and_then(|dev| self.aws.attach_ebs_volume(dev))
            .and_then(|dev| self.aws.tag_as_delete_on_term(dev))
            .and_then(
                |dev|
                self.fs.expand_volume(dev)
                    .map_err(|_e| Box::new(aws::GenericAWSError))
            )?)
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
}


#[cfg(test)]
mod tests {
    use super::*;
    use figment::{Figment, providers::{Format, Toml, Serialized}};

    struct Context {
        ebs_manager: Box<EBSManager>,
    }


    fn setup(
        mock_diskmgr: disk::MockDiskMgr,
        mock_aws: aws::MockAWS,
        mock_fs: fs::MockFS,
    ) -> Result<Context, Box<dyn Error>> {
        let config : config::Config = Figment::from(Serialized::defaults(config::Config::default()))
            .merge(Toml::file("Test.toml"))
            .extract()?;
        let mock_diskmgr : Box<dyn disk::DiskMgr> = Box::new(mock_diskmgr);
        let mock_aws = Box::new(mock_aws);
        let mock_fs = Box::new(mock_fs);
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
        let mut ctx = setup(
            disk::MockDiskMgr::default(),
            aws::MockAWS::default(),
            fs::MockFS::default()
        ).unwrap();
        assert_eq!(ctx.ebs_manager.power_on_self_test()?, true);
        Ok(())
    }

    #[test]
    fn test_need_more_space_false() -> Result<(), Box<dyn Error>> {
        let mut ctx = setup(
            disk::MockDiskMgr::default(),
            aws::MockAWS::default(),
            fs::MockFS::default()
        ).unwrap();
        assert_eq!(ctx.ebs_manager.need_more_space()?, false);
        Ok(())
    }

    #[test]
    fn test_need_more_space_true() -> Result<(), Box<dyn Error>> {
        let mut ctx = setup(disk::MockDiskMgr {
            disks: vec!["test".to_string()],
            utilization_percentage: 95,
            total_disk_size: 100,
            sim_no_more_device_names: false,
        }, aws::MockAWS::default(), fs::MockFS::default()).unwrap();
        assert_eq!(ctx.ebs_manager.need_more_space()?, true);
        Ok(())
    }

    #[test]
    fn test_add_more_space_no_available_dev_name() -> Result<(), Box<dyn Error>> {
        let mut ctx = setup(disk::MockDiskMgr {
            disks: vec!["test".to_string()],
            utilization_percentage: 95,
            total_disk_size: 100,
            sim_no_more_device_names: true,
        }, aws::MockAWS::default(), fs::MockFS::default()).unwrap();
        assert!(ctx.ebs_manager.add_more_space(101).is_err());
        Ok(())
    }

    #[test]
    fn test_add_more_space_max_ebs_count() -> Result<(), Box<dyn Error>> {
        let mut ctx = setup(
            disk::MockDiskMgr::default(),
            aws::MockAWS::default(),
            fs::MockFS::default()
        ).unwrap();
        assert!(ctx.ebs_manager.add_more_space(101).is_err());
        Ok(())
    }

    #[test]
    fn test_add_more_space_max_logical_size() -> Result<(), Box<dyn Error>> {
        let mut ctx = setup(disk::MockDiskMgr {
            disks: vec!["test".to_string()],
            utilization_percentage: 10,
            total_disk_size: 1000,
            sim_no_more_device_names: false,
        }, aws::MockAWS::default(), fs::MockFS::default()).unwrap();
        assert!(ctx.ebs_manager.add_more_space(10).is_err());
        Ok(())
    }

    #[test]
    fn test_add_more_space_aws_err() -> Result<(), Box<dyn Error>> {
        let mut ctx = setup(
            disk::MockDiskMgr::default(),
            aws::MockAWS {
                simulate_aws_err: true,
                mounted_devices_count: 1
            },
            fs::MockFS::default()
        ).unwrap();
        assert!(ctx.ebs_manager.add_more_space(1).is_err());
        Ok(())
    }

    #[test]
    fn test_add_more_space_logical_volume_err() -> Result<(), Box<dyn Error>> {
        let mut ctx = setup(
            disk::MockDiskMgr::default(),
            aws::MockAWS::default(),
            fs::MockFS {
                simulate_fs_err: true
            }
        ).unwrap();
        assert!(ctx.ebs_manager.add_more_space(10).is_err());
        Ok(())
    }

    #[test]
    fn test_add_more_space() -> Result<(), Box<dyn Error>> {
        let mut ctx = setup(
            disk::MockDiskMgr::default(),
            aws::MockAWS::default(),
            fs::MockFS::default()
        ).unwrap();
        assert!(ctx.ebs_manager.add_more_space(10).is_ok());
        Ok(())
    }

    #[test]
    fn test_calc_threshold() {
        let mut ctx = setup(
            disk::MockDiskMgr::default(),
            aws::MockAWS::default(),
            fs::MockFS::default()
        ).unwrap();
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
        // TODO https://github.com/awslabs/amazon-ebs-autoscale/blob/8c8ac28914bf9302e4f5821ba99daac6e7fc05eb/bin/ebs-autoscale#L123
        let mut ctx = setup(
            disk::MockDiskMgr::default(),
            aws::MockAWS::default(),
            fs::MockFS::default()
        ).unwrap();
        assert_eq!(ctx.ebs_manager.calc_new_size(1),
                   Some(ctx.ebs_manager.config.limits.initial_utilization_threshold)
        );
        for dev_count in [4, 5, 6] {
            assert_eq!(ctx.ebs_manager.calc_new_size(dev_count), Some(80));
        }
        for dev_count in [7, 8, 9, 10] {
            assert_eq!(ctx.ebs_manager.calc_new_size(dev_count), Some(90));
        }
        assert_eq!(ctx.ebs_manager.calc_new_size(11), Some(90));
    }
}
