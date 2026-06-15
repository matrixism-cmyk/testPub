// zip 아카이브 탐색 + 추출
use crate::model::Entry;
use crate::progress::SharedProgress;
use native_windows_gui as nwg;
use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io;
use std::path::Path;

/// 아카이브 내부 inner 디렉터리 바로 아래 항목 나열.
/// inner: "" = 루트, 그 외 "a/b" (슬래시 구분, 끝 슬래시 없음)
pub fn list(archive: &Path, inner: &str) -> io::Result<Vec<Entry>> {
    let file = File::open(archive)?;
    let mut zip = zip::ZipArchive::new(file).map_err(to_io)?;
    let prefix = if inner.is_empty() {
        String::new()
    } else {
        format!("{}/", inner)
    };

    // child 이름 -> (디렉터리 여부, 크기)
    let mut children: BTreeMap<String, (bool, u64)> = BTreeMap::new();
    for i in 0..zip.len() {
        let e = zip.by_index(i).map_err(to_io)?;
        let name = e.name().replace('\\', "/");
        if !name.starts_with(&prefix) {
            continue;
        }
        let rest = &name[prefix.len()..];
        if rest.is_empty() {
            continue;
        }
        match rest.find('/') {
            Some(p) => {
                children.entry(rest[..p].to_string()).or_insert((true, 0));
            }
            None => {
                if !name.ends_with('/') {
                    children.insert(rest.to_string(), (false, e.size()));
                }
            }
        }
    }

    let mut out = Vec::with_capacity(children.len() + 1);
    out.push(Entry::parent());
    for (name, (is_dir, size)) in children {
        out.push(Entry {
            name,
            is_dir,
            is_parent: false,
            size,
            modified: None,
        });
    }
    Ok(out)
}

/// inner 아래의 선택된 항목(names)을 dest 디렉터리로 추출.
/// 추출 트리는 사용자가 보던 inner 기준 상대경로를 따른다.
pub fn extract(
    archive: &Path,
    inner: &str,
    names: &[String],
    dest: &Path,
    sh: &SharedProgress,
    notice: &nwg::NoticeSender,
) -> io::Result<()> {
    let file = File::open(archive)?;
    let mut zip = zip::ZipArchive::new(file).map_err(to_io)?;
    let prefix = if inner.is_empty() {
        String::new()
    } else {
        format!("{}/", inner)
    };
    let selected: Vec<String> = names.iter().map(|n| format!("{}{}", prefix, n)).collect();

    sh.lock().unwrap().total = zip.len() as u64;
    for i in 0..zip.len() {
        if sh.lock().unwrap().cancel {
            break;
        }
        let mut e = zip.by_index(i).map_err(to_io)?;
        let name = e.name().replace('\\', "/");
        let under = selected
            .iter()
            .any(|s| name == *s || name.starts_with(&format!("{}/", s)));
        if !under {
            sh.lock().unwrap().done += 1;
            continue;
        }
        let rel = &name[prefix.len().min(name.len())..];
        let outpath = dest.join(rel);
        sh.lock().unwrap().current = rel.to_string();
        notice.notice();
        if name.ends_with('/') {
            fs::create_dir_all(&outpath)?;
        } else {
            if let Some(parent) = outpath.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut out = File::create(&outpath)?;
            io::copy(&mut e, &mut out)?;
        }
        sh.lock().unwrap().done += 1;
    }
    Ok(())
}

fn to_io(e: zip::result::ZipError) -> io::Error {
    io::Error::new(io::ErrorKind::Other, format!("zip: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    /// 테스트용 zip 생성: a.txt, dir/b.txt, dir/sub/c.txt
    fn make_zip() -> std::path::PathBuf {
        let path = std::env::temp_dir().join("commander_test.zip");
        let f = File::create(&path).unwrap();
        let mut zw = zip::ZipWriter::new(f);
        let opt = zip::write::FileOptions::default();
        for (name, body) in [
            ("a.txt", "AAA"),
            ("dir/b.txt", "BBB"),
            ("dir/sub/c.txt", "CCC"),
        ] {
            zw.start_file(name, opt).unwrap();
            zw.write_all(body.as_bytes()).unwrap();
        }
        zw.finish().unwrap();
        path
    }

    #[test]
    fn lists_root_and_subdir() {
        let zip = make_zip();
        let root = list(&zip, "").unwrap();
        let names: Vec<&str> = root.iter().map(|e| e.name.as_str()).collect();
        assert!(names.contains(&"..")); // 부모 항목
        assert!(names.contains(&"a.txt"));
        assert!(names.contains(&"dir"));
        // dir 는 디렉터리로 인식
        assert!(root.iter().any(|e| e.name == "dir" && e.is_dir));

        let sub = list(&zip, "dir").unwrap();
        let names: Vec<&str> = sub.iter().map(|e| e.name.as_str()).collect();
        assert!(names.contains(&"b.txt"));
        assert!(names.contains(&"sub"));
        assert!(!names.contains(&"a.txt")); // 다른 레벨 항목은 안 보임
    }
}
