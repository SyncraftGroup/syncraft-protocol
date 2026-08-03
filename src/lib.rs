use std::{
    collections::HashMap,
    io,
    path::{Path, PathBuf},
    time::SystemTime,
};

use serde::{Deserialize, Serialize};
use tokio::{
    fs::{self, File},
    io::AsyncReadExt,
};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum Loader {
    Vanilla,
    Fabric { loader_version: String },
    Forge { loader_version: String },
    NeoForge { loader_version: String },
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct VersionInfo {
    pub name: String,
    pub vanilla_version: String,
    pub loader: Loader,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone)]
pub struct FileState {
    pub size: u64,
    pub hash: blake3::Hash,
    pub modified: SystemTime,
}

impl FileState {
    pub async fn from_path(path: &Path) -> io::Result<Self> {
        let metadata = fs::metadata(path).await?;

        Ok(Self {
            hash: Self::hash(path).await?,
            size: metadata.len(),
            modified: metadata.modified()?,
        })
    }

    pub async fn hash(path: &Path) -> io::Result<blake3::Hash> {
        let mut file = File::open(path).await?;
        let mut hasher = blake3::Hasher::new();
        let mut buf = [0u8; 8192];

        loop {
            let n = file.read(&mut buf).await?;
            if n == 0 {
                break;
            }
            hasher.update(&buf[..n]);
        }

        Ok(hasher.finalize())
    }

    pub async fn matches(&self, path: &Path) -> io::Result<bool> {
        Ok(self.hash == Self::hash(path).await?)
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SyncMode {
    Default,
    Minimal,
    Strict,
    Forbidden,
}

impl Default for SyncMode {
    fn default() -> Self {
        Self::Strict
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SyncFile {
    pub state: Option<FileState>,
    pub mode: SyncMode,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Manifest {
    pub version_hash: String,
    pub version: VersionInfo,
    pub files: HashMap<PathBuf, SyncFile>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateInfo {
    pub message: String,
    pub hash: String,
}
