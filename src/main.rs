use lfs_core;
use udev::{self, Device};

fn device_info(device: &Device) -> Option<String> {
    let serial = device
        .property_value("ID_SERIAL_SHORT")
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string_lossy());
    let dev_name = device
        .property_value("DEVNAME")
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string_lossy());

    if let (Some(serial), Some(dev_name)) = (serial, dev_name) {
        let info = format!("{}     {}", &serial, &dev_name);

        Some(info)
    } else {
        None
    }

    // println!();
    // println!(
    //     "Device: {}, serial: {}",
    //     device().to_string_lossy(),
    //     serial.to_string_lossy()
    // );

    // println!();
    // println!("{:#?}", device);

    // println!("  [properties]");
    // for property in device.properties() {
    //     println!("    - {:?} {:?}", property.name(), property.value());
    // }

    // println!("  [attributes]");
    // for attribute in device.attributes() {
    //     println!("    - {:?} {:?}", attribute.name(), attribute.value());
    // }
}

fn get_root(device: Device) -> Device {
    match device.parent() {
        Some(parent) => {
            if let Some(_) = parent.devtype().filter(|s| *s == "disk") {
                get_root(parent)
            } else {
                device
            }
        }
        None => device,
    }
}

fn main() {
    let root_device_id = lfs_core::read_mountinfo()
        .unwrap()
        .drain(..)
        .filter(|m| m.mount_point.to_string_lossy() == "/")
        .next()
        .map(|m| m.dev)
        .unwrap();

    let device_list = lfs_core::BlockDeviceList::read().unwrap();

    let block_device = device_list.find_by_id(root_device_id).unwrap();
    // println!("{:?}", block_device);

    let mut enumerator = udev::Enumerator::new().unwrap();
    enumerator.match_subsystem("block").unwrap();

    for device in enumerator.scan_devices().unwrap() {

        if device.sysname().to_string_lossy() == block_device.name {
            // println!("{:#?}", device);

            let serial_short = device.property_value("ID_SERIAL_SHORT").map(|s| s.to_string_lossy());
            let serial = device.property_value("ID_SERIAL").map(|s| s.to_string_lossy());

            if let Some(serial_short) = serial_short {
                println!("{}", serial_short);
            } else if let Some(serial) = serial {
                println!("{}", serial);
            } else {
                println!("Failed to find serial!");
            }
        }

        // if let Some(devlinks) = device
        //     .property_value("DEVLINKS")
        //     .map(|s| s.to_string_lossy())
        // {
        //    if devlinks.contains(&root_fs) {
        //         println!("{:#?}", device);

        //         println!("  [properties]");
        //         for property in device.properties() {
        //             println!("    - {:?} {:?}", property.name(), property.value());
        //         }

        //         println!("{}", device.sysname().to_string_lossy());
        //         println!("{}", device.syspath().to_string_lossy());
        //         println!("{}", device.devpath().to_string_lossy());
        //         println!("{}", device.devnode().unwrap().to_string_lossy());
        //         println!("{}", device.devnum().unwrap());
        //    }
        // }

        // // let device = get_root(device.clone());

        // // if let Some(info) = device_info(&device) {
        // println!();
        // // println!("{}", info);
        // println!("{:#?}", device);

        // println!("  [properties]");
        // for property in device.properties() {
        //     println!("    - {:?} {:?}", property.name(), property.value());
        // }

        // // println!("  [attributes]");
        // // for attribute in device.attributes() {
        // //     println!("    - {:?} {:?}", attribute.name(), attribute.value());
        // // }
        // // }
    }
}
