// 파일 엔트리 모델 + 디렉터리 읽기 + 정렬 + 표시 포맷
use crate::state::{Sort, SortKey, SortOrder};
use std::path::Path;
use std::time::SystemTime;

/// 패널에 표시되는 한 줄(파일 또는 디렉터리)
#[derive(Clone, Debug)]
pub struct Entry {
    pub name: String,
    pub is_dir: bool,
    pub is_parent: bool, // ".." 상위 이동 항목 여부
    pub size: u64,
    pub modified: Option<SystemTime>,
}

impl Entry {
    /// 상위 디렉터리(..) 항목
    pub fn parent() -> Entry {
        Entry {
            name: "..".to_string(),
            is_dir: true,
            is_parent: true,
            size: 0,
            modified: None,
        }
    }

    /// 확장자(소문자). 디렉터리거나 확장자가 없으면 빈 문자열
    pub fn ext(&self) -> String {
        if self.is_dir {
            return String::new();
        }
        match self.name.rfind('.') {
            Some(i) if i > 0 => self.name[i + 1..].to_lowercase(),
            _ => String::new(),
        }
    }

    /// 크기 컬럼 표시 문자열
    pub fn size_text(&self) -> String {
        if self.is_dir {
            if self.is_parent {
                "  [..]".to_string()
            } else {
                " <DIR>".to_string()
            }
        } else {
            format_size(self.size)
        }
    }

    /// 수정 시각 컬럼 표시 문자열
    pub fn time_text(&self) -> String {
        match self.modified {
            Some(t) => format_time(t),
            None => String::new(),
        }
    }
}

/// 사람이 읽기 쉬운 크기 (B/K/M/G)
pub fn format_size(bytes: u64) -> String {
    const UNITS: [&str; 4] = ["B", "K", "M", "G"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{} B", bytes)
    } else {
        format!("{:.1} {}", value, UNITS[unit])
    }
}

/// 수정 시각을 "YYYY-MM-DD HH:MM" 형태로
pub fn format_time(t: SystemTime) -> String {
    let dt: chrono::DateTime<chrono::Local> = t.into();
    dt.format("%Y-%m-%d %H:%M").to_string()
}

/// 로컬 디렉터리를 읽어 Entry 목록 생성 (정렬 전).
/// 루트가 아니면 맨 앞에 ".." 추가.
pub fn read_local_dir(path: &Path) -> std::io::Result<Vec<Entry>> {
    let mut entries = Vec::new();
    if path.parent().is_some() {
        entries.push(Entry::parent());
    }
    for dent in std::fs::read_dir(path)? {
        let dent = match dent {
            Ok(d) => d,
            Err(_) => continue,
        };
        let name = dent.file_name().to_string_lossy().to_string();
        let meta = dent.metadata().ok();
        let is_dir = meta.as_ref().map(|m| m.is_dir()).unwrap_or(false);
        let size = meta.as_ref().map(|m| m.len()).unwrap_or(0);
        let modified = meta.as_ref().and_then(|m| m.modified().ok());
        entries.push(Entry {
            name,
            is_dir,
            is_parent: false,
            size,
            modified,
        });
    }
    Ok(entries)
}

/// Entry 목록을 정렬한다. ".." 는 항상 맨 위, 디렉터리가 파일보다 위.
pub fn sort_entries(entries: &mut [Entry], sort: Sort) {
    entries.sort_by(|a, b| {
        // .. 최우선
        if a.is_parent != b.is_parent {
            return b.is_parent.cmp(&a.is_parent);
        }
        // 디렉터리 우선
        if a.is_dir != b.is_dir {
            return b.is_dir.cmp(&a.is_dir);
        }
        let ord = match sort.key {
            SortKey::Name => cmp_name(a, b),
            SortKey::Size => a.size.cmp(&b.size),
            SortKey::Modified => a.modified.cmp(&b.modified),
            SortKey::Ext => a.ext().cmp(&b.ext()).then_with(|| cmp_name(a, b)),
        };
        match sort.order {
            SortOrder::Asc => ord,
            SortOrder::Desc => ord.reverse(),
        }
    });
}

/// 이름 비교 (대소문자 무시)
fn cmp_name(a: &Entry, b: &Entry) -> std::cmp::Ordering {
    a.name.to_lowercase().cmp(&b.name.to_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn size_formatting() {
        assert_eq!(format_size(512), "512 B");
        assert_eq!(format_size(1024), "1.0 K");
        assert_eq!(format_size(1536), "1.5 K");
        assert_eq!(format_size(1048576), "1.0 M");
    }

    #[test]
    fn dirs_sort_before_files_and_parent_first() {
        let mut v = vec![
            Entry { name: "b.txt".into(), is_dir: false, is_parent: false, size: 1, modified: None },
            Entry { name: "adir".into(), is_dir: true, is_parent: false, size: 0, modified: None },
            Entry::parent(),
            Entry { name: "a.txt".into(), is_dir: false, is_parent: false, size: 1, modified: None },
        ];
        sort_entries(&mut v, Sort::default());
        assert!(v[0].is_parent);
        assert_eq!(v[1].name, "adir");
        assert_eq!(v[2].name, "a.txt");
        assert_eq!(v[3].name, "b.txt");
    }

    #[test]
    fn ext_extraction() {
        let e = Entry { name: "Photo.JPG".into(), is_dir: false, is_parent: false, size: 0, modified: None };
        assert_eq!(e.ext(), "jpg");
    }
}
