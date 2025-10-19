use env_logger;
use std::process::ExitCode;
use uio_rs;

fn main() -> ExitCode {
    env_logger::init();

    for device in uio_rs::DeviceDescription::enumerate().iter() {
        println!(
            "uio{}: {} {} {}",
            device.uio(),
            device.name(),
            device.version(),
            device.event_count(),
        );
        for map_descriptor in device.maps().iter() {
            if map_descriptor.offset() > 0 {
                println!(
                    "  map{}: {:016x}+{:016x} {:016x} {}",
                    map_descriptor.map(),
                    map_descriptor.base_address(),
                    map_descriptor.offset(),
                    map_descriptor.size(),
                    map_descriptor.name().unwrap_or(String::new()),
                );
            } else {
                println!(
                    "  map{}: {:016x} {:016x} {}",
                    map_descriptor.map(),
                    map_descriptor.base_address(),
                    map_descriptor.size(),
                    map_descriptor.name().unwrap_or(String::new()),
                );
            }
        }
    }
    ExitCode::SUCCESS
}
