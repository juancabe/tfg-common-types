use postcard::experimental::max_size::MaxSize;
use postcard_bindgen::PostcardBindings;
pub const PK_SIZE: usize = 32;
pub const MAX_SERVER_URL_SIZE: usize = 128;
pub const MAX_SSID_SIZE: usize = 128;
pub const MAX_WIFI_PASS_SIZE: usize = 128;

#[derive(
    Debug,
    defmt::Format,
    serde::Deserialize,
    serde::Serialize,
    PostcardBindings,
    MaxSize,
    Clone,
    Copy,
)]
pub struct WifiComm {
    #[serde(with = "serde_big_array::BigArray")]
    pub ssid: [u8; MAX_SSID_SIZE],
    #[serde(with = "serde_big_array::BigArray")]
    pub pass: [u8; MAX_WIFI_PASS_SIZE],
    #[serde(with = "serde_big_array::BigArray")]
    pub server_url: [u8; MAX_SERVER_URL_SIZE],
}

#[derive(
    Debug,
    defmt::Format,
    serde::Deserialize,
    serde::Serialize,
    PostcardBindings,
    MaxSize,
    Clone,
    Copy,
)]
pub struct LoraComm {
    pub forwarder_public_key: [u8; PK_SIZE],
    pub forwarder_lora_addr: u16,
}

#[derive(
    Debug,
    defmt::Format,
    serde::Deserialize,
    serde::Serialize,
    PostcardBindings,
    MaxSize,
    Clone,
    Copy,
)]
#[allow(clippy::large_enum_variant)]
pub enum Communication {
    Lora(LoraComm),
    Wifi(WifiComm),
}

#[derive(
    Debug,
    defmt::Format,
    serde::Deserialize,
    serde::Serialize,
    PostcardBindings,
    MaxSize,
    Clone,
    Copy,
)]
pub struct Config {
    pub server_public_key: [u8; PK_SIZE],
    pub private_key: [u8; PK_SIZE],
    pub self_uuid: u128,
    pub communication: Communication,
    pub forward_lora: Option<bool>,
}

impl Config {
    pub fn validate(self) -> Result<Self, ()> {
        // TODO: Do
        Ok(self)
    }

    pub fn needs_signal_discovery(&self) -> bool {
        matches!(self.communication, Communication::Lora(_))
    }
}
