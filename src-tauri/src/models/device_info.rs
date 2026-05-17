use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct DeviceEntry {
    pub serial: String,
    pub state: String,
    pub source: String,
}

#[derive(Serialize)]
pub struct DeviceScanReport {
    pub raw_output: String,
    pub devices: Vec<DeviceEntry>,
}

#[derive(Serialize)]
pub struct DeviceScanSnapshot {
    pub devices: Vec<DeviceEntry>,
    pub adb_report: DeviceScanReport,
    pub fastboot_report: DeviceScanReport,
    pub duration_ms: u64,
}

#[derive(Serialize)]
pub struct DeviceBasicInfo {
    pub device_name: String,
    pub device_codename: String,
    pub serial: String,
    pub state: String,
    pub brand: String,
    pub android_version: String,
    pub os_version: String,
    pub cpu_codename: String,
    pub cpu_arch: String,
    pub hardware_platform: String,
    pub board_id: String,
    pub resolution: String,
    pub display_density: String,
    pub unlock_state: String,
    pub ab_slot: String,
    pub vndk_version: String,
    pub uptime: String,
    pub build_date: String,
    pub build_version: String,
    pub fingerprint: String,
    pub kernel_version: String,
    pub manufacturer: String,
    pub product_model: String,
    pub product_name: String,
    pub security_patch: String,
    pub vendor_security_patch: String,
    pub build_incremental: String,
    pub build_type: String,
    pub build_tags: String,
    pub baseband_version: String,
    pub soc_manufacturer: String,
    pub soc_model: String,
    pub cpu_abilist: String,
}

#[derive(Serialize)]
pub struct ResourceInfo {
    pub storage_used_gb: f64,
    pub storage_total_gb: f64,
    pub storage_percent: u32,
    pub memory_used_gb: f64,
    pub memory_total_gb: f64,
    pub memory_percent: u32,
    pub battery_level: u32,
    pub battery_temp: f64,
}

#[derive(Serialize)]
pub struct PartitionEntry {
    pub name: String,
    pub block_device: String,
}

#[derive(Serialize)]
pub struct AppStatus {
    pub system_count: u32,
    pub user_count: u32,
    pub total_count: u32,
}

#[derive(Serialize)]
pub struct PackageInfo {
    pub package_name: String,
    pub is_system: bool,
    pub is_enabled: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AppLabelEntry {
    pub package_name: String,
    pub label: String,
    pub uid: u32,
    pub version_name: String,
    pub source_dir: String,
}

#[derive(Serialize)]
pub struct PropEntry {
    pub key: String,
    pub value: String,
}

#[derive(Serialize)]
pub struct MdnsDevice {
    pub instance_name: String,
    pub service_type: String,
    pub ip: String,
    pub port: String,
}
