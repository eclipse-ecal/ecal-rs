use std::env;
use std::path::PathBuf;

fn main() {
    let crate_root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let pwd = env::current_dir().unwrap();
    assert_eq!(crate_root, pwd);

    let mut config = prost_build::Config::new();

    // Automatically derive ecal message for these types.
    config.type_attribute(".", "#[derive(ecal::Message)]");
    config.type_attribute(".", "#[type_prefix = \"kpns_msgs.\"]");

    // Compile proto messages
    eprintln!("Compiling protobuf messages with prost");
    config
        .out_dir("src/")
        .compile_protos(
            &[format!("{}/Ping.proto", crate_root.to_str().unwrap())],
            &[crate_root.to_str().unwrap().to_owned()],
        )
        .expect("prost generation");
}
