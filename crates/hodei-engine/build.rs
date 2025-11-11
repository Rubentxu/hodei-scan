//! Build script to generate Rust code from Cap'n Proto schemas
//! This is a no-op if capnp tool is not available

fn main() {
    // Only run if capnp tool is available
    if let Ok(output) = std::process::Command::new("capnp").arg("--version").output() {
        if output.status.success() {
            println!("cargo:rerun-if-changed=schemas/");
            
            match capnpc::CompilerCommand::new()
                .src_prefix("schemas")
                .file("schemas/extractor_protocol.capnp")
                .run()
            {
                Ok(_) => println!("Cap'n Proto schemas compiled successfully"),
                Err(e) => println!("Warning: Failed to compile Cap'n Proto schemas: {}", e),
            }
        } else {
            println!("Warning: capnp tool found but returned error");
        }
    } else {
        println!("Info: capnp tool not found, skipping schema compilation");
        println!("Info: Using pre-generated Rust modules from extractor_protocol_capnp.rs");
    }
}
