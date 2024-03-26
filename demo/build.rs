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
 
use std::env;
use std::path::PathBuf;

fn main() {
    let crate_root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let pwd = env::current_dir().unwrap();
    assert_eq!(crate_root, pwd);

    let mut config = prost_build::Config::new();

    // Automatically derive ecal message for these types.
    config.type_attribute(".", "#[derive(ecal::Message)]");
    // This is influencing how the type names are derived and
    // ultimately displayed in eCAL mon.
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
