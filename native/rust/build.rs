use std::path::PathBuf;

fn main() {
    let manifest = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").expect("manifest dir"));
    let proto = manifest.join("../proto/gluerift_native.proto");
    let include = manifest.join("../proto");
    let protoc = protoc_bin_vendored::protoc_bin_path().expect("vendored protoc");

    // SAFETY: Cargo executes each build script in its own process. This value
    // is set before prost-build is invoked and cannot race another thread.
    unsafe { std::env::set_var("PROTOC", protoc) };
    prost_build::Config::new()
        .compile_protos(&[proto.clone()], &[include])
        .expect("compile native protobuf schema");
    println!("cargo:rerun-if-changed={}", proto.display());
}
