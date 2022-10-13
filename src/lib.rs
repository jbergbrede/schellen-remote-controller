use log::{debug, info};
use rumqttc::{Event, Packet, Publish};
use std::error::Error;
use tokio_serial::SerialPortBuilderExt;

enum Command {
    Stop, // 0x00
    Up,   // 0x01
    Down, // 0x02
}

fn send_command(payload: String, tty_path: &String) -> Result<(), Box<dyn Error>> {
    info!("Sending command: {:?}", payload);
    let mut port = tokio_serial::new(tty_path, 9600).open_native_async()?;
    port.set_exclusive(false)?;
    let size = port.try_write(payload.as_bytes())?;
    Ok(info!("{} bytes written!", size))
}

fn make_payload(cmd: Command, device_id: String) -> String {
    let hex_cmd = match cmd {
        Command::Stop => "00",
        Command::Up => "01",
        Command::Down => "02",
    };
    format!("ss{}9{}0000", device_id, hex_cmd)
}

fn match_command(packet: Publish) -> Result<Command, Box<dyn Error>> {
    match packet.topic.split("/").last() {
        Some("up") => Ok(Command::Up),
        Some("down") => Ok(Command::Down),
        Some("stop") => Ok(Command::Stop),
        _ => Err("Invalid command!")?,
    }
}

pub fn handle_event(event: Event, tty_path: &String) -> Result<(), Box<dyn Error>> {
    match event {
        rumqttc::Event::Incoming(Packet::Publish(packet)) => match_command(packet)
            .map(|cmd| make_payload(cmd, String::from("11")))
            .and_then(|payload| send_command(payload, tty_path)),
        _ => Ok(debug!("{:?}", event)),
    }
}
