use postcard::experimental::max_size::MaxSize;
use postcard_bindgen::PostcardBindings;
use uuid::fmt::Hyphenated;

use crate::common_sizes::{
    MAX_GROUP_NAME_LEN, MAX_PASSWORD_LEN, MAX_PRODUCER_NAME_LEN, MAX_USERNAME_LEN,
};
pub const PK_SIZE: usize = 32;
pub const MAX_BROKER_HOSTNAME_SIZE: usize = 50;
pub const MAX_SSID_SIZE: usize = 128;
pub const MAX_WIFI_PASS_SIZE: usize = 128;
pub const DEFAULT_MQTT_BROKER_PORT: u16 = 1883;

const fn default_mqtt_broker_port() -> u16 {
    DEFAULT_MQTT_BROKER_PORT
}

#[derive(
    Debug, defmt::Format, serde::Deserialize, serde::Serialize, PostcardBindings, MaxSize, Clone,
)]
pub enum BrokerIp {
    Ip([u8; 4]),
    DnsName(heapless::String<{ MAX_BROKER_HOSTNAME_SIZE }>),
}

#[derive(
    Debug, defmt::Format, serde::Deserialize, serde::Serialize, PostcardBindings, MaxSize, Clone,
)]
pub struct WifiComm {
    pub ssid: heapless::String<{ MAX_SSID_SIZE }>,
    pub pass: heapless::String<{ MAX_WIFI_PASS_SIZE }>,
    pub broker_ip: BrokerIp,
    #[serde(default = "default_mqtt_broker_port")]
    pub broker_port: u16,
    pub mqtt_username: heapless::String<{ MAX_USERNAME_LEN }>,
    pub mqtt_password: heapless::String<{ MAX_PASSWORD_LEN }>,
}

#[derive(
    Debug,
    defmt::Format,
    serde::Deserialize,
    serde::Serialize,
    PostcardBindings,
    Clone,
    Copy,
    MaxSize,
)]
pub struct LoraComm {
    pub forwarder_public_key: [u8; PK_SIZE],
    pub forwarder_lora_addr: u16,
}

#[derive(
    Debug, defmt::Format, serde::Deserialize, serde::Serialize, PostcardBindings, MaxSize, Clone,
)]
#[allow(clippy::large_enum_variant)]
pub enum Communication {
    Lora(LoraComm),
    Wifi(WifiComm),
}

#[derive(
    Debug, defmt::Format, serde::Deserialize, serde::Serialize, PostcardBindings, MaxSize, Clone,
)]
pub struct Config {
    // pub server_public_key: [u8; PK_SIZE],
    pub communication: Communication,
    pub forward_lora: Option<bool>,
    pub producer_name: heapless::String<{ MAX_PRODUCER_NAME_LEN }>,
    pub group_name: heapless::String<{ MAX_GROUP_NAME_LEN }>,
    pub data_template_uuid: Option<heapless::String<{ Hyphenated::LENGTH }>>,
}

impl Config {
    pub fn uses_lora(&self) -> bool {
        self.forward_lora.is_some() || matches!(self.communication, Communication::Lora(_))
    }

    pub fn validate(self) -> Result<Self, ()> {
        if self.producer_name.is_empty() || contains_mqtt_topic_meta(self.producer_name.as_str()) {
            return Err(());
        }

        if self.group_name.is_empty() || contains_mqtt_topic_meta(self.group_name.as_str()) {
            return Err(());
        }

        if let Some(template_uuid) = &self.data_template_uuid {
            if !is_valid_hyphenated_uuid(template_uuid.as_str()) {
                return Err(());
            }
        }

        if matches!(self.communication, Communication::Lora(_))
            && matches!(self.forward_lora, Some(true))
        {
            return Err(());
        }

        match &self.communication {
            Communication::Wifi(wifi) => {
                if wifi.ssid.is_empty()
                    || wifi.pass.is_empty()
                    || wifi.mqtt_username.is_empty()
                    || wifi.mqtt_password.is_empty()
                {
                    return Err(());
                }

                if wifi.broker_port == 0 {
                    return Err(());
                }

                if let BrokerIp::DnsName(name) = &wifi.broker_ip {
                    if name.is_empty() || !is_valid_dns_name(name.as_str()) {
                        return Err(());
                    }
                }
            }
            Communication::Lora(_) => {}
        }

        Ok(self)
    }

    pub fn needs_signal_discovery(&self) -> bool {
        matches!(self.communication, Communication::Lora(_))
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, defmt::Format, MaxSize, PostcardBindings)]
pub enum SignalProtocolStatus {
    NotPeered,
    Signal(u8),
}

impl From<u8> for SignalProtocolStatus {
    fn from(value: u8) -> Self {
        Self::Signal(value)
    }
}

fn contains_mqtt_topic_meta(v: &str) -> bool {
    v.as_bytes()
        .iter()
        .any(|c| matches!(*c, b'/' | b'+' | b'#'))
}

fn is_valid_hyphenated_uuid(v: &str) -> bool {
    if v.len() != Hyphenated::LENGTH {
        return false;
    }

    for (idx, c) in v.as_bytes().iter().copied().enumerate() {
        match idx {
            8 | 13 | 18 | 23 => {
                if c != b'-' {
                    return false;
                }
            }
            _ => {
                if !c.is_ascii_hexdigit() {
                    return false;
                }
            }
        }
    }

    true
}

fn is_valid_dns_name(name: &str) -> bool {
    if name.len() > MAX_BROKER_HOSTNAME_SIZE {
        return false;
    }

    let mut label_count = 0usize;
    for label in name.split('.') {
        label_count += 1;
        if !is_valid_dns_label(label) {
            return false;
        }
    }

    label_count > 0
}

fn is_valid_dns_label(label: &str) -> bool {
    if label.is_empty() || label.len() > 63 {
        return false;
    }

    let bytes = label.as_bytes();
    let first = bytes[0];
    let last = bytes[bytes.len() - 1];
    if !first.is_ascii_alphanumeric() || !last.is_ascii_alphanumeric() {
        return false;
    }

    for &b in bytes {
        if !(b.is_ascii_alphanumeric() || b == b'-') {
            return false;
        }
    }

    true
}
