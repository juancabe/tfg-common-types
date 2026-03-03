use postcard_bindgen::PostcardBindings;
const KEY_SIZE: usize = 32;
const PUB_KEY_SIZE: usize = KEY_SIZE;
const PRIV_KEY_SIZE: usize = KEY_SIZE;

#[derive(Debug, defmt::Format, serde::Deserialize, serde::Serialize, PostcardBindings)]
pub enum CommMethod {
    // TODO: placeholder for forwarder's pub key
    Lora([u8; PUB_KEY_SIZE]),
    // SERVER's URL
    Internet([u8; 32]),
}

#[derive(Debug, defmt::Format, serde::Deserialize, serde::Serialize, PostcardBindings)]
pub struct Config {
    // TODO: Create and define the crypto module
    pub secret: [u8; PRIV_KEY_SIZE],
    pub comm_method: CommMethod,
    pub forward_lora: Option<bool>,
    pub user_id: [u8; PRIV_KEY_SIZE],
}
