pub(super) const REMOTE_BOOT_PATCH_DIR: &str = "/data/local/tmp/ranran_boot_patch_workspace";

pub(super) const PAYLOAD_TEMP_DIR_NAME: &str = "ranran-boot-patch-payload";

pub(super) const MAGISK_KIT_DIR_NAME: &str = "magisk-kit";

pub(super) const APATCH_KIT_DIR_NAME: &str = "apatch-kit";

pub(super) const BOOT_PATCH_RESOURCE_DIR_NAME: &str = "boot-patch";

pub(super) const PATCH_MODE_MAGISK: &str = "magisk";

pub(super) const PATCH_MODE_MAGISK_ALPHA: &str = "magisk_alpha";

pub(super) const PATCH_MODE_APATCH: &str = "apatch";

pub(super) const PATCH_MODE_FOLKPATCH: &str = "folkpatch";

pub(super) const PATCH_MODE_KERNELSU: &str = "kernelsu";

pub(super) const PATCH_MODE_KERNELSU_NEXT: &str = "kernelsu_next";

pub(super) const PATCH_MODE_SUKISU_ULTRA: &str = "sukisu_ultra";

pub(super) const APATCH_SUPER_KEY_LENGTH: usize = 24;

pub(super) const APATCH_SUPER_KEY_MIN_LENGTH: usize = 8;

pub(super) const APATCH_SUPER_KEY_MAX_LENGTH: usize = 63;

pub(super) const FLASH_PACKAGE_README_FILE_NAME: &str = "线刷必看.txt";

pub(super) const FLASH_PACKAGE_BAT_FILE_NAME: &str = "双击我获取root.bat";

pub(super) const APATCH_SUPER_KEY_UPPERCASE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ";

pub(super) const APATCH_SUPER_KEY_LOWERCASE: &[u8] = b"abcdefghijklmnopqrstuvwxyz";

pub(super) const APATCH_SUPER_KEY_DIGITS: &[u8] = b"0123456789";

pub(super) const APATCH_SUPER_KEY_CHARSET: &[u8] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";

// 1200 * 100ms = 120000ms = 120秒 我们要最多等待120秒且100ms 轮询一次
pub(super) const DEVICE_MODE_WAIT_ATTEMPTS: usize = 1200;

pub(super) const DEVICE_MODE_WAIT_INTERVAL_MS: u64 = 100;
