use clap::{arg, command};
use log::error;
use rumqttc::{self, AsyncClient, MqttOptions, QoS};
use std::env;
use std::error::Error;
use std::time::Duration;

use schellen_remote_controller::handle_event;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    pretty_env_logger::init();
    let matches = command!()
        .arg(arg!(--mqtt_broker <URL>).required(true))
        .arg(arg!(--mqtt_topic <TOPIC_NAME>).default_value("#"))
        .arg(arg!(--tty_path <PATH>).default_value("/dev/ttyACM0"))
        .get_matches();

    let broker = matches.try_get_one::<String>("mqtt_broker")?.unwrap();
    let topic = matches.try_get_one::<String>("mqtt_topic")?.unwrap();
    let mqtt_user = env::var("MQTT_USER").or(Err("Provide MQTT_USER!"))?;
    let mqtt_pass = env::var("MQTT_PASS").or(Err("Provide MQTT_PASS!"))?;
    let tty_path = matches.try_get_one::<String>("tty_path")?.unwrap();

    let mut mqttoptions = MqttOptions::new("rumqtt-async", broker, 1883);
    mqttoptions
        .set_credentials(mqtt_user, mqtt_pass)
        .set_keep_alive(Duration::from_secs(60));
    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);
    client.subscribe(topic, QoS::AtMostOnce).await.unwrap();

    while let Ok(event) = eventloop.poll().await {
        if let Err(e) = handle_event(event, tty_path) {
            error!("Error: {:?}", e)
        }
    }

    Ok(())
}
