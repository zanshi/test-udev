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
    let mut enumerator = udev::Enumerator::new().unwrap();

    enumerator.match_subsystem("block").unwrap();

    for device in enumerator.scan_devices().unwrap() {
        // let device = get_root(device.clone());

        // if let Some(info) = device_info(&device) {
        println!();
        // println!("{}", info);
        println!("{:#?}", device);

        println!("  [properties]");
        for property in device.properties() {
            println!("    - {:?} {:?}", property.name(), property.value());
        }

        // println!("  [attributes]");
        // for attribute in device.attributes() {
        //     println!("    - {:?} {:?}", attribute.name(), attribute.value());
        // }
        // }
    }
}
