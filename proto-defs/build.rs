fn main() -> Result<(), Box<dyn std::error::Error>> {    
    tonic_build::configure()
    .type_attribute("identifier.GetIdRequest", "#[derive(serde::Serialize, serde::Deserialize)]")
    .out_dir("src")
    .compile_protos(&[
        "identifier.proto",
        "epoch.proto",
        "connection.proto",
        "transaction.proto",
        ], 
        &[
            format!("{}/protos", env!("CARGO_MANIFEST_DIR"))
        ])?;
    Ok(())
}