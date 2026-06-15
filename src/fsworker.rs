// 재귀 파일 작업 워커 (작업 스레드에서 실행)
use crate::progress::SharedProgress;
use native_windows_gui as nwg;
use std::fs;
use std::path::Path;

/// 작업 종류
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Op {
    Copy,
    Move,
    Delete,
}

/// 여러 소스에 대해 작업을 수행한다. 실패 시 첫 오류 메시지 반환.
pub fn run(
    op: Op,
    srcs: &[std::path::PathBuf],
    dest_dir: &Path,
    sh: &SharedProgress,
    notice: &nwg::NoticeSender,
) -> Result<(), String> {
    for src in srcs {
        if sh.lock().unwrap().cancel {
            break;
        }
        let name = src.file_name().ok_or("잘못된 경로")?;
        match op {
            Op::Copy => copy_tree(src, &dest_dir.join(name), sh, notice).map_err(io(src))?,
            Op::Move => move_one(src, &dest_dir.join(name), sh, notice).map_err(io(src))?,
            Op::Delete => delete_tree(src, sh, notice).map_err(io(src))?,
        }
    }
    Ok(())
}

/// 파일/디렉터리를 재귀 복사
fn copy_tree(
    src: &Path,
    dst: &Path,
    sh: &SharedProgress,
    notice: &nwg::NoticeSender,
) -> std::io::Result<()> {
    let meta = fs::symlink_metadata(src)?;
    if meta.is_dir() {
        fs::create_dir_all(dst)?;
        for child in fs::read_dir(src)? {
            let child = child?;
            if sh.lock().unwrap().cancel {
                break;
            }
            copy_tree(&child.path(), &dst.join(child.file_name()), sh, notice)?;
        }
    } else {
        set_current(sh, src);
        notice.notice();
        fs::copy(src, dst)?;
        add_done(sh, meta.len());
    }
    Ok(())
}

/// 한 소스를 이동 (같은 볼륨이면 rename, 아니면 복사 후 삭제)
fn move_one(
    src: &Path,
    dst: &Path,
    sh: &SharedProgress,
    notice: &nwg::NoticeSender,
) -> std::io::Result<()> {
    set_current(sh, src);
    notice.notice();
    if fs::rename(src, dst).is_ok() {
        let len = fs::symlink_metadata(dst).map(|m| m.len()).unwrap_or(0);
        add_done(sh, len);
        return Ok(());
    }
    // 볼륨이 다르면 복사 후 원본 삭제
    copy_tree(src, dst, sh, notice)?;
    delete_tree(src, sh, notice)
}

/// 파일/디렉터리를 재귀 삭제 (진행은 항목 수 기준)
fn delete_tree(
    src: &Path,
    sh: &SharedProgress,
    notice: &nwg::NoticeSender,
) -> std::io::Result<()> {
    let meta = fs::symlink_metadata(src)?;
    if meta.is_dir() {
        for child in fs::read_dir(src)? {
            let child = child?;
            delete_tree(&child.path(), sh, notice)?;
        }
        fs::remove_dir(src)?;
    } else {
        set_current(sh, src);
        fs::remove_file(src)?;
    }
    add_done(sh, 1);
    notice.notice();
    Ok(())
}

/// 소스들의 전체 바이트 수 (복사/이동 진행 분모)
pub fn total_size(srcs: &[std::path::PathBuf]) -> u64 {
    srcs.iter().map(|p| size_of(p)).sum()
}

/// 소스들의 전체 항목 수 (삭제 진행 분모)
pub fn count_items(srcs: &[std::path::PathBuf]) -> u64 {
    srcs.iter().map(|p| count_of(p)).sum()
}

fn size_of(p: &Path) -> u64 {
    match fs::symlink_metadata(p) {
        Ok(m) if m.is_dir() => fs::read_dir(p)
            .map(|rd| rd.flatten().map(|e| size_of(&e.path())).sum())
            .unwrap_or(0),
        Ok(m) => m.len(),
        Err(_) => 0,
    }
}

fn count_of(p: &Path) -> u64 {
    match fs::symlink_metadata(p) {
        Ok(m) if m.is_dir() => {
            1 + fs::read_dir(p)
                .map(|rd| rd.flatten().map(|e| count_of(&e.path())).sum::<u64>())
                .unwrap_or(0)
        }
        _ => 1,
    }
}

fn set_current(sh: &SharedProgress, p: &Path) {
    let name = p
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    sh.lock().unwrap().current = name;
}

fn add_done(sh: &SharedProgress, amount: u64) {
    sh.lock().unwrap().done += amount;
}

/// io 오류에 경로 컨텍스트를 붙여 문자열로
fn io(path: &Path) -> impl Fn(std::io::Error) -> String + '_ {
    move |e| format!("{}\n{}", path.display(), e)
}
