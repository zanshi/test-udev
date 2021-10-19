use lfs_core::{DeviceId, Disk, Mount};
use proc_mounts::MountIter;
use std::{error::Error, path::Path};

#[derive(thiserror::Error, Debug)]
pub enum LicenseSerialError {
    #[error("Udev error")]
    UdevError(#[source] std::io::Error),
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
    #[error("Overlayroot lower_dir not found")]
    OverlayRootLowerDirNotFound,
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
    let mut enumerator = udev::Enumerator::new().map_err(LicenseSerialError::UdevError)?;
    enumerator
        .match_subsystem("block")
        .map_err(LicenseSerialError::UdevError)?;

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

fn get_root_mount() -> Result<Mount, LicenseSerialError> {
    let mounts = lfs_core::read_mounts().map_err(LicenseSerialError::MountInfoReadFailure)?;

    let mut root_mount = mounts
        .iter()
        .find(|m| m.info.mount_point == Path::new("/"))
        .ok_or(LicenseSerialError::RootDeviceIdNotFound)?;

    if root_mount.info.fs == "overlayroot" {
        let proc_mounts = MountIter::new()?;

        let mut proc_mounts = proc_mounts.filter_map(|m| m.ok());

        let overlay_root_mount = proc_mounts
            .find(|m| m.dest == Path::new("/"))
            .ok_or(LicenseSerialError::RootDeviceIdNotFound)?;

        let lower_dir = overlay_root_mount
            .options
            .iter()
            .find(|s| s.starts_with("lowerdir="))
            .ok_or(LicenseSerialError::OverlayRootLowerDirNotFound)?
            .trim_start_matches("lowerdir=");

        root_mount = mounts
            .iter()
            .find(|m| m.info.mount_point == Path::new(lower_dir))
            .ok_or(LicenseSerialError::RootDeviceIdNotFound)?;
    }

    let root_mount = root_mount.clone();

    Ok(root_mount)
}

fn get_device_serial() -> Result<String, LicenseSerialError> {
    let root_mount = get_root_mount()?;

    let root_disk = root_mount
        .disk
        .as_ref()
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
