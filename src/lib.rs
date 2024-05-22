use std::io;
use log::{info, trace};

pub fn need_more_space() -> Result<bool, io::Error> {
    let dev_count = count_mounted_ebs_devices();
    let threshold = calc_threshold(dev_count.unwrap());
    let disk_utilization = get_disk_utilization();

    if disk_utilization >= threshold {
        info!("Low disk space - adding more disks");
        add_more_space(dev_count.unwrap())?;
    }
    Ok(true)
}

pub fn add_more_space(dev_count: u32) -> Result<bool, io::Error> {
    Ok(true)
}

fn count_mounted_ebs_devices() -> Option<u32> {
    Some(10)
}

fn calc_threshold(dev_count: u32) -> Option<u32> {
    Some(dev_count)
}

fn calc_new_size(dev_count: u32) -> Option<u32> {
    Some(10)
}

fn get_disk_utilization() -> Option<u32> {
    Some(10)
}

fn get_next_logical_device() -> Option<String> {
    Some("/dev/xvdb".to_string())
}

fn request_new_ebs_volume(size: u32) -> Result<bool, io::Error> {
    Ok(true)
}

fn extend_logical_device() -> Result<bool, io::Error> {
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_need_more_space() -> Result<(), io::Error> {
        assert_eq!(need_more_space()?, true);
        Ok(())
    }

    #[test]
    fn test_request_new_ebs_volume() -> Result<(), io::Error> {
        assert_eq!(request_new_ebs_volume(32)?, true);
        Ok(())
    }

    #[test]
    fn test_count_mounted_ebs_devices() {
        assert_eq!(count_mounted_ebs_devices(), Some(10));
    }

    #[test]
    fn test_calc_threshold() {
        assert_eq!(calc_threshold(10), Some(10));
    }

    #[test]
    fn test_calc_new_size() {
        assert_eq!(calc_new_size(10), Some(10));
    }

    #[test]
    fn test_get_disk_utilization() {
        assert_eq!(get_disk_utilization(), Some(10));
    }

    #[test]
    fn test_get_next_logical_device() {
        assert_eq!(get_next_logical_device(), Some("/dev/xvdb".to_string()));
    }

    #[test]
    fn test_extend_logical_device() -> Result<(), io::Error> {
        assert_eq!(extend_logical_device()?, true);
        Ok(())
    }
}
