pub mod drive_handle;
pub mod error;
pub mod mount;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct DriveItem {
    pub name: String,
    pub device_path: String,
    pub size: String,
    pub uuid: Option<String>,
    pub is_mounted: bool,
    pub fstype: String,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct DriveConfig {
    pub(crate) name: String,
    pub(crate) device_path: String,
    pub(crate) uuid: String,
}
