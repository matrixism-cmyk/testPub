// 가상 위치(로컬/압축/원격) 추상화 + 탐색 로직
use crate::model::Entry;
use crate::state::default_start_dir;
use crate::vfs_ftp::FtpSession;
use crate::vfs_zip;
use std::io;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// FTP 연결 공유 핸들
pub type FtpHandle = Arc<Mutex<FtpSession>>;

/// 패널이 가리키는 위치
#[derive(Clone)]
pub enum Location {
    Local(PathBuf),
    Archive {
        archive: PathBuf,
        inner: String,
    },
    Remote {
        conn: FtpHandle,
        host: String,
        path: String,
    },
}

impl Location {
    /// 현재 위치의 항목 목록 (".." 포함)
    pub fn list(&self) -> io::Result<Vec<Entry>> {
        match self {
            Location::Local(p) => crate::model::read_local_dir(p),
            Location::Archive { archive, inner } => vfs_zip::list(archive, inner),
            Location::Remote { conn, path, .. } => conn.lock().unwrap().list(path),
        }
    }

    /// 경로 라벨 표시 문자열
    pub fn display(&self) -> String {
        match self {
            Location::Local(p) => p.display().to_string(),
            Location::Archive { archive, inner } if inner.is_empty() => {
                format!("[zip] {}", archive.display())
            }
            Location::Archive { archive, inner } => {
                format!("[zip] {}\\{}", archive.display(), inner)
            }
            Location::Remote { host, path, .. } => format!("ftp://{}{}", host, path),
        }
    }

    /// 항목으로 진입한 새 위치. None 이면 진입 불가(일반 파일).
    pub fn enter(&self, entry: &Entry) -> Option<Location> {
        if entry.is_parent {
            return self.parent();
        }
        match self {
            Location::Local(p) => {
                if entry.is_dir {
                    Some(Location::Local(p.join(&entry.name)))
                } else if is_zip(&entry.name) {
                    Some(Location::Archive {
                        archive: p.join(&entry.name),
                        inner: String::new(),
                    })
                } else {
                    None
                }
            }
            Location::Archive { archive, inner } if entry.is_dir => Some(Location::Archive {
                archive: archive.clone(),
                inner: join_inner(inner, &entry.name),
            }),
            Location::Archive { .. } => None,
            Location::Remote {
                conn,
                host,
                path,
            } if entry.is_dir => Some(Location::Remote {
                conn: conn.clone(),
                host: host.clone(),
                path: join_remote(path, &entry.name),
            }),
            Location::Remote { .. } => None,
        }
    }

    /// 상위 위치. 아카이브/원격 루트에서는 로컬 홈으로 빠져나온다.
    pub fn parent(&self) -> Option<Location> {
        match self {
            Location::Local(p) => p.parent().map(|pp| Location::Local(pp.to_path_buf())),
            Location::Archive { archive, inner } if inner.is_empty() => {
                archive.parent().map(|pp| Location::Local(pp.to_path_buf()))
            }
            Location::Archive { archive, inner } => Some(Location::Archive {
                archive: archive.clone(),
                inner: parent_inner(inner),
            }),
            Location::Remote { path, .. } if path == "/" || path.is_empty() => {
                Some(Location::Local(default_start_dir()))
            }
            Location::Remote { conn, host, path } => Some(Location::Remote {
                conn: conn.clone(),
                host: host.clone(),
                path: parent_remote(path),
            }),
        }
    }

    /// 로컬 디렉터리면 그 경로
    pub fn as_local(&self) -> Option<&PathBuf> {
        match self {
            Location::Local(p) => Some(p),
            _ => None,
        }
    }

    /// 아카이브 위치면 (아카이브 경로, inner)
    pub fn as_archive(&self) -> Option<(&PathBuf, &str)> {
        match self {
            Location::Archive { archive, inner } => Some((archive, inner)),
            _ => None,
        }
    }

    /// 원격 위치면 (연결 핸들, 경로)
    pub fn as_remote(&self) -> Option<(&FtpHandle, &str)> {
        match self {
            Location::Remote { conn, path, .. } => Some((conn, path)),
            _ => None,
        }
    }
}

fn join_remote(path: &str, name: &str) -> String {
    if path.ends_with('/') {
        format!("{}{}", path, name)
    } else {
        format!("{}/{}", path, name)
    }
}

fn parent_remote(path: &str) -> String {
    let trimmed = path.trim_end_matches('/');
    match trimmed.rfind('/') {
        Some(0) | None => "/".to_string(),
        Some(i) => trimmed[..i].to_string(),
    }
}

fn is_zip(name: &str) -> bool {
    name.to_lowercase().ends_with(".zip")
}

fn join_inner(inner: &str, name: &str) -> String {
    if inner.is_empty() {
        name.to_string()
    } else {
        format!("{}/{}", inner, name)
    }
}

fn parent_inner(inner: &str) -> String {
    match inner.rfind('/') {
        Some(i) => inner[..i].to_string(),
        None => String::new(),
    }
}
