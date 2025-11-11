fn main() {
    // Try to compile Cap'n Proto schemas, but don't fail if compiler is not available
    match capnpc::CompilerCommand::new()
        .file("schema/ir.capnp")
        .output_path("src/generated")
        .run()
    {
        Ok(_) => println!("Cap'n Proto schema compiled successfully"),
        Err(e) => {
            println!("Warning: Cap'n Proto schema compilation failed: {}", e);
            println!("This is expected if the capnp compiler is not installed.");
            println!("The capnp_serialization module will use stub implementations.");
        }
    }
}
