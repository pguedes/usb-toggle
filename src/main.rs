use std::fs;
use colored::Colorize;

trait PortConnectedDevice {
    fn id(&self) -> String;
    fn path(&self) -> &str;
    fn vendor(&self) -> String;
    fn product(&self) -> String;
    fn active(&self) -> bool;
    fn toggle(&self) -> std::io::Result<()>;
    fn bind(&self) -> std::io::Result<()>;
    fn unbind(&self) -> std::io::Result<()>;
    fn description(&self) -> String;
}

struct SysFsDevice {
    path: String,
}

impl PortConnectedDevice for SysFsDevice {
    fn id(&self) -> String {
        let mut id = String::new();
        id.push_str(&self.vendor());
        id.push_str(":");
        id.push_str(&self.product());
        id
    }

    fn path(&self) -> &str {
        self.path.as_str()
    }

    fn vendor(&self) -> String {
        fs::read_to_string(format!("/sys/bus/usb/devices/{}/idVendor", self.path))
            .map(|s| s.trim().to_string())
            .unwrap_or("unknown".to_string())
    }

    fn product(&self) -> String {
        fs::read_to_string(format!("/sys/bus/usb/devices/{}/idProduct", self.path))
            .map(|s| s.trim().to_string())
            .unwrap_or("unknown".to_string())
    }

    fn active(&self) -> bool {
        fs::read_to_string(format!("/sys/bus/usb/devices/{}/bConfigurationValue", self.path))
            .map(|s| s.trim().to_string() != "")
            .unwrap_or(false)
    }

    fn toggle(&self) -> std::io::Result<()> {
        if self.active() {
            self.unbind()
        } else {
            self.bind()
        }
    }

    fn bind(&self) -> std::io::Result<()> {
        fs::write("/sys/bus/usb/drivers/usb/bind", self.path())
    }

    fn unbind(&self) -> std::io::Result<()> {
        fs::write("/sys/bus/usb/drivers/usb/unbind", self.path())
    }

    fn description(&self) -> String {
        fs::read_to_string(format!("/sys/bus/usb/devices/{}/product", self.path))
            .map(|s| s.trim().to_string())
            .unwrap_or(format!("unknown device with id = {}", self.path))
    }
}

// impl Display for SysFsDevice {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{:2}{:>10} {:>width$} {}",
//                if self.active() { "✓" } else { "" },
//                self.id(),
//                self.path(),
//                self.description(),
//                width = 6)
//     }
// }

fn sysfs_devices() -> Vec<SysFsDevice> {
    fs::read_dir("/sys/bus/usb/devices").unwrap().into_iter()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .filter(|id| !id.contains(":"))
        .map(|id| SysFsDevice { path: id })
        .filter(|device| device.vendor() != "1d6b")// linux foundation
        .collect::<Vec<SysFsDevice>>()
}

fn main() {
    let arg = std::env::args().nth(1);
    if let Some(arg) = arg.as_ref() {
        if arg == "-h" || arg == "--help" {
            println!("Usage: usb-toggle [id]");
            println!("  id: id of the device to toggle");
            println!("      if not provided, all devices are listed");
            return;
        } else if arg == "-v" || arg == "--version" {
            println!("usb-toggle v{}", env!("CARGO_PKG_VERSION"));
        }
    }

    let devices = sysfs_devices();

    if let Some(id) = arg {
        devices.into_iter()
            .find(|device| device.id() == id)
            .map(|device| device.toggle());
    } else {
        devices.into_iter()
            .for_each(|device|
                if device.active() {
                    println!("{:2}{:>10} {:>width$} {}",
                             "✓".green().bold(),
                             device.id().bold(),
                             device.path().italic(),
                             device.description(),
                             width = 6)
                } else {
                    println!("{:2}{:>10} {:>width$} {}",
                             "",
                             device.id().truecolor(128, 128, 128).bold(),
                             device.path().truecolor(128, 128, 128).bold(),
                             device.description().truecolor(128, 128, 128).bold(),
                             width = 6)
                }
            );
    }
}
