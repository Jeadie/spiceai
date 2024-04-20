use std::env;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
   let proto_file = "./proto/raft.proto";
   let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

   tonic_build::configure()
           .protoc_arg("--experimental_allow_proto3_optional") 
           .build_client(true)
           .build_server(true)
           .use_arc_self(true)
           .file_descriptor_set_path(out_dir.join("store_descriptor.bin"))
           .out_dir("./src")
           .compile(&[proto_file], &["proto"])?;

   Ok(())
}
