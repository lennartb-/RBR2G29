use hidapi::{DeviceInfo, HidApi, HidDevice};
use rbr2g29::common::leds::LEDS;
use rbr2g29::common::util::{RBR2G29Result, G27_PID, G29_PID, G920_PID, LOGITECH_VID};
use std::net::UdpSocket;
use std::thread::sleep;
use std::time::Duration;

fn read_telemetry_and_update(device: HidDevice) -> RBR2G29Result {
    let socket = UdpSocket::bind("127.0.0.1:6776")?;
    let mut leds = LEDS::new(device);
    let mut data = [0; 664];
    let mut rbr = rbr2g29::common::rbr::RBR::new();

    println!("Looking for RBR process...");
    loop {        
        match rbr.initialize() {
            Err(error) => {                
                sleep(Duration::from_secs(1));
            }
            _ => break,
        }        
    }

    loop {
        match socket.recv(&mut data) {
            Ok(_) => leds.update(&data, &rbr)?,
            Err(e) => println!("recv function failed: {e:?}"),
        };
    }
}

fn device_connected(hid: &HidApi) -> Option<DeviceInfo> {
    println!("Looking for devices...");
    for device in hid.device_list() {
        if device.vendor_id() != LOGITECH_VID {
            continue;
        }

        if device.product_id() == G27_PID {
            println!("Found G27: {}", device.interface_number());
            return Some(device.clone());
        }

        // G29 will appear multiple times as HID device, and only the one with interface number 0 seems to do anything with the RPM data send to it.
        if (device.product_id() == G29_PID || device.product_id() == G920_PID)
            && device.interface_number() == 0
        {
            println!("Found G29");
            return Some(device.clone());
        }
    }

    None
}

fn connect_and_bridge() -> RBR2G29Result {
    println!("Initializing");
    let mut hid = HidApi::new()?;

    match device_connected(&hid) {
        Some(device) => {
            let dev = device.open_device(&hid)?;
            println!("Connected");
            read_telemetry_and_update(dev)?;
        }
        None => println!("Could not find G27 or G29"),
    }
    sleep(Duration::from_secs(1));
    hid.refresh_devices()?;
    Ok(())
}

fn main() {
    loop {
        if let Err(error) = connect_and_bridge() {
            println!("{:?}", error);
        }

        sleep(Duration::from_secs(1));
    }
}
