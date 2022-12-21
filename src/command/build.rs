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
use crate::builder;
use crate::generator::Target;
use std::fs::File;
use std::io::stdout;
use std::io::Write;
use std::path::Path;

pub fn build(target: &Target, in_file: &Path, out_file: &Path) -> Result<(), String> {
    let mut buf = Box::<Vec<u8>>::default();
    build_to_buffer(target, in_file, &mut buf)?;

    if out_file.to_str() == Some("-") {
        stdout()
            .write_all(&buf)
            .map_err(|e| format!("Could not write to stdout: {}", e))
    } else {
        File::create(out_file)
            .map_err(|e| format!("Could not create output file: {}", e))?
            .write_all(&buf)
            .map_err(|e| format!("Could not write to file: {}", e))
    }
}

pub fn build_to_buffer(
    target: &Target,
    in_file: &Path,
    buf: &mut Box<impl Write>,
) -> Result<(), String> {
    let mut b = builder::Builder::new(in_file.to_path_buf());
    b.build(target)?;
    b.generate(target, buf)
}
