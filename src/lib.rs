#![no_std]

pub mod crypto;
pub mod firmware_app;

pub mod common_sizes {
    pub const MAX_USERNAME_LEN: usize = 32;
    pub const MAX_PASSWORD_LEN: usize = 32;
    pub const MAX_PRODUCER_NAME_LEN: usize = 16;
    pub const MAX_GROUP_NAME_LEN: usize = 32;
}

pub mod common_template_uuids {
    pub const RANDOMFLOAT_TEMPLATE_UUID: &str = "8f1ab69e-0fcb-4e18-9d3f-42f71d2f0001";
    pub const SCD41_TEMPLATE_UUID: &str = "992be127-edf8-443e-a301-cd17121672d7";
}

#[cfg(test)]
mod tests {}
