use std::error::Error;

use lfs_core::{self, Disk, Mount};
use udev;

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
        .ok_or_else(|| LicenseSerialError::LvmDeviceSlaveNotFound)?;

    let entry = entry?;
    let lvm_physical_disk_name = entry.file_name().to_string_lossy().to_string();

    Ok(lvm_physical_disk_name)
}

fn get_lvm_device_serial(mount: Mount, root_disk: Disk) -> Result<String, LicenseSerialError> {
    let root_mount_info = &mount.info;
    let root_device_id = root_mount_info.dev;

    let disk_name = &root_disk.name;

    let lvm_disk_slaves_dir = format!("/sys/block/{}/slaves", disk_name);

    let lvm_physical_disk_name = read_lvm_slave_name(lvm_disk_slaves_dir)?;

    // println!("{:?}", slave);

    Ok("".to_string())
}

fn get_regular_device_serial(mount: Mount) -> Result<String, LicenseSerialError> {
    let root_mount_info = mount.info;
    let root_device_id = root_mount_info.dev;

    let device_list = lfs_core::BlockDeviceList::read()
        .map_err(|e| LicenseSerialError::DeviceBlockListReadFailure(e))?;

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

fn get_device_serial() -> Result<String, LicenseSerialError> {
    let mounts =
        lfs_core::read_mounts().map_err(|e| LicenseSerialError::MountInfoReadFailure(e))?;

    let root_mount = mounts
        .into_iter()
        .filter(|m| m.info.mount_point.to_string_lossy() == "/")
        .next()
        .ok_or_else(|| LicenseSerialError::RootDeviceIdNotFound)?;

    let root_disk = root_mount
        .disk
        .clone()
        .ok_or_else(|| LicenseSerialError::RootDiskNotFound)?;

    if root_disk.lvm {
        get_lvm_device_serial(root_mount, root_disk)
    } else {
        get_regular_device_serial(root_mount)
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let serial = get_device_serial()?;

    println!("Serial: {}", serial);

    Ok(())
}
