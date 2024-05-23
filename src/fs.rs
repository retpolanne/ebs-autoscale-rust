use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct GenericFSError;

impl Error for GenericFSError {}

impl fmt::Display for GenericFSError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error expanding filesystem")
    }
}

pub trait FS {
    fn expand_volume(&self, dev: String) -> Result<bool, Box<GenericFSError>>;
}

pub struct ConcreteFS {
    pub fs_type: String
}

impl FS for ConcreteFS {
    fn expand_volume(&self, _dev: String) -> Result<bool, Box<GenericFSError>> {
        Ok(true)
    }
}

pub struct MockFS {
    pub simulate_fs_err: bool,
}

impl Default for MockFS {
    fn default() -> Self {
        MockFS {
            simulate_fs_err: false,
        }
    }
}

impl FS for MockFS {
    fn expand_volume(&self, _dev: String) -> Result<bool, Box<GenericFSError>> {
        if self.simulate_fs_err {
            return Err(Box::new(GenericFSError))
        }
        Ok(true)
    }
}
