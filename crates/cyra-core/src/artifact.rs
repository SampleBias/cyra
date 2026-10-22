//! Descriptor-relative artifact access: no symlink traversal or overwrite.
//! The initial supported reference platform is Linux/Unix.
use rustix::fs::{self, AtFlags, Mode, OFlags};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{self, Read, Write};
use std::os::fd::OwnedFd;
use std::path::{Component, Path};
use std::sync::atomic::{AtomicU64, Ordering};

pub const MAX_ARTIFACT_BYTES: usize = 16 * 1024 * 1024;
static COUNTER: AtomicU64 = AtomicU64::new(0);
const DIRECTORY: OFlags = OFlags::RDONLY
    .union(OFlags::DIRECTORY)
    .union(OFlags::NOFOLLOW)
    .union(OFlags::CLOEXEC);

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ArtifactReference {
    pub sha256: String,
    pub size_bytes: u64,
    pub relative_path: String,
}

pub fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

/// cyra-json-v1: recursively sorted keys, compact UTF-8, no trailing newline.
/// Number spelling follows the pinned serde_json; not a claim of RFC 8785 compliance.
pub fn canonical_json(value: &Value) -> io::Result<Vec<u8>> {
    fn sorted(value: &Value) -> Value {
        match value {
            Value::Object(map) => {
                let ordered: std::collections::BTreeMap<_, _> = map.iter().collect();
                Value::Object(
                    ordered
                        .into_iter()
                        .map(|(k, v)| (k.clone(), sorted(v)))
                        .collect(),
                )
            }
            Value::Array(items) => Value::Array(items.iter().map(sorted).collect()),
            _ => value.clone(),
        }
    }
    serde_json::to_vec(&sorted(value)).map_err(io::Error::other)
}

pub struct ArtifactStore {
    directory: OwnedFd,
}
impl ArtifactStore {
    /// Creates missing directories privately; refuses a shared final directory.
    pub fn create(path: &Path) -> io::Result<Self> {
        let absolute = if path.is_absolute() {
            path.to_owned()
        } else {
            std::env::current_dir()?.join(path)
        };
        let mut fd = fs::open("/", DIRECTORY, Mode::empty())?;
        let mut traversed = false;
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
            traversed = true;
            match fs::openat(&fd, name, DIRECTORY, Mode::empty()) {
                Ok(next) => fd = next,
                Err(rustix::io::Errno::NOENT) => {
                    match fs::mkdirat(&fd, name, Mode::from_raw_mode(0o700)) {
                        Ok(()) | Err(rustix::io::Errno::EXIST) => {}
                        Err(error) => return Err(error.into()),
                    }
                    fd = fs::openat(&fd, name, DIRECTORY, Mode::empty())?;
                }
                Err(error) => return Err(error.into()),
            }
        }
        if !traversed || fs::fstat(&fd)?.st_mode & 0o077 != 0 {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "artifact directory must be private (0700)",
            ));
        }
        Ok(Self { directory: fd })
    }

    pub fn stage(&self) -> io::Result<StagedArtifact<'_>> {
        for _ in 0..32 {
            let name = format!(
                ".stage-{}-{}",
                std::process::id(),
                COUNTER.fetch_add(1, Ordering::Relaxed)
            );
            match fs::openat(
                &self.directory,
                &name,
                OFlags::WRONLY | OFlags::CREATE | OFlags::EXCL | OFlags::NOFOLLOW | OFlags::CLOEXEC,
                Mode::from_raw_mode(0o600),
            ) {
                Ok(fd) => {
                    return Ok(StagedArtifact {
                        store: self,
                        name,
                        file: File::from(fd),
                        hasher: Sha256::new(),
                        size: 0,
                    });
                }
                Err(rustix::io::Errno::EXIST) => continue,
                Err(error) => return Err(error.into()),
            }
        }
        Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "temporary artifact collision limit",
        ))
    }

    pub fn put_json(&self, value: &Value) -> io::Result<ArtifactReference> {
        let bytes = canonical_json(value)?;
        let mut stage = self.stage()?;
        stage.write_all(&bytes)?;
        stage.commit()
    }

    pub fn read_relative(&self, path: &Path) -> io::Result<Vec<u8>> {
        let components: Vec<_> = path.components().collect();
        if components.is_empty()
            || components
                .iter()
                .any(|c| !matches!(c, Component::Normal(_)))
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "only relative normal path components are permitted",
            ));
        }
        let mut parent = rustix::io::dup(&self.directory)?;
        for component in &components[..components.len() - 1] {
            parent = fs::openat(&parent, component.as_os_str(), DIRECTORY, Mode::empty())?;
        }
        let fd = fs::openat(
            &parent,
            components[components.len() - 1].as_os_str(),
            OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::CLOEXEC | OFlags::NONBLOCK,
            Mode::empty(),
        )?;
        let stat = fs::fstat(&fd)?;
        if fs::FileType::from_raw_mode(stat.st_mode) != fs::FileType::RegularFile {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "artifact must be a regular file",
            ));
        }
        let mut bytes = Vec::new();
        File::from(fd)
            .take((MAX_ARTIFACT_BYTES + 1) as u64)
            .read_to_end(&mut bytes)?;
        if bytes.len() > MAX_ARTIFACT_BYTES {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "artifact exceeds byte limit",
            ));
        }
        Ok(bytes)
    }

    pub fn read_verified(&self, reference: &ArtifactReference) -> io::Result<Vec<u8>> {
        if reference.sha256.len() != 64
            || !reference
                .sha256
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            || reference.relative_path != format!("sha256-{}", reference.sha256)
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "invalid content-addressed reference",
            ));
        }
        let bytes = self.read_relative(Path::new(&reference.relative_path))?;
        if bytes.len() as u64 != reference.size_bytes || sha256(&bytes) != reference.sha256 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "artifact integrity check failed",
            ));
        }
        Ok(bytes)
    }
}

pub struct StagedArtifact<'a> {
    store: &'a ArtifactStore,
    name: String,
    file: File,
    hasher: Sha256,
    size: u64,
}
impl Write for StagedArtifact<'_> {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        if buffer.len() as u64 > MAX_ARTIFACT_BYTES as u64 - self.size {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "artifact exceeds byte limit",
            ));
        }
        let written = self.file.write(buffer)?;
        self.hasher.update(&buffer[..written]);
        self.size += written as u64;
        Ok(written)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.file.flush()
    }
}
impl StagedArtifact<'_> {
    /// Atomic link publication refuses an existing destination, including a symlink.
    pub fn commit(self) -> io::Result<ArtifactReference> {
        self.file.sync_all()?;
        let hash = format!("{:x}", self.hasher.clone().finalize());
        let path = format!("sha256-{hash}");
        fs::linkat(
            &self.store.directory,
            &self.name,
            &self.store.directory,
            &path,
            AtFlags::empty(),
        )?;
        fs::fsync(&self.store.directory)?;
        Ok(ArtifactReference {
            sha256: hash,
            size_bytes: self.size,
            relative_path: path,
        })
    }
}
impl Drop for StagedArtifact<'_> {
    fn drop(&mut self) {
        let _ = fs::unlinkat(&self.store.directory, &self.name, AtFlags::empty());
    }
}
