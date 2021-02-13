use crate::command::build;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;
use tempfile::tempdir;

pub fn run(in_file: PathBuf) -> Result<(), String> {
    let out_dir = tempdir()
        .expect("Could not create temporary file")
        .into_path();

    let intermediate_out_file_path = out_dir.join("intermediate.c");
    build::build(&in_file, &intermediate_out_file_path)?;
    let out_file = out_dir.join("out");
    if cfg!(feature = "backend_c") {
        Command::new("/usr/bin/cc")
            .arg(&intermediate_out_file_path)
            .arg("-o")
            .arg(&out_file)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .expect("Could not spawn compilation process");

        let out = Command::new(out_file)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .expect("Could not spawn run process");

        std::io::stdout()
            .write_all(&out.stdout)
            .expect("Could not write to stdout");

        std::io::stderr()
            .write_all(&out.stderr)
            .expect("Could not write to stderr");
    } else if cfg!(feature = "backend_node") {
        let out = Command::new("node")
            .arg(&intermediate_out_file_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .expect("Could not spawn run process");

        std::io::stdout()
            .write_all(&out.stdout)
            .expect("Could not write to stdout");

        std::io::stderr()
            .write_all(&out.stderr)
            .expect("Could not write to stderr");
    }
    Ok(())
}
