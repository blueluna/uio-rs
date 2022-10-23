use clap::{Arg, Command};
use env_logger;
use std::process::ExitCode;
use uio_rs;

fn main() -> ExitCode {
    env_logger::init();

    let cmd = Command::new("internal-tool")
        .bin_name("internal-tool")
        .arg(
            Arg::new("internal")
                .short('u')
                .long("internal")
                .required(true)
                .value_parser(clap::value_parser!(usize))
                .action(clap::ArgAction::Set),
        )
        .arg(
            Arg::new("map")
                .short('m')
                .long("map")
                .value_parser(clap::value_parser!(usize))
                .action(clap::ArgAction::Set)
                .default_value("0"),
        )
        .arg(
            Arg::new("interrupt")
                .short('i')
                .long("interrupt")
                .action(clap::ArgAction::SetTrue),
        )
        .subcommand(
            Command::new("write")
                .about("Write to the mapped region")
                .arg(
                    Arg::new("value")
                        .index(1)
                        .value_parser(clap::value_parser!(u32))
                        .action(clap::ArgAction::Set)
                        .required(true),
                )
                .arg(
                    Arg::new("offset")
                        .index(2)
                        .value_parser(clap::value_parser!(usize))
                        .action(clap::ArgAction::Set)
                        .default_value("0"),
                ),
        )
        .subcommand(
            Command::new("read")
                .about("Read from the mapped region")
                .arg(
                    Arg::new("offset")
                        .index(1)
                        .value_parser(clap::value_parser!(usize))
                        .action(clap::ArgAction::Set)
                        .required(true),
                ),
        );
    let matches = cmd.get_matches();
    let uio_number = *matches.get_one("internal").unwrap();
    let map_number = *matches.get_one("map").unwrap();

    if *matches.get_one("interrupt").unwrap() {
        let mut interrupt = uio_rs::Interrupt::new(uio_number).expect("Bad interrupt");
        interrupt.enable().expect("Failed to enable interrupt");
        let value = interrupt.wait().expect("Failed to wait for interrupt");
        println!("Interrupt {}", value);
    }

    let mut mem_map = if let Ok(mm) = uio_rs::Map::new(uio_number, map_number) {
        mm
    } else {
        return ExitCode::FAILURE;
    };

    match matches.subcommand() {
        Some(("read", cmd)) => {
            if let Some(offset) = cmd.get_one("offset") {
                match mem_map.read_u32(*offset) {
                    Ok(value) => {
                        println!("{:08x}: {:08x}", offset, value);
                    }
                    Err(e) => {
                        println!("Read {:08x} failed, {:?}", offset, e);
                    }
                }
            }
        }
        Some(("write", cmd)) => {
            if let (Some(offset), Some(value)) = (cmd.get_one("offset"), cmd.get_one("value")) {
                println!("{:08x}: {:08x}", offset, value);
                if let Err(e) = mem_map.write_u32(*offset, *value) {
                    println!("Write {:08x}: {:08x} failed, {:?}", offset, value, e);
                }
            }
        }
        _ => unreachable!("Invalid configuration"),
    }
    ExitCode::SUCCESS
}
