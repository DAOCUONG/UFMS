//! Build script for UFMS.
//!
//! Compiles the VDA 5050 v3.0.0 protobuf schema into Rust types via `prost`.
//! The generated code is written into `OUT_DIR` and re-exported from
//! `src/lib.rs` under the `vda5050::v3` module tree.

use std::io::Result;

fn main() -> Result<()> {
    let proto_root = "VDA5050/proto";

    // Compile every VDA 5050 topic proto plus the shared `common.proto`.
    // The order is irrelevant — prost-build resolves imports via the
    // include path.
    let proto_files = [
        "VDA5050/proto/common.proto",
        "VDA5050/proto/connection.proto",
        "VDA5050/proto/instant_actions.proto",
        "VDA5050/proto/order.proto",
        "VDA5050/proto/state.proto",
        "VDA5050/proto/visualization.proto",
        "VDA5050/proto/factsheet.proto",
        "VDA5050/proto/zone_set.proto",
        "VDA5050/proto/responses.proto",
    ];

    // Rerun the build script if any proto file changes.
    for proto in &proto_files {
        println!("cargo:rerun-if-changed={proto}");
    }
    println!("cargo:rerun-if-changed={proto_root}");

    prost_build::Config::new()
        .compile_protos(&proto_files, &[proto_root])?;

    Ok(())
}