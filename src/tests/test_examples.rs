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
use crate::command::run;
use crate::generator::Target;

fn test_directory(dir_in: &str) -> Result<(), String> {
    let dir = std::env::current_dir().unwrap();

    let examples = std::fs::read_dir(dir.join(dir_in)).map_err(|err| err.to_string())?;

    for ex in examples {
        let example = ex.map_err(|err| err.to_string())?;
        let in_file = dir.join(dir_in).join(example.file_name());

        // We don't want to build submodules, since they don't run without a main function
        if in_file.is_dir() {
            continue;
        }

        run::run(Target::JS, in_file)?;
    }
    Ok(())
}

#[test]
fn test_examples() -> Result<(), String> {
    test_directory("examples")?;
    Ok(())
}

#[test]
fn test_testcases() -> Result<(), String> {
    let dir = std::env::current_dir().unwrap();

    let in_file = dir.join("tests/main.sb");
    run::run(Target::JS, in_file)
}
