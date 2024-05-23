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
    fn request_ebs_volume(&mut self, size: u64) -> Result<String, Box<GenericAWSError>>;
    fn attach_ebs_volume(&mut self, device: String) -> Result<String, Box<GenericAWSError>>;
    fn get_attached_ebs_volumes(&mut self);
    fn delete_ebs_volume(&mut self);
}

pub struct ConcreteAWS {}

impl AWS for ConcreteAWS {
    fn request_ebs_volume(&mut self, size: u64) -> Result<String, Box<GenericAWSError>> {
        Ok("/dev/test".to_string())
    }
    fn attach_ebs_volume(&mut self, device: String) -> Result<String, Box<GenericAWSError>> {
        Ok("/dev/test".to_string())
    }
    fn get_attached_ebs_volumes(&mut self) {

    }
    fn delete_ebs_volume(&mut self) {

    }
}

pub struct MockAWS {
    pub simulate_aws_err: bool,
}

impl AWS for MockAWS {
    fn request_ebs_volume(&mut self, _size: u64) -> Result<String, Box<GenericAWSError>>{
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
    fn get_attached_ebs_volumes(&mut self) {

    }
    fn delete_ebs_volume(&mut self) {

    }
}
