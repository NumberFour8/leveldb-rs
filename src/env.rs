//! An `env` is an abstraction layer that allows the database to run both on different platforms as
//! well as persisting data on disk or in memory.

use error::{self, Result};

use std::io::{self, Read, Write, Seek};
use std::path::Path;
use std::sync::{Arc, Mutex};

/// RandomAccessFile wraps a type implementing read and seek to enable atomic random reads
#[derive(Clone)]
pub struct RandomAccessFile<F: Read + Seek> {
    f: Arc<Mutex<F>>,
}

impl<F: Read + Seek> RandomAccessFile<F> {
    pub fn new(f: F) -> RandomAccessFile<F> {
        RandomAccessFile { f: Arc::new(Mutex::new(f)) }
    }

    pub fn read_at(&self, off: usize, len: usize) -> Result<Vec<u8>> {
        let mut f = try!(error::from_lock_result(self.f.lock()));
        try!(error::from_io_result(f.seek(io::SeekFrom::Start(off as u64))));

        let mut buf = Vec::new();
        buf.resize(len, 0);
        error::from_io_result(f.read_exact(&mut buf)).map(|_| buf)
    }
}

pub trait Env {
    type SequentialReader: Read;
    type RandomReader: Read + Seek;
    type Writer: Write;
    type FileLock;

    fn open_sequential_file(&self, &Path) -> Result<Self::SequentialReader>;
    fn open_random_access_file(&self, &Path) -> Result<RandomAccessFile<Self::RandomReader>>;
    fn open_writable_file(&self, &Path) -> Result<Self::Writer>;
    fn open_appendable_file(&self, &Path) -> Result<Self::Writer>;

    fn exists(&self, &Path) -> Result<bool>;
    fn children(&self, &Path) -> Result<Vec<String>>;
    fn size_of(&self, &Path) -> Result<usize>;

    fn delete(&self, &Path) -> Result<()>;
    fn mkdir(&self, &Path) -> Result<()>;
    fn rmdir(&self, &Path) -> Result<()>;
    fn rename(&self, &Path, &Path) -> Result<()>;

    fn lock(&self, &Path) -> Result<Self::FileLock>;
    fn unlock(&self, l: Self::FileLock);

    fn new_logger(&self, &Path) -> Result<Logger>;

    fn micros(&self) -> u64;
    fn sleep_for(&self, micros: u32);
}

pub struct Logger {
    dst: Box<Write>,
}

impl Logger {
    pub fn new(w: Box<Write>) -> Logger {
        Logger { dst: w }
    }

    pub fn log(&mut self, message: &String) {
        let _ = self.dst.write(message.as_bytes());
        let _ = self.dst.write("\n".as_bytes());
    }
}
