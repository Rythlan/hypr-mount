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
            mount_point: "/run/media/user/MyBook".to_string(),
            size: "500 GiB".to_string(),
            uuid: Some("AABBCCDD-1122-3344-5566-778899AABBCC".to_string()),
            is_mounted: true,
            fstype: "ext4".to_string(),
            is_luks: false,
        };

        let drive2 = DriveItem {
            name: "/dev/sdb1".to_string(),
            mount_point: "/run/media/user/Windows".to_string(),
            size: "2 TiB".to_string(),
            uuid: Some("FEEFF00D-0000-0000-0000-000000000000".to_string()),
            is_mounted: false,
            fstype: "ntfs".to_string(),
            is_luks: false,
        };

        let drive3 = DriveItem {
            name: "/dev/sdc".to_string(),
            mount_point: String::new(),
            size: "16 GiB".to_string(),
            uuid: None,
            is_mounted: false,
            fstype: "None".to_string(),
            is_luks: false,
        };

        let drives = vec![drive1, drive2, drive3];
        let mut selected_rows = HashSet::new();
        selected_rows.insert(0);
        selected_rows.insert(2);

        let result = to_automount_conf(&drives, &selected_rows);

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].name, "/dev/sda1");
        assert_eq!(result[0].mount_point, "/run/media/user/MyBook");
        assert_eq!(result[0].uuid, "AABBCCDD-1122-3344-5566-778899AABBCC");
        assert_eq!(result[1].name, "/dev/sdc");
        assert_eq!(result[1].mount_point, "");
        assert_eq!(result[1].uuid, "");
    }

    #[test]
    fn test_clean_udisks_error() {
        let error_msg = "Error: Failed to mount /dev/sda1: org.freedesktop.UDisks2.Error.NotAuthorizedCanObtain";
        let cleaned = clean_udisk_error(error_msg);

        assert!(cleaned.contains("NotAuthorizedCanObtain"));
    }

    #[test]
    fn test_clean_udisks_error_multi_colon() {
        // Test the fix: messages with multiple ": " should only split on the last one
        let error_msg = "GDBus.Error(org.freedesktop.UDisks2.error): Failed to mount: device busy";
        let cleaned = clean_udisk_error(error_msg);

        assert_eq!(cleaned, "device busy");
    }

    #[test]
    fn test_clean_udisks_error_no_gdbus() {
        // Non-GDBus errors should pass through unchanged
        let error_msg = "Some random error message";
        let cleaned = clean_udisk_error(error_msg);

        assert_eq!(cleaned, "Some random error message");
    }

    #[test]
    fn test_drive_item_mount_point_field() {
        let drive = DriveItem {
            name: "/dev/sda1".to_string(),
            mount_point: "/mnt/data".to_string(),
            size: "1TB".to_string(),
            uuid: Some("abc-123".to_string()),
            is_mounted: true,
            fstype: "ext4".to_string(),
            is_luks: false,
        };

        assert_eq!(drive.mount_point, "/mnt/data");
        assert_eq!(drive.name, "/dev/sda1");
    }

    #[test]
    fn test_drive_config_creation() {
        let drive_item = DriveItem {
            name: "/dev/sda1".to_string(),
            mount_point: "/run/media/user/MyBook".to_string(),
            size: "1 TB".to_string(),
            uuid: Some("12345678-1234-1234-1234-123456789abc".to_string()),
            is_mounted: false,
            fstype: "ext4".to_string(),
            is_luks: false,
        };

        let config = DriveConfig {
            name: drive_item.name.clone(),
            mount_point: drive_item.mount_point.clone(),
            uuid: drive_item.uuid.clone().unwrap_or_default(),
        };

        assert_eq!(config.name, "/dev/sda1");
        assert_eq!(config.mount_point, "/run/media/user/MyBook");
        assert_eq!(config.uuid, "12345678-1234-1234-1234-123456789abc");
    }

    #[test]
    fn test_drive_item_properties() {
        let drive_item = DriveItem {
            name: "/dev/sda1".to_string(),
            mount_point: "/run/media/user/MyBook".to_string(),
            size: "1 TB".to_string(),
            uuid: Some("12345678-1234-1234-1234-123456789abc".to_string()),
            is_mounted: true,
            fstype: "ext4".to_string(),
            is_luks: false,
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

    #[test]
    fn test_luks_partition_detection() {
        let luks_partition = Partition {
            name: "sda3".to_string(),
            size: "100 GiB".to_string(),
            uuid: Some("luks-12345678-1234-1234-1234-123456789abc".to_string()),
            mountpoints: Vec::new(),
            fstype: Some("crypto_LUKS".to_string()),
        };

        assert!(
            !luks_partition.is_system_drive(),
            "LUKS partition should not be filtered as system drive"
        );
        assert!(
            luks_partition.is_luks(),
            "Partition with fstype crypto_LUKS should be detected as LUKS"
        );

        // Test that non-LUKS partitions still work correctly
        let normal_partition = Partition {
            name: "sda1".to_string(),
            size: "500 MiB".to_string(),
            uuid: Some("abcd-1234".to_string()),
            mountpoints: vec!["/boot".to_string()],
            fstype: Some("ext4".to_string()),
        };

        assert!(
            normal_partition.is_system_drive(),
            "/boot partition should be filtered as system drive"
        );
        assert!(
            !normal_partition.is_luks(),
            "ext4 partition should not be detected as LUKS"
        );

        // Test that swap partitions are still filtered
        let swap_partition = Partition {
            name: "sda2".to_string(),
            size: "8 GiB".to_string(),
            uuid: Some("swap-1234".to_string()),
            mountpoints: Vec::new(),
            fstype: Some("swap".to_string()),
        };

        assert!(
            swap_partition.is_system_drive(),
            "swap partition should be filtered as system drive"
        );
        assert!(
            !swap_partition.is_luks(),
            "swap partition should not be detected as LUKS"
        );
    }
}
