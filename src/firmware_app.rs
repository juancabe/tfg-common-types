use postcard_bindgen::PostcardBindings;

#[derive(Debug, defmt::Format, serde::Deserialize, serde::Serialize, PostcardBindings)]
pub enum CommMethod {
    // TODO: placeholder for forwarder's pub key
    Lora([u8; 10]),
    // SERVER's URL
    Internet([u8; 32]),
}

#[derive(Debug, defmt::Format, serde::Deserialize, serde::Serialize, PostcardBindings)]
pub struct Config {
    // TODO: Create and define the crypto module
    pub key_pair: [u8; 20],
    pub comm_method: CommMethod,
    pub forward_lora: Option<bool>,
    pub user_id: [u8; 32],
}
