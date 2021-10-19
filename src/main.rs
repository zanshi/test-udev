use std::error::Error;

use lfs_core::{self, DeviceId};
use udev;

#[derive(thiserror::Error, Debug)]
pub enum LicenseSerialError {
    #[error("Failed to mount points")]
    MountPointReadFailure(#[source] lfs_core::Error),
    #[error("Failed to get root file system device ID")]
    RootDeviceIdNotFound,
    #[error("Failed to read device block list")]
    DeviceBlockListReadFailure,
    #[error("Failed to find block device")]
    BlockDeviceNotFound,
    #[error("Failed to scan devices")]
    UdevDeviceScanFailure(#[source] std::io::Error),
    #[error("Failed to find udev device")]
    UdevDeviceNotFound,
    #[error("Serial not found")]
    SerialNotFound,
}

fn get_device_serial() -> Result<String, LicenseSerialError> {
    let mounts =
        lfs_core::read_mountinfo().map_err(|e| LicenseSerialError::MountPointReadFailure(e))?;

    let root_device_id: DeviceId = mounts
        .into_iter()
        .filter(|m| m.mount_point.to_string_lossy() == "/")
        .next()
        .map(|m| m.dev)
        .ok_or_else(|| LicenseSerialError::RootDeviceIdNotFound)?;

    let device_list = lfs_core::BlockDeviceList::read()
        .map_err(|e| LicenseSerialError::DeviceBlockListReadFailure)?;

    let block_device = device_list
        .find_by_id(root_device_id)
        .ok_or_else(|| LicenseSerialError::BlockDeviceNotFound)?;

    let mut enumerator = udev::Enumerator::new().unwrap();
    enumerator.match_subsystem("block").unwrap();

    let devices = enumerator
        .scan_devices()
        .map_err(|e| LicenseSerialError::UdevDeviceScanFailure(e))?;

    let device = devices
        .into_iter()
        .filter(|d| d.sysname().to_string_lossy() == block_device.name)
        .next()
        .ok_or_else(|| LicenseSerialError::UdevDeviceNotFound)?;

    let serial = device
        .property_value("ID_SERIAL_SHORT")
        .or_else(|| device.property_value("ID_SERIAL"))
        .map(|s| s.to_string_lossy().to_string())
        .ok_or_else(|| LicenseSerialError::SerialNotFound)?;

    Ok(serial)
}

fn main() -> Result<(), Box<dyn Error>> {
    let serial = get_device_serial()?;

    println!("Serial: {}", serial);

    Ok(())
}
