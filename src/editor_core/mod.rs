use crate::ropey;
use std::collections::HashMap;
use std::fs;
use std::io;
use std::io::{BufReader};
use std::path;
use std::borrow::Cow;

use ropey::Rope;
use ropey::iter::Bytes;

use std::sync::atomic::{AtomicU16, Ordering::SeqCst};

/// Engine Errors
#[derive(Debug)]
pub enum Error {
   InsertError,
   DeleteError,
   FileExists,
   FileError(io::Error),
}

pub type Result<T> = ::std::result::Result<T, Error>;

/// An operation that can be performed on a document.
/// Engine performs these operations on the Document.
pub enum Operation  {
}


pub struct Engine {
   /// List of all open documents.
   docs: Vec<Document>,

   /// A acciciation of panes to indices in docs vector.
   document_map: HashMap<DocID, usize>,
}

#[derive(Debug, Clone)]
pub struct Document {
   /// file of the document
   path: Option<String>,
   /// string representation of the document
   content: ropey::Rope,
   /// unique id
   id: DocID
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct DocID(u16);

impl DocID {
   fn next() -> Self {
      static TOKEN: AtomicU16 = AtomicU16::new(0);

      Self { 0: TOKEN.fetch_add(0, SeqCst) }
   }
}

impl Document {
   /// creates a document that doesnt have a name
   /// but can be written on. I.G. a scratch buffer
   /// If saved a filename will be requested.
   pub fn name_less() -> Result<Self> {
      Ok(Self {
         path: None,
         content: Rope::new(),
         id: DocID::next(),
      })
   }

   pub fn from_path(path: &str) -> Result<Self> {
      let path = path::Path::new(path);
      let id = DocID::next();

      let error_map = |e| { Error::FileError(e) };

      let content = if path.exists() {
         Rope::from_reader(
            BufReader::new(
                  fs::File::open(path).map_err(error_map)?
               )
            ).map_err(error_map)?
      }
      else {
         Rope::new()
      };

      println!("Num Lines: {}", content.len_lines());
      println!("Size: {}", content.len_bytes());

      Ok(
         Self {
            // @TODO: handle error from both canonicalizing and string unwrapping.
            path: Some(path.canonicalize().unwrap().as_path().to_str().unwrap().to_string()),
            content,
            id,
         }
      )
   }

   pub fn id(&self) -> DocID {
      self.id
   }

   pub fn as_str<'a>(&'a self) -> Cow<str> {
      self.content.clone().into()
   }
}


impl Engine {
   pub fn new() -> Self {
      Self {
         docs: Vec::new(),
         document_map: HashMap::new(),
      }
   }

   /// Attempts to open a given file on a pane.
   /// Handles the pane/document association.
   /// pane: The ID of the pane that is opening this file
   /// path: Path of the file attempting to be open.
   pub fn open_document(&mut self, path: &str) -> Result<DocID> {
      let (doc, index) = self.open_file(path)?;

      self.document_map.entry(doc)
         .and_modify(|e| { *e = index } )
         .or_insert(index);

      Ok(doc)
   }

   /// Opens the file and retuns the index in docs.
   fn open_file(&mut self, path: &str) -> Result<(DocID, usize)> {
      // @TODO: Implement
      let index = self.docs.len();
      self.docs.push(Document::from_path(path)?);
      // the unwrap is always ok because of the above line.
      Ok((self.docs.last().unwrap().id(), index as usize))
   }



   /// Executes a given operation on document of pane.
   /// pane: The identifier to know which file is being operated on.
   /// op: The operation being executed. See Operation for more detail.
   pub fn execute_on(&mut self, doc: DocID, op: Operation) -> Result<()> {
      unimplemented!()
   }
}
