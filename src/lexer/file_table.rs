use std::path::PathBuf;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct FileId {
    index: usize,
}

impl FileId {
    pub fn path<'a>(&self, table: &'a FileTable) -> &'a PathBuf {
        &self.source_file(table).path
    }

    pub fn contents<'a>(&self, table: &'a FileTable) -> &'a String {
        &self.source_file(table).contents
    }

    fn source_file<'a>(&self, table: &'a FileTable) -> &'a SourceFile {
        &table.files[self.index]
    }
}

#[derive(Debug)]
struct SourceFile {
    path: PathBuf,
    contents: String,
}

#[derive(Debug)]
pub struct FileTable {
    files: Vec<SourceFile>,
}

impl FileTable {
    pub fn new() -> FileTable {
        FileTable { files: Vec::new() }
    }

    pub fn insert(&mut self, path: PathBuf, contents: String) -> FileId {
        let index = self.files.len();
        self.files.push(SourceFile { path, contents });
        FileId { index }
    }
}
