use std::env::var;

fn main() {
    match var("CARGO_CFG_TARGET_OS").unwrap().as_str() {
        "linux" => linux_build_script(),
        "windows" => windows_build_script(),
        "macos" => macos_build_script(),

        other => {
            panic!("Unsupported OS: {}", other);
        }
    }

    if let Ok(ecal_dir) = var("ECAL_DIR") {
        println!("cargo:rustc-link-search={}/lib", ecal_dir);
    }
}

fn linux_build_script() {
    match var("CARGO_CFG_TARGET_ARCH").unwrap().as_str() {
        "aarch64" => {
            println!("cargo:rustc-link-lib=static=ecal_core_c");
            println!("cargo:rustc-link-lib=static=ecal_core");
            println!("cargo:rustc-link-lib=static=ecal_proto");
            println!("cargo:rustc-link-lib=static=ecal_pb");
            println!("cargo:rustc-link-lib=static=protobuf");
            println!("cargo:rustc-link-lib=static=ecal_utils");
            println!("cargo:rustc-link-lib=static=ecal_CustomTclap");
            println!("cargo:rustc-link-lib=rt");
            println!("cargo:rustc-link-lib=dl");
            println!("cargo:rustc-link-lib=stdc++");
        }

        "x86_64" => {
            println!("cargo:rustc-link-lib=dylib=ecal_core_c");
            println!("cargo:rustc-link-lib=dylib=ecal_core");
            println!("cargo:rustc-link-lib=rt");
            println!("cargo:rustc-link-lib=dl");
            println!("cargo:rustc-link-lib=stdc++");
        }

        other => {
            panic!("Unsupported Linux architecture: {}", other);
        }
    }
}

fn windows_build_script() {
    if var("ECAL_DIR").is_err() {
        println!("cargo:rustc-link-search=C:/eCAL/lib");
    }
    println!("cargo:rustc-link-lib=dylib=ecal_core_c");
    println!("cargo:rustc-link-lib=dylib=ecal_core");
}

fn macos_build_script() {
    println!("cargo:rustc-link-lib=dylib=ecal_core_c");
    println!("cargo:rustc-link-lib=dylib=ecal_core");
}
