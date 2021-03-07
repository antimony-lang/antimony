use crate::builder;
use crate::generator;
use std::fs::File;
use std::io::Read;
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
use std::io::Write;
use std::path::Path;

pub fn build(target: generator::Target, in_file: &Path, out_file: &Path) -> Result<(), String> {
    let mut buf = Box::new(Vec::new());
    File::open(out_file)
        .expect("Could not open file for writing")
        .read_to_end(&mut buf)
        .expect("Could not read from file");
    build_to_buffer(target, in_file, &mut buf)
}

pub fn build_to_buffer(
    target: generator::Target,
    in_file: &Path,
    buf: &mut Box<impl Write>,
) -> Result<(), String> {
    let mut b = builder::Builder::new(in_file.to_path_buf());
    b.build()?;
    b.generate(target, buf)
}
