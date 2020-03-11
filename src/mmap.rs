use std::{fs::File, io, path::Path, sync::Arc};

use fs2::FileExt;
use memmap::Mmap;

#[derive(Debug, Clone)]
pub struct SharedFileMmap(Arc<FileMmap>);

impl SharedFileMmap {
    #[allow(dead_code)]
    pub fn try_unwrap(this: SharedFileMmap) -> Result<FileMmap, Self> {
        Arc::try_unwrap(this.0).map_err(SharedFileMmap)
    }
}

#[derive(Debug)]
pub struct FileMmap {
    mmap: Mmap,
    fh: File,
}

impl FileMmap {
    pub fn open<P>(path: P) -> io::Result<Self>
    where
        P: AsRef<Path>,
    {
        let fh = File::open(path)?;
        fh.try_lock_exclusive()?;
        // If another process writes to the file, segments of the mmap could be mutated so that
        // for example, mmap[0] a few seconds ago != mmap[0]
        // The file lock above isn't reliable but it's better than nothing.
        // Don't even talk about SIGBUS
        let mmap = unsafe { Mmap::map(&fh) }?;
        Ok(Self { mmap, fh })
    }

    pub fn make_shared(self) -> SharedFileMmap {
        SharedFileMmap(Arc::new(self))
    }
}

impl AsRef<[u8]> for FileMmap {
    fn as_ref(&self) -> &[u8] {
        &self.mmap
    }
}

impl AsRef<[u8]> for SharedFileMmap {
    fn as_ref(&self) -> &[u8] {
        &self.0.mmap
    }
}
