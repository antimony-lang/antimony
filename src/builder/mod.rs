use crate::generator;
use crate::lexer;
use crate::parser;
use crate::Lib;
use crate::PathBuf;
use parser::node_type::Module;
use std::env;
/**
 * Copyright 2021 Garrit Franke
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
use std::fs::File;
use std::io::Read;
use std::io::Write;

pub struct Builder {
    in_file: PathBuf,
    modules: Vec<Module>,
}

impl Builder {
    pub fn new(entrypoint: PathBuf) -> Self {
        Self {
            in_file: entrypoint,
            modules: Vec::new(),
        }
    }

    fn get_base_path(&self) -> Result<PathBuf, String> {
        Ok(self
            .in_file
            .parent()
            .ok_or("File does not have a parent")?
            .to_path_buf())
    }

    pub fn build(&mut self) -> Result<(), String> {
        let in_file = self.in_file.clone();
        // Resolve path deltas between working directory and entrypoint
        let base_directory = self.get_base_path()?;
        if let Ok(resolved_delta) = in_file.strip_prefix(&base_directory) {
            // TODO: This error could probably be handled better
            let _ = env::set_current_dir(base_directory);
            self.in_file = resolved_delta.to_path_buf();
        }
        self.build_module(self.in_file.clone())?;

        // Append standard library
        self.build_stdlib();
        Ok(())
    }

    fn build_module(&mut self, file_path: PathBuf) -> Result<Module, String> {
        // TODO: This method can probably cleaned up quite a bit

        // In case the module is a directory, we have to append the filename of the entrypoint
        let resolved_file_path = if file_path.is_dir() {
            file_path.join("module.sb")
        } else {
            file_path
        };
        let mut file = File::open(&resolved_file_path)
            .map_err(|_| format!("Could not open file: {}", resolved_file_path.display()))?;
        let mut contents = String::new();

        file.read_to_string(&mut contents)
            .expect("Could not read file");
        let tokens = lexer::tokenize(&contents);
        let module = parser::parse(
            tokens,
            Some(contents),
            resolved_file_path.display().to_string(),
        )?;
        for import in &module.imports {
            // Build module relative to the current file
            let mut import_path = resolved_file_path
                .parent()
                .unwrap()
                .join(PathBuf::from(import));

            if import_path.is_dir() {
                import_path = import_path.join("module.sb");
            } else if !import_path.ends_with(".sb") {
                import_path.set_extension("sb");
            }

            self.build_module(import_path)?;
        }
        self.modules.push(module.clone());
        Ok(module)
    }

    pub(crate) fn generate(&mut self, out_file: PathBuf) -> Result<(), String> {
        let mut mod_iter = self.modules.iter();

        // TODO: We shouldn't clone here
        let mut condensed = mod_iter.next().ok_or("No module specified")?.clone();
        for module in mod_iter {
            condensed.merge_with(module.clone());
        }
        let output = generator::generate(condensed);
        let mut file = std::fs::File::create(out_file).expect("create failed");
        file.write_all(output.as_bytes()).expect("write failed");
        file.flush().map_err(|_| "Could not flush file".into())
    }

    fn build_stdlib(&mut self) {
        let assets = Lib::iter();

        for file in assets {
            let stdlib_raw =
                Lib::get(&file).expect("Standard library not found. This should not occur.");
            let stblib_str =
                std::str::from_utf8(&stdlib_raw).expect("Could not interpret standard library.");
            let stdlib_tokens = lexer::tokenize(&stblib_str);
            let module = parser::parse(stdlib_tokens, Some(stblib_str.into()), file.to_string())
                .expect("Could not parse stdlib");
            self.modules.push(module);
        }
    }
}
