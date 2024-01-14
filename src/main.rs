use std::fmt::Display;
use std::fs;
use std::time::Duration;

struct PortConnectedDevice {
    device: rusb::Device<rusb::GlobalContext>,
    device_desc: rusb::DeviceDescriptor,
    ports: Vec<u8>,
}

impl PortConnectedDevice {
    fn from(device: rusb::Device<rusb::GlobalContext>) -> Option<PortConnectedDevice> {
        let device_desc = device.device_descriptor().unwrap();
        device.port_numbers().ok()
            .filter(|p| !p.is_empty())
            .map(|ports| PortConnectedDevice { device, device_desc, ports })
    }

    fn id(&self) -> String {
        format!("{}:{}", self.device_desc.vendor_id(), self.device_desc.product_id())
    }

    fn active(&self) -> bool {
        self.device.active_config_descriptor().map_or(false, |_| true)
    }

    fn toggle(&self) -> std::io::Result<()> {
        if self.active() {
            self.unbind()
        } else {
            self.bind()
        }
    }

    fn path(&self) -> String {
        format!("{}-{}",
                self.device.bus_number(),
                self.ports.iter().map(|p| p.to_string()).collect::<Vec<String>>().join(".")
        )
    }

    fn bind(&self) -> std::io::Result<()> {
        fs::write("/sys/bus/usb/drivers/usb/bind", self.path())
    }

    fn unbind(&self) -> std::io::Result<()> {
        fs::write("/sys/bus/usb/drivers/usb/unbind", self.path())
    }

    fn description(&self) -> String {

        if !self.active() {
            return format!("inactive device with id = {}:{}", self.device_desc.vendor_id(), self.device_desc.product_id());
        }

        self.device.open().ok().map(
            |d| {
                d.read_languages(Duration::from_millis(10))
                    .ok().map(|langs| langs.first().cloned()).flatten()
                    .map(|lang| d.read_product_string(lang, &self.device_desc, Duration::from_millis(10)).ok())
                    .flatten()
                    .or(d.read_product_string_ascii(&self.device_desc).ok())
                    .unwrap_or(format!("{}:{}", self.device_desc.vendor_id(), self.device_desc.product_id()))
            },
        ).expect("failed to open device")
    }
}

impl Display for PortConnectedDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:>10}: [{:<10}] {}", self.path(), self.id(), self.description())
    }
}

fn main() {

    let arg = std::env::args().nth(1);
    if let Some(arg) = arg.as_ref() {
        if arg == "-h" || arg == "--help" {
            println!("Usage: usb-toggle [id]");
            println!("  id: id of the device to toggle");
            println!("      if not provided, all devices are listed");
            return;
        } else if (arg == "-v" || arg == "--version") {
            println!("usb-toggle v{}", env!("CARGO_PKG_VERSION"));
        } else if arg == "-d" || arg == "--disable" {
        } else if arg == "-e" || arg == "--enable" {
        }
    }

    let devices = rusb::devices().unwrap().iter()
        .map(|device| PortConnectedDevice::from(device))
        .flatten()
        .collect::<Vec<PortConnectedDevice>>();

    if let Some(id) = arg {
        devices.into_iter()
            .find(|device| device.id() == id)
            .map(|device| device.toggle());
    } else {
        devices.into_iter().for_each(|device| println!("{}", device));
    }
}
