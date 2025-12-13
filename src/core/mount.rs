use crate::core::error::HyprMountError;
use crate::core::{DriveConfig, DriveItem, drive_handle};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::fs::OpenOptions;
use std::io::Read;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug)]
struct AutoMountConf {
    auto_mount_drives: Vec<DriveConfig>,
}

impl From<&DriveItem> for DriveConfig {
    fn from(drive: &DriveItem) -> Self {
        DriveConfig {
            device_path: drive.device_path.to_owned(),
            name: drive.name.to_owned(),
            uuid: drive.uuid.as_deref().unwrap_or("").to_string(),
        }
    }
}

pub fn get_config_path() -> Result<PathBuf, HyprMountError> {
    let home_dir = std::env::var("HOME")?;
    let conf_path = PathBuf::from(home_dir)
        .join(".config")
        .join("hypr-mount")
        .join("automount.json");
    Ok(conf_path)
}

pub fn auto_mount() -> Result<(), HyprMountError> {
    let config_content = read_config_file()?;
    let parsed_conf: Vec<DriveConfig> = serde_json::from_str(&config_content)?;

    for item in parsed_conf.iter() {
        drive_handle::mount_drive(&item.uuid)?;
    }

    Ok(())
}

fn read_config_file() -> Result<String, HyprMountError> {
    let location = get_config_path()?;
    let mut file = OpenOptions::new().read(true).open(location)?;
    let mut s = String::new();
    file.read_to_string(&mut s)?;

    if s.is_empty() {
        return Err(HyprMountError::Parse(
            "Content of the config file is empty!".to_string(),
        ));
    }
    Ok(s)
}

pub fn to_automount_conf(drives: &[DriveItem], selected_rows: &HashSet<usize>) -> Vec<DriveConfig> {
    drives
        .iter()
        .enumerate()
        .filter_map(|(idx, drive)| {
            if selected_rows.contains(&idx) {
                Some(drive.into())
            } else {
                None
            }
        })
        .collect::<Vec<DriveConfig>>()
}

pub fn driveconf_to_vec_json(conf: Vec<DriveConfig>) -> Result<Vec<u8>, HyprMountError> {
    Ok(serde_json::to_vec_pretty(&conf)?)
}

pub fn driveconf_to_string(conf: Vec<DriveConfig>) -> Result<String, HyprMountError> {
    Ok(serde_json::to_string_pretty(&conf)?)
}

pub fn driveconf_to_string_censored(conf: Vec<DriveConfig>) -> Result<String, HyprMountError> {
    let censored_conf: Vec<DriveConfig> = conf
        .into_iter()
        .map(|mut drive_config| {
            drive_config.uuid = format!("{}...", drive_config.uuid.split_at(8).0);
            drive_config
        })
        .collect();

    Ok(serde_json::to_string_pretty(&censored_conf)?)
}

pub fn driveconf_script_gen(conf: Vec<DriveConfig>) -> Result<(), HyprMountError> {
    let config_data = driveconf_to_vec_json(conf)?;
    let location = get_config_path()?;

    if let Some(parent_dir) = location.parent() {
        fs::create_dir_all(parent_dir)?;
        fs::write(location, config_data)?;
    }
    Ok(())
}

pub fn automount_drives_service() -> Result<(), HyprMountError> {
    let home = std::env::var("HOME")?;
    let config_name = "hypr-mount.service";
    let exec_path = std::env::current_exe()?;

    let exec_path_str = exec_path.to_str()
        .ok_or(HyprMountError::ExePath())?;

    let dir = PathBuf::from(home)
        .join(".config")
        .join("systemd")
        .join("user");

    let script_text = format!(
        "[Unit]\n\
         Description={}\n\
         After=default.target\n\
         \n\
         [Service]\n\
         ExecStart={} --auto-mount\n\
         Restart=on-failure\n\
         \n\
         [Install]\n\
         WantedBy=default.target",
        "hypr-mount service",
        exec_path_str
    );

    fs::create_dir_all(&dir)?;
    fs::write(dir.join(config_name), &script_text)?;

    println!(
        "✅ Successfully generated systemd service at:\n{:?}\n",
        dir.join(config_name)
    );
    println!("To enable automounting, run these commands:");
    println!("  systemctl --user daemon-reload");
    println!("  systemctl --user enable --now {}", config_name);
    Ok(())
}
