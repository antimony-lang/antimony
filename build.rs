use std::process::Command;

// Example custom build script.
fn main() -> std::io::Result<()> {
    
    println!("cargo:warning=QBE source has changed. Recompiling...");

    let success = Command::new("make")
        .arg("-C")
        .arg("vendor/qbe")
        .spawn()?
        .wait()?
        .success();

    match success {
        true => println!("cargo:warning=Successfully compiled QBE"),
        false => panic!("cargo:warning=QBE compilation failed"),
    };

    Ok(())
}
