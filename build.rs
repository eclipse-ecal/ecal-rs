/********************************************************************************
 * Copyright (c) 2024 Kopernikus Automotive
 * 
 * This program and the accompanying materials are made available under the
 * terms of the Apache License, Version 2.0 which is available at
 * https://www.apache.org/licenses/LICENSE-2.0.
 * 
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS, WITHOUT
 * WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the
 * License for the specific language governing permissions and limitations
 * under the License.
 * 
 * SPDX-License-Identifier: Apache-2.0
 ********************************************************************************/
 
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
