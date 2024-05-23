use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct GenericAWSError;

impl Error for GenericAWSError {}

impl fmt::Display for GenericAWSError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error calling AWS API")
    }
}

pub trait AWS {
    fn request_ebs_volume(
        &mut self, size: u64, vol_type: String,
        encrypted: bool, throughput: u64,
    ) -> Result<String, Box<GenericAWSError>>;
    fn attach_ebs_volume(&mut self, device: String) -> Result<String, Box<GenericAWSError>>;
    fn get_managed_ebs_volumes(&mut self);
    fn count_mounted_ebs_volumes(&mut self) -> u32;
    fn delete_ebs_volume(&mut self);
    fn tag_as_delete_on_term(&mut self, device: String) -> Result<String, Box<GenericAWSError>>;
}

pub struct ConcreteAWS {}

impl AWS for ConcreteAWS {
    fn request_ebs_volume(
        &mut self, size: u64, vol_type: String,
        encrypted: bool, throughput: u64
    ) -> Result<String, Box<GenericAWSError>> {
        Ok("/dev/test".to_string())
    }
    fn attach_ebs_volume(&mut self, device: String) -> Result<String, Box<GenericAWSError>> {
        Ok("/dev/test".to_string())
    }
    fn get_managed_ebs_volumes(&mut self) {

    }
    fn delete_ebs_volume(&mut self) {

    }

    fn count_mounted_ebs_volumes(&mut self) -> u32 {
        30
    }
    fn tag_as_delete_on_term(&mut self, device: String) -> Result<String, Box<GenericAWSError>> {
        Ok("/dev/test".to_string())
    }
}

pub struct MockAWS {
    pub simulate_aws_err: bool,
    pub mounted_devices_count: u32,
}

impl Default for MockAWS {
    fn default() -> Self {
        MockAWS {
            simulate_aws_err: false,
            mounted_devices_count: 1,
        }
    }
}

impl AWS for MockAWS {
    fn request_ebs_volume(
        &mut self, _size: u64, _vol_type: String,
        _encrypted: bool, _throughput: u64
    ) -> Result<String, Box<GenericAWSError>>{
        if self.simulate_aws_err {
            return Err(Box::new(GenericAWSError))
        }
        Ok("/dev/test".to_string())
    }

    fn attach_ebs_volume(&mut self, _device: String) -> Result<String, Box<GenericAWSError>>{
        if self.simulate_aws_err {
            return Err(Box::new(GenericAWSError))
        }
        Ok("/dev/test".to_string())
    }

    fn get_managed_ebs_volumes(&mut self) {

    }

    fn delete_ebs_volume(&mut self) {

    }

    fn count_mounted_ebs_volumes(&mut self) -> u32 {
        self.mounted_devices_count
    }

    fn tag_as_delete_on_term(&mut self, device: String) -> Result<String, Box<GenericAWSError>> {
        if self.simulate_aws_err {
            return Err(Box::new(GenericAWSError))
        }
        Ok("/dev/test".to_string())
    }
}
