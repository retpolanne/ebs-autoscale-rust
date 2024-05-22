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
        write!(f, "Maximum EBS logic size exceeded")
    }
}

#[derive(Debug, Clone)]
pub struct GenericAWSError;

impl Error for GenericAWSError {}

impl fmt::Display for GenericAWSError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error calling AWS API")
    }
}

#[derive(Debug, Clone)]
pub struct GenericFSError;

impl Error for GenericFSError {}

impl fmt::Display for GenericFSError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error expanding filesystem")
    }
}
