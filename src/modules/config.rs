use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::{fs::File, io::BufReader, sync::RwLock};

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    website_ip: String,
    website_port: String,
    devices: Vec<Device>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Device {
    id: String,
    widgets: Vec<Widget>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Widget {
    id: String,
    options: Vec<Option>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Option {
    id: String,
    value: String,
}

// Static Config struct
lazy_static! {
    static ref CONFIG: RwLock<Config> = RwLock::new(Config {
        website_ip: "".to_string(),
        website_port: "".to_string(),
        devices: vec![]
    });
}

/// Initialize Config store
pub fn init_config() {
    let mut config = CONFIG.write().unwrap();
    *config = read_config_from_file("../config.json");
}

fn read_config_from_file(path: &str) -> Config {
    // Open the file in read-only mode with buffer.
    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);

    // parse json file to Config
    let u: Config = serde_json::from_reader(reader).unwrap();

    // Return the Config.
    u
}

pub fn get_widgets_of_device(id: &str) -> Vec<Widget> {
    let devices: Vec<Device> = CONFIG.read().unwrap().devices.clone();
    // Look for device with matching ID
    for d in devices.clone() {
        if d.id.to_lowercase() == id.to_lowercase() {
            return d.widgets;
        }
    }
    // If no matching device was found, check for wildcard device
    for d in devices {
        if d.id == "*" {
            return d.widgets;
        }
    }
    // if device does not exist return empty widget list
    return vec![];
}

pub fn get_website_ip() -> String {
    let website_ip: String = CONFIG.read().unwrap().website_ip.clone();
    website_ip
}

pub fn get_website_port() -> String {
    let website_port: String = CONFIG.read().unwrap().website_port.clone();
    website_port
}
