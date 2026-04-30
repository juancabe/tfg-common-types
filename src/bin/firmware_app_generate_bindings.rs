#[cfg(feature = "std")]
fn main() {
    use common_types::firmware_app::{
        BrokerIp, Communication, Config, LoraComm, SignalProtocolStatus, WifiComm,
    };
    use postcard_bindgen::{
        generate_bindings,
        javascript::{build_package, GenerationSettings},
        PackageInfo,
    };
    use std::path::Path;

    // Needed because of postcard_bindgen
    // BUG: When no root types are found, Option::None::unwrap is called
    // TODO: Report the bug
    #[derive(serde::Serialize, postcard_bindgen::PostcardBindings)]
    struct RootDummy {
        a: u8,
    }

    let out_dir = Path::new("../app/common-types-bindings");

    build_package(
        out_dir,
        PackageInfo {
            name: "firmware".into(),
            version: "0.1.0".try_into().unwrap(),
        },
        GenerationSettings::enable_all(),
        generate_bindings!(
            RootDummy,
            Config,
            Communication,
            BrokerIp,
            WifiComm,
            LoraComm,
            SignalProtocolStatus
        ),
    )
    .unwrap();
    println!(
        "Bindings generated at: {:?}",
        out_dir.canonicalize().unwrap_or(out_dir.into())
    );
}

#[cfg(not(feature = "std"))]
fn main() {
    panic!("This binary requires the 'std' feature to run.");
}
