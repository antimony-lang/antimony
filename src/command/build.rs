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
use std::path::Path;

pub fn build(in_file: &Path, out_file: &Path) -> Result<(), String> {
    let mut b = builder::Builder::new(in_file.to_path_buf());
    b.build()?;
    b.generate(out_file.to_path_buf())
}
