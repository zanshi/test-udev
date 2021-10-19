use std::error::Error;

use lfs_core::{self, DeviceId, Disk, Mount};

#[derive(thiserror::Error, Debug)]
pub enum LicenseSerialError {
    #[error("Failed to mount points")]
    MountInfoReadFailure(#[source] lfs_core::Error),
    #[error("Failed to get root file system device ID")]
    RootDeviceIdNotFound,
    #[error("Failed to get root device disk")]
    RootDiskNotFound,
    #[error("Failed to read device block list")]
    DeviceBlockListReadFailure(#[source] lfs_core::Error),
    #[error("Failed to find block device")]
    BlockDeviceNotFound,
    #[error("Failed to scan devices")]
    UdevDeviceScanFailure(#[source] std::io::Error),
    #[error("Failed to find udev device")]
    UdevDeviceNotFound,
    #[error("Serial not found")]
    SerialNotFound,
    #[error("Slave devices not found for LVM root")]
    LvmDeviceSlavesNotFound(#[source] std::io::Error),
    #[error("Slave device not found for LVM root")]
    LvmDeviceSlaveNotFound,
    #[error("Io Error: `{0}`")]
    IoError(#[from] std::io::Error),
}

fn read_lvm_slave_name(lvm_disk_slaves_dir: String) -> Result<String, LicenseSerialError> {
    let entry = std::fs::read_dir(lvm_disk_slaves_dir)?
        .next()
        .ok_or(LicenseSerialError::LvmDeviceSlaveNotFound)?;

    let entry = entry?;
    let lvm_physical_disk_name = entry.file_name().to_string_lossy().to_string();

    Ok(lvm_physical_disk_name)
}

fn get_device_serial_from_name(name: &str) -> Result<String, LicenseSerialError> {
    let mut enumerator = udev::Enumerator::new().unwrap();
    enumerator.match_subsystem("block").unwrap();

    let devices = enumerator
        .scan_devices()
        .map_err(LicenseSerialError::UdevDeviceScanFailure)?;

    let device = devices
        .into_iter()
        .find(|d| d.sysname().to_string_lossy() == name)
        .ok_or(LicenseSerialError::UdevDeviceNotFound)?;

    let serial = device
        .property_value("ID_SERIAL_SHORT")
        .or_else(|| device.property_value("ID_SERIAL"))
        .map(|s| s.to_string_lossy().to_string())
        .ok_or(LicenseSerialError::SerialNotFound)?;

    Ok(serial)
}

fn get_lvm_device_serial(root_disk: &Disk) -> Result<String, LicenseSerialError> {
    let disk_name = &root_disk.name;

    let lvm_disk_slaves_dir = format!("/sys/block/{}/slaves", disk_name);

    let lvm_physical_disk_name = read_lvm_slave_name(lvm_disk_slaves_dir)?;

    get_device_serial_from_name(&lvm_physical_disk_name)
}

fn get_regular_device_serial(root_device_id: DeviceId) -> Result<String, LicenseSerialError> {
    let device_list = lfs_core::BlockDeviceList::read()
        .map_err(LicenseSerialError::DeviceBlockListReadFailure)?;

    let block_device = device_list
        .find_by_id(root_device_id)
        .ok_or(LicenseSerialError::BlockDeviceNotFound)?;

    get_device_serial_from_name(&block_device.name)
}

fn get_device_serial() -> Result<String, LicenseSerialError> {
    let mounts = lfs_core::read_mounts().map_err(LicenseSerialError::MountInfoReadFailure)?;

    let root_mount = mounts
        .into_iter()
        .find(|m| m.info.mount_point.to_string_lossy() == "/")
        .ok_or(LicenseSerialError::RootDeviceIdNotFound)?;

    println!("{:?}", root_mount);

    let root_disk = &root_mount
        .disk
        .ok_or(LicenseSerialError::RootDiskNotFound)?;

    if root_disk.lvm {
        get_lvm_device_serial(root_disk)
    } else {
        get_regular_device_serial(root_mount.info.dev)
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let serial = get_device_serial()?;

    println!("Serial: {}", serial);

    Ok(())
}
