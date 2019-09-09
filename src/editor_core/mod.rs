use crate::ropey;
use std::borrow::Cow;
use std::collections::HashMap;
use std::fs;
use std::io;
use std::io::BufReader;
use std::path;
use std::rc::Rc;

use ropey::iter::Bytes;
use ropey::Rope;
use crate::config;

use std::sync::atomic::{AtomicU16, Ordering::SeqCst};

/// Engine Errors
#[derive(Debug)]
pub enum Error {
    InsertError,
    DeleteError,
    FileExists,
    InvalidDocID,
    MissingPath,
    FileError(io::Error),
}

pub type Result<T> = ::std::result::Result<T, Error>;

/// describes the differnt operations that can be performed
#[derive(Debug, Clone)]
enum OperationKind {
    /// Insert a single character at .0, .1
    Insert(usize, u32, u32, char),
    /// Delete character at .0, .1
    Delete(usize, u32, u32),
    /// Paste the content of the clipboard at .0, .1
    Paste(usize, u32, u32, String),
    /// Writes buffer to disk
    WriteFile,
    /// Closes the file
    CloseFile,
    /// Invalid operation
    Invalid,

    // future operations
    CopySelection,
    DeleteSelection,
}

/// An operation that is being performed on the given file.
#[derive(Debug, Clone)]
pub struct Operation {
    doc: DocID,
    kind: OperationKind,
}

impl Operation {
    pub fn write_file(doc: DocID) -> Self {
        Self {
            doc,
            kind: OperationKind::WriteFile,
        }
    }

    pub fn close_file(doc: DocID) -> Self {
        Self {
            doc,
            kind: OperationKind::CloseFile,
        }
    }

    pub fn insert(doc: DocID, start_index: usize, x: u32, y: u32, ch: char) -> Self {
        Self {
            doc,
            kind: OperationKind::Insert(start_index, x, y, ch),
        }
    }

    pub fn delete(doc: DocID, start_index: usize, x: u32, y: u32, ch: char) -> Self {
        Self {
            doc,
            kind: OperationKind::Delete(start_index, x, y),
        }
    }

    pub fn paste(doc: DocID, start_index: usize, x: u32, y: u32, data: &str) -> Self {
        Self {
            doc,
            kind: OperationKind::Paste(start_index, x, y, data.to_owned()),
        }
    }
}

pub struct Engine {
    /// List of all open documents.
    docs: Vec<Document>,

    /// A acciciation of panes to indices in docs vector.
    document_map: HashMap<DocID, usize>,
    config: Rc<config::Config>
}

#[derive(Debug, Clone)]
pub struct Document {
    /// file of the document
    path: Option<String>,
    /// string representation of the document
    content: ropey::Rope,
    /// unique id
    id: DocID,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct DocID(u16);

impl DocID {
    fn next() -> Self {
        static TOKEN: AtomicU16 = AtomicU16::new(0);

        Self {
            0: TOKEN.fetch_add(0, SeqCst),
        }
    }
}

impl Document {
    /// creates a document that doesnt have a name
    /// but can be written on. I.G. a scratch buffer
    /// If saved a filename will be requested.
    pub fn empty(path: Option<&str>) -> Result<Self> {
        Ok(Self {
            path: {
                // @NOTE: maybe this path should be checked?
                match path {
                    Some(s) => Some(s.to_string()),
                    None => None,
                }
            },
            content: Rope::new(),
            id: DocID::next(),
        })
    }

    pub fn from_path(path: &str) -> Result<Self> {
        let path = path::Path::new(path);
        let id = DocID::next();

        let error_map = |e| Error::FileError(e);

        let content = if path.exists() {
            Rope::from_reader(BufReader::new(fs::File::open(path).map_err(error_map)?))
                .map_err(error_map)?
        } else {
            Rope::new()
        };

        println!("Num Lines: {}", content.len_lines());
        println!("Size: {}", content.len_bytes());

        Ok(Self {
            // @TODO: handle error from both canonicalizing and string unwrapping.
            path: Some(
                path.canonicalize()
                    .unwrap()
                    .as_path()
                    .to_str()
                    .unwrap()
                    .to_string(),
            ),
            content,
            id,
        })
    }

    pub fn id(&self) -> DocID {
        self.id
    }

    pub fn as_str<'a>(&'a self) -> Cow<str> {
        self.content.clone().into()
    }

