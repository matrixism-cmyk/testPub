// 파일 작업 액션 (복사/이동/새폴더/삭제) + 진행 알림 처리
use crate::app::App;
use crate::dialogs;
use crate::fsworker::{self, Op};
use crate::progress::new_shared;
use crate::vfs_zip;
use native_windows_gui as nwg;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;
use std::thread;

/// 활성 패널(로컬)의 작업 대상 경로들과 그 작업 디렉터리.
/// 로컬이 아니면 None.
fn local_sources(app: &Rc<App>) -> Option<(Vec<PathBuf>, PathBuf)> {
    let p = app.active_panel();
    let cwd = p.local_dir()?;
    let list = p
        .action_entries()
        .iter()
        .map(|e| cwd.join(&e.name))
        .collect();
    Some((list, cwd))
}

/// 이미 작업이 진행 중이면 true (경고 표시)
fn busy(app: &Rc<App>) -> bool {
    if app.job.borrow().is_some() {
        nwg::modal_info_message(&app.window, "작업 중", "이전 작업이 끝날 때까지 기다려 주세요.");
        true
    } else {
        false
    }
}

pub fn copy_action(app: &Rc<App>) {
    if busy(app) {
        return;
    }
    if route_remote(app) {
        return;
    }
    // 활성 패널이 압축이면 추출
    if let Some((archive, inner)) = archive_of(app) {
        extract_action(app, archive, inner);
        return;
    }
    let (srcs, scwd) = match local_sources(app) {
        Some(s) => s,
        None => return,
    };
    let dest = match app.inactive_panel().local_dir() {
        Some(d) => d,
        None => return,
    };
    if srcs.is_empty() || !check_dest(app, &scwd, &dest) {
        return;
    }
    if dialogs::confirm(
        app.window.handle,
        "복사",
        &format!("{}개 항목을 복사할까요?\n→ {}", srcs.len(), dest.display()),
    ) {
        start(app, Op::Copy, srcs, dest);
    }
}

pub fn move_action(app: &Rc<App>) {
    if busy(app) {
        return;
    }
    if route_remote(app) {
        return;
    }
    // 압축 안에서는 이동 대신 추출
    if let Some((archive, inner)) = archive_of(app) {
        extract_action(app, archive, inner);
        return;
    }
    let (srcs, scwd) = match local_sources(app) {
        Some(s) => s,
        None => return,
    };
    let dest = match app.inactive_panel().local_dir() {
        Some(d) => d,
        None => return,
    };
    if srcs.is_empty() || !check_dest(app, &scwd, &dest) {
        return;
    }
    if dialogs::confirm(
        app.window.handle,
        "이동",
        &format!("{}개 항목을 이동할까요?\n→ {}", srcs.len(), dest.display()),
    ) {
        start(app, Op::Move, srcs, dest);
    }
}

pub fn delete_action(app: &Rc<App>) {
    if busy(app) {
        return;
    }
    let (srcs, cwd) = match local_sources(app) {
        Some(s) => s,
        None => {
            nwg::modal_info_message(&app.window, "삭제", "압축/원격 위치에서는 삭제할 수 없습니다.");
            return;
        }
    };
    if srcs.is_empty() {
        return;
    }
    if dialogs::confirm(
        app.window.handle,
        "삭제",
        &format!("{}개 항목을 정말 삭제할까요?\n이 작업은 되돌릴 수 없습니다.", srcs.len()),
    ) {
        start(app, Op::Delete, srcs, cwd);
    }
}

/// 원격이 관여하면 다운로드/업로드로 처리하고 true 반환.
fn route_remote(app: &Rc<App>) -> bool {
    // 활성이 원격 → 다운로드
    if let Some((conn, path)) = app
        .active_panel()
        .location()
        .as_remote()
        .map(|(c, p)| (c.clone(), p.to_string()))
    {
        crate::remote::download(app, conn, path);
        return true;
    }
    // 비활성이 원격 → 업로드
    if let Some((conn, path)) = app
        .inactive_panel()
        .location()
        .as_remote()
        .map(|(c, p)| (c.clone(), p.to_string()))
    {
        crate::remote::upload(app, conn, path);
        return true;
    }
    false
}

