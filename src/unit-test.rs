#[cfg(test)]
mod tests {
    use crate::core::drive_handle::*;
    use crate::core::mount::*;
    use crate::core::{DriveConfig, DriveItem};
    use std::collections::HashSet;

    #[test]
    fn test_to_automount_conf() {
        let drive1 = DriveItem {
            name: "/dev/sda1".to_string(),
            device_path: "/dev/sda1".to_string(),
            size: "500 GiB".to_string(),
            uuid: Some("AABBCCDD-1122-3344-5566-778899AABBCC".to_string()),
            is_mounted: true,
            fstype: "ext4".to_string(),
        };

        let drive2 = DriveItem {
            name: "/dev/sdb1".to_string(),
            device_path: "/dev/sdb1".to_string(),
            size: "2 TiB".to_string(),
            uuid: Some("FEEFF00D-0000-0000-0000-000000000000".to_string()),
            is_mounted: false,
            fstype: "ntfs".to_string(),
        };

        let drive3 = DriveItem {
            name: "/dev/sdc".to_string(),
            device_path: "/dev/sdc".to_string(),
            size: "16 GiB".to_string(),
            uuid: None,
            is_mounted: false,
            fstype: "None".to_string(),
        };

        let drives = vec![drive1, drive2, drive3];
        let mut selected_rows = HashSet::new();
        selected_rows.insert(0);
        selected_rows.insert(2);

        let result = to_automount_conf(&drives, &selected_rows);

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].name, "/dev/sda1");
        assert_eq!(result[0].device_path, "/dev/sda1");
        assert_eq!(result[0].uuid, "AABBCCDD-1122-3344-5566-778899AABBCC");
        assert_eq!(result[1].name, "/dev/sdc");
        assert_eq!(result[1].device_path, "/dev/sdc");
        assert_eq!(result[1].uuid, "");
    }

    #[test]
    fn test_clean_udisks_error() {
        let error_msg = "Error: Failed to mount /dev/sda1: org.freedesktop.UDisks2.Error.NotAuthorizedCanObtain";
        let cleaned = clean_udisk_error(error_msg);

        assert!(cleaned.contains("NotAuthorizedCanObtain"));
    }

    #[test]
    fn test_drive_config_creation() {
        let drive_item = DriveItem {
            name: "/dev/sda1".to_string(),
            device_path: "/mnt/data".to_string(),
            size: "1 TB".to_string(),
            uuid: Some("12345678-1234-1234-1234-123456789abc".to_string()),
            is_mounted: false,
            fstype: "ext4".to_string(),
        };

        let config = DriveConfig {
            name: drive_item.name.clone(),
            device_path: drive_item.device_path.clone(),
            uuid: drive_item.uuid.clone().unwrap_or_default(),
        };

        assert_eq!(config.name, "/dev/sda1");
        assert_eq!(config.device_path, "/mnt/data");
        assert_eq!(config.uuid, "12345678-1234-1234-1234-123456789abc");
    }

    #[test]
    fn test_drive_item_properties() {
        let drive_item = DriveItem {
            name: "/dev/sda1".to_string(),
            device_path: "/mnt/data".to_string(),
            size: "1 TB".to_string(),
            uuid: Some("12345678-1234-1234-1234-123456789abc".to_string()),
            is_mounted: true,
            fstype: "ext4".to_string(),
        };

        assert_eq!(drive_item.name, "/dev/sda1");
        assert_eq!(drive_item.fstype, "ext4");
        assert_eq!(drive_item.size, "1 TB");
        assert!(drive_item.is_mounted);
        assert_eq!(
            drive_item.uuid,
            Some("12345678-1234-1234-1234-123456789abc".to_string())
        );
    }
}

