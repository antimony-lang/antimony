use crate::check::Module as CheckedModule;
use crate::lexer;
use crate::parser;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

pub fn check(in_file: PathBuf) -> Result<(), String> {
    let mut file = File::open(&in_file).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .map_err(|err| err.to_string())?;

    let mut table = lexer::FileTable::new();
    let file = table.insert(in_file, contents);

    let tokens = lexer::tokenize(file, &table).map_err(|err| err.format(&table))?;
    let module = parser::parse(tokens).map_err(|err| err.format(&table))?;
    println!("Parsed: {:#?}", module);
    let checked_module = CheckedModule::from_ast(module).map_err(|err| err.format(&table))?;
    println!("Checked: {:#?}", checked_module);
    Ok(())
}
