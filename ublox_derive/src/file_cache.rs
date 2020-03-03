/// To prevent modification time changing
use std::{
    fs::File,
    io,
    io::{Read, Write},
    path::PathBuf,
};

/// Implement write cache in memory, and update file only if necessary
pub struct FileWriteCache {
    cnt: Vec<u8>,
    path: PathBuf,
}

impl FileWriteCache {
    pub fn new<P: Into<PathBuf>>(p: P) -> FileWriteCache {
        let path = p.into();
        FileWriteCache { cnt: vec![], path }
    }

    pub fn update_file_if_necessary(self) -> Result<(), io::Error> {
        if let Ok(mut f) = File::open(&self.path) {
            let mut cur_cnt = vec![];
            f.read_to_end(&mut cur_cnt)?;
            if cur_cnt == self.cnt {
                return Ok(());
            }
        }
        let mut f = File::create(&self.path)?;
        f.write_all(&self.cnt)?;
        Ok(())
    }

    pub fn replace_content(&mut self, bytes: Vec<u8>) {
        self.cnt = bytes;
    }
}

impl io::Write for FileWriteCache {
    fn write(&mut self, data: &[u8]) -> Result<usize, io::Error> {
        self.cnt.extend_from_slice(data);
        Ok(data.len())
    }
    fn flush(&mut self) -> Result<(), io::Error> {
        Ok(())
    }
}
