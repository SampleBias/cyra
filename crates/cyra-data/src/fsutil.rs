//! Dataset files stay inside a caller-chosen directory. Manifest paths cannot escape it.
use rustix::fs::{self, Mode, OFlags};
use std::fs::File;
use std::io::{self, Read, Write};
use std::os::fd::OwnedFd;
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt};
use std::path::{Component, Path};

pub const MAX_FILE_BYTES: usize = 16 * 1024 * 1024;

const DIRECTORY: OFlags = OFlags::RDONLY
    .union(OFlags::DIRECTORY)
    .union(OFlags::NOFOLLOW)
    .union(OFlags::CLOEXEC);

pub fn create_private_dir(path: &Path) -> io::Result<()> {
    std::fs::DirBuilder::new()
        .mode(0o700)
        .recursive(false)
        .create(path)
}

pub fn write_private(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)?;
    file.write_all(bytes)?;
    file.sync_all()
}

pub fn sync_dir(path: &Path) -> io::Result<()> {
    File::open(path)?.sync_all()
}

pub fn relative_components(relative: &str) -> io::Result<Vec<String>> {
    let path = Path::new(relative);
    let mut names = Vec::new();
    for component in path.components() {
        match component {
            Component::Normal(name) => {
                let name = name.to_str().ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidInput, "path is not UTF-8")
                })?;
                if name.starts_with('.') && !matches!(name, ".zarray" | ".zattrs" | ".zgroup") {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "only Zarr metadata names may start with a dot",
                    ));
                }
                names.push(name.to_owned());
            }
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "dataset paths must be relative and cannot traverse parents",
                ));
            }
        }
    }
    if names.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "dataset path is empty",
        ));
    }
    Ok(names)
}

pub fn read_relative(root: &Path, relative: &str) -> io::Result<Vec<u8>> {
    let names = relative_components(relative)?;
    let mut directory = open_dir(root)?;
    for name in &names[..names.len() - 1] {
        directory =
            fs::openat(&directory, name, DIRECTORY, Mode::empty()).map_err(io::Error::from)?;
    }
    let fd = fs::openat(
        &directory,
        names.last().expect("component list is non-empty"),
        OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
        Mode::empty(),
    )?;
    let stat = fs::fstat(&fd)?;
    if fs::FileType::from_raw_mode(stat.st_mode) != fs::FileType::RegularFile {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "dataset entry must be a regular file",
        ));
    }
    let mut bytes = Vec::new();
    File::from(fd)
        .take((MAX_FILE_BYTES + 1) as u64)
        .read_to_end(&mut bytes)?;
    if bytes.len() > MAX_FILE_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "dataset file exceeds the byte limit",
        ));
    }
    Ok(bytes)
}

pub fn open_dir(path: &Path) -> io::Result<OwnedFd> {
    let absolute = if path.is_absolute() {
        path.to_owned()
    } else {
        std::env::current_dir()?.join(path)
    };
    let mut fd = fs::open("/", DIRECTORY, Mode::empty())?;
    for component in absolute.components() {
        let name = match component {
            Component::RootDir | Component::CurDir => continue,
            Component::Normal(name) => name,
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "parent traversal is forbidden",
                ));
            }
        };
        fd = fs::openat(&fd, name, DIRECTORY, Mode::empty())?;
    }
    Ok(fd)
}

pub fn mode_is_private_dir(path: &Path) -> io::Result<bool> {
    let fd = open_dir(path)?;
    Ok(fs::fstat(&fd)?.st_mode & 0o077 == 0)
}
