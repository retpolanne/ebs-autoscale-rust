use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Limits {
    pub initial_utilization_threshold: u32,
    pub min_ebs_volume_size: u32,
    pub max_ebs_volume_size: u32,
    pub max_logical_volume_size: u32,
    pub max_ebs_volume_count: u32
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

    pub fs_type: String,
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
            },
            fs_type: "btrfs".to_string()
        }
    }
}