/// 활성 패널이 압축 위치면 (아카이브 경로, inner)
fn archive_of(app: &Rc<App>) -> Option<(PathBuf, String)> {
    app.active_panel()
        .location()
        .as_archive()
        .map(|(a, i)| (a.clone(), i.to_string()))
}

/// 압축 항목을 비활성(로컬) 패널 폴더로 추출
fn extract_action(app: &Rc<App>, archive: PathBuf, inner: String) {
    let dest = match app.inactive_panel().local_dir() {
        Some(d) => d,
        None => {
            nwg::modal_info_message(&app.window, "추출", "추출 대상은 로컬 폴더여야 합니다.");
            return;
        }
    };
    let names: Vec<String> = app
        .active_panel()
        .action_entries()
        .iter()
        .map(|e| e.name.clone())
        .collect();
    if names.is_empty() {
        return;
    }
    if !dialogs::confirm(
        app.window.handle,
        "추출",
        &format!("{}개 항목을 추출할까요?\n→ {}", names.len(), dest.display()),
    ) {
        return;
    }
    let shared = new_shared("추출");
    *app.job.borrow_mut() = Some(shared.clone());
    let sender = app.notice.sender();
    let sh = shared.clone();
    thread::spawn(move || {
        let res = vfs_zip::extract(&archive, &inner, &names, &dest, &sh, &sender);
        {
            let mut p = sh.lock().unwrap();
            if let Err(e) = res {
                p.error = Some(e.to_string());
            }
            p.finished = true;
        }
        sender.notice();
    });
    app.status.set_text("추출 준비 중...");
}

pub fn mkdir_action(app: &Rc<App>) {
    let dir = match app.active_panel().local_dir() {
        Some(d) => d,
        None => {
            nwg::modal_info_message(&app.window, "새 폴더", "이 위치에는 폴더를 만들 수 없습니다.");
            return;
        }
    };
    let name = match dialogs::prompt("새 폴더", "폴더 이름:", "새 폴더") {
        Some(n) => n.trim().to_string(),
        None => return,
    };
    let path = dir.join(&name);
    match fs::create_dir(&path) {
        Ok(_) => {
            app.active_panel().reload();
            app.update_status();
        }
        Err(e) => {
            nwg::modal_error_message(&app.window, "오류", &format!("폴더 생성 실패:\n{}", e));
        }
    }
}

/// 복사/이동 대상이 소스 폴더와 같은지 검사
fn check_dest(app: &Rc<App>, scwd: &PathBuf, dest: &PathBuf) -> bool {
    if scwd == dest {
        nwg::modal_info_message(&app.window, "알림", "원본과 대상 폴더가 같습니다.");
        false
    } else {
        true
    }
}

/// 작업 스레드를 띄운다.
fn start(app: &Rc<App>, op: Op, srcs: Vec<PathBuf>, dest: PathBuf) {
    let verb = match op {
        Op::Copy => "복사",
        Op::Move => "이동",
        Op::Delete => "삭제",
    };
    let shared = new_shared(verb);
    *app.job.borrow_mut() = Some(shared.clone());
    let sender = app.notice.sender();
    let sh = shared.clone();
    thread::spawn(move || {
        let total = match op {
            Op::Delete => fsworker::count_items(&srcs),
            _ => fsworker::total_size(&srcs),
        };
        sh.lock().unwrap().total = total;
        let res = fsworker::run(op, &srcs, &dest, &sh, &sender);
        {
            let mut p = sh.lock().unwrap();
            if let Err(e) = res {
                p.error = Some(e);
            }
            p.finished = true;
        }
        sender.notice();
    });
    app.status.set_text(&format!("{} 준비 중...", verb));
}

/// 작업 스레드의 Notice 신호 처리 (진행률 갱신/완료)
pub fn on_progress(app: &Rc<App>) {
    let job = app.job.borrow().clone();
    let sh = match job {
        Some(s) => s,
        None => return,
    };
    let (finished, text, err) = {
        let p = sh.lock().unwrap();
        (
            p.finished,
            format!("{} {}% · {}", p.verb, p.percent(), p.current),
            p.error.clone(),
        )
    };
    app.status.set_text(&text);
    if finished {
        *app.job.borrow_mut() = None;
        app.left.reload();
        app.right.reload();
        if let Some(e) = err {
            nwg::modal_error_message(&app.window, "작업 오류", &e);
        }
        app.update_status();
    }
}