    pub fn line_slice(&self, start: usize, end: usize) -> Vec<ropey::RopeSlice> {
        let first = self.content.lines().skip(start);
        first.take(end).collect()
    }

    pub fn write(&self) -> Result<()> {
        // this will error if the write inself fails or if there isnt a path associated with the
        // document.
        Ok(())
    }

    pub fn insert(&mut self, index: u64, ch: char) -> Result<()> {
        println!("Character '{}' inserted at {}", ch, index);
        self.content.insert_char(index as usize, ch);
        Ok(())
    }

    pub fn delete(&mut self, index: u64) -> Result<()> {
        println!("Deleting character at index {}", index);
        Ok(())
    }

    pub fn paste(&mut self, index: u64, data: &str) -> Result<()> {
        println!("Pasting {} at index {}", data, index);
        Ok(())
    }

    pub fn cursor_index(&self, first_line: usize, mut x: u32, y: u32, tab_characters: u32) -> u64 {
        let first_line = first_line + y as usize;
        let line_index = self.content.line_to_char(first_line);
        let line_slice = self.content.line(line_index);
        
        // simple solution to tab handling
        for c in line_slice.chars() {
            if c == '\t' {
                x -= tab_characters - 1; 
            }
        }
        
        line_index as u64 + x as u64
    }
}

impl Engine {
    pub fn new(config: Rc<config::Config>) -> Self {
        Self {
            docs: Vec::new(),
            document_map: HashMap::new(),
            config
        }
    }

    /// Attempts to open a given file on a pane.
    /// Handles the pane/document association.
    /// pane: The ID of the pane that is opening this file
    /// path: Path of the file attempting to be open.
    pub fn open_document(&mut self, path: &str) -> Result<DocID> {
        let doc = self.open_file(path)?;
        self.register_document(&doc)
    }

    pub fn register_document(&mut self, document: &Document) -> Result<DocID> {
        // This is a copy of the entire document. This isn't good.
        let id = document.id();
        self.docs.push(document.clone());
        let index = self.docs.len() - 1;

        self.document_map
            .entry(id)
            .and_modify(|e| *e = index)
            .or_insert(index);

        Ok(document.id())
    }

    /// Opens the file and retuns the index in docs.
    fn open_file(&mut self, path: &str) -> Result<Document> {
        Ok(Document::from_path(path)?)
    }

    pub fn close_file(&mut self, doc: DocID) -> Result<()> {
        unimplemented!();
    }

    pub fn get_document(&self, doc: DocID) -> Option<&Document> {
        match self.document_map.get(&doc) {
            Some(e) => Some(&self.docs[*e]),
            _ => None,
        }
    }

    pub fn get_mut_document(&mut self, doc: DocID) -> Option<&mut Document> {
        match self.document_map.get(&doc) {
            Some(e) => Some(&mut self.docs[*e]),
            _ => None,
        }
    }

    pub fn create_empty_document(&mut self) -> Result<DocID> {
        let document = Document::empty(None)?;
        self.register_document(&document)?;
        Ok(document.id())
    }

    /// Executes a given operation on document of pane.
    /// pane: The identifier to know which file is being operated on.
    /// op: The operation being executed. See Operation for more detail.
    pub fn execute_on(&mut self, op: Operation) -> Result<()> {
        let tab_size = self.config.as_ref().tabs.tab_width as u32;
        let document = self.get_mut_document(op.doc).ok_or(Error::InvalidDocID)?;

        println!("Operation {:?}", op);

        match op.kind {
            OperationKind::Insert(start_index, x, y, ch) => {
                let index = document.cursor_index(start_index, x, y, tab_size);
                document.insert(index, ch)?
            }
            OperationKind::Delete(start_index, x, y) => {
                let index = document.cursor_index(start_index, x, y, tab_size);
                document.delete(index)?
            }
            OperationKind::Paste(start_index, x, y, data) => {
                let index = document.cursor_index(start_index, x, y, tab_size);
                document.paste(index, data.as_str())?
            }
            OperationKind::WriteFile => document.write()?,
            OperationKind::CloseFile => self.close_file(op.doc)?,
            OperationKind::Invalid => panic!("Attempting to execute invalid operatiion"),
            _ => unimplemented!(),
        }

        Ok(())
    }
}
