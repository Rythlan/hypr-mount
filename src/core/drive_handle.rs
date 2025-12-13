use crate::core::DriveItem;
use crate::core::error::HyprMountError;
use serde::{Deserialize, Serialize};
use std::process::{Command, Stdio};

#[derive(Serialize, Deserialize, Debug)]
struct LsblkData {
    blockdevices: Vec<Drives>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Drives {
    name: String,
    size: String,
    uuid: Option<String>,
    children: Option<Vec<Partition>>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Partition {
    name: String,
    size: String,
    uuid: Option<String>,
    mountpoints: Vec<String>,
    fstype: Option<String>,
}

impl Partition {
    fn is_system_drive(&self) -> bool {
        self.fstype.as_deref().unwrap_or("None").contains("swap")
            || self.get_device_path().contains("boot")
            || self.name.contains("loop")
    }
    fn get_device_path(&self) -> String {
        self.mountpoints
            .first()
            .map(|s| s.to_string())
            .unwrap_or_default()
    }
}

pub fn list_drives() -> Result<Vec<DriveItem>, HyprMountError> {
    let command = Command::new("lsblk")
        .args(["--json", "-o", "NAME,SIZE,UUID,MOUNTPOINTS,FSTYPE"])
        .output()?;

    let output = command.stdout;
    let converted_output = str::from_utf8(&output)?;

    let mut drives_list: Vec<DriveItem> = vec![];

    let lsblk_data: LsblkData = serde_json::from_str(converted_output)?;

    for drive in lsblk_data.blockdevices {
        if let Some(child_parts) = drive.children {
            for part in child_parts {
                if part.is_system_drive() {
                    continue;
                }
                drives_list.push(DriveItem {
                    name: format!("/dev/{}", part.name),
                    device_path: part.get_device_path(),
                    size: part.size,
                    uuid: part.uuid,
                    is_mounted: !part.mountpoints.is_empty(),
                    fstype: part.fstype.unwrap_or("None".to_string()),
                });
            }
        }
    }
    Ok(drives_list)
}

pub fn mount_drive(uuid: &str) -> Result<(), HyprMountError> {
    run_udisk_command("mount", uuid)
}

pub fn unmount_drive(uuid: &str) -> Result<(), HyprMountError> {
    run_udisk_command("unmount", uuid)
}

fn run_udisk_command(action: &str, uuid: &str) -> Result<(), HyprMountError> {
    if action.contains("mount") || action.contains("unmount") {
        let output = Command::new("udisksctl")
            .arg(action)
            .arg("--block-device")
            .arg(format!("/dev/disk/by-uuid/{}", uuid))
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .output()?;

        udiskctl_error_handle(output)
    } else {
        Err(HyprMountError::UDiskCtlError {
            err_msg: format!("Invalid drive action: {}", action),
        })
    }
}
pub fn clean_udisk_error(stderr: &str) -> String {
    if stderr.contains("GDBus.Error") {
        let parts: Vec<&str> = stderr.split(": ").collect();

        if let Some(part) = parts.last() {
            return part.trim().to_string();
        }
    }
    stderr.trim().to_string()
}

fn udiskctl_error_handle(output: std::process::Output) -> Result<(), HyprMountError> {
    if !output.status.success() {
        let err_msg = clean_udisk_error(str::from_utf8(&output.stderr)?);
        return Err(HyprMountError::UDiskCtlError { err_msg });
    };
    Ok(())
}
