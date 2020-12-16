use std::fs;
/**
 * Copyright 2020 Garrit Franke
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      https://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */
use std::io::Error;
use std::process::Command;
#[test]
#[cfg(feature = "backend_node")]
fn test_examples() -> Result<(), Error> {
    let dir = std::env::current_dir().unwrap();

    let examples = std::fs::read_dir(dir.join("examples"))?;

    let _ = fs::create_dir("examples_out");

    for ex in examples {
        let example = ex?;
        let in_file = dir.join("examples").join(example.file_name());
        let out_file = dir.join("examples_out").join(
            example
                .file_name()
                .into_string()
                .unwrap()
                .replace(".sb", ".js"),
        );
        let success = Command::new("cargo")
            .arg("run")
            .arg("build")
            .arg(&in_file)
            .arg("-o")
            .arg(&out_file)
            .spawn()?
            .wait()?
            .success();
        assert_eq!(success, true, "{:?}", &in_file);

        let node_installed = Command::new("node").arg("-v").spawn()?.wait()?.success();
        if node_installed {
            let execution = Command::new("node")
                .arg(out_file)
                .spawn()?
                .wait()?
                .success();
            assert_eq!(execution, true, "{:?}", &in_file)
        }
    }
    Ok(())
}
