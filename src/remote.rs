// FTP 연결/끊기 + 다운로드/업로드 (UI 스레드에서 동기 수행)
use crate::app::App;
use crate::dialogs;
use crate::state::default_start_dir;
use crate::vfs::{FtpHandle, Location};
use crate::vfs_ftp::FtpSession;
use native_windows_gui as nwg;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

/// FTP 서버에 연결해 활성 패널을 원격 위치로 전환
pub fn connect_ftp(app: &Rc<App>) {
    let host_in = match dialogs::prompt("FTP 연결", "호스트 (host 또는 host:port)", "") {
        Some(h) => h.trim().to_string(),
        None => return,
    };
    if host_in.is_empty() {
        return;
    }
    let (host, port) = split_host_port(&host_in);
    let user = dialogs::prompt("FTP 연결", "사용자 이름", "anonymous").unwrap_or_default();
    let pass = dialogs::prompt("FTP 연결", "비밀번호", "").unwrap_or_default();

    app.status.set_text(&format!("{} 연결 중...", host));
    match FtpSession::connect(&host, port, &user, &pass) {
        Ok(sess) => {
            let conn: FtpHandle = Arc::new(Mutex::new(sess));
            let loc = Location::Remote {
                conn,
                host,
                path: "/".to_string(),
            };
            app.active_panel().load(loc);
            app.update_status();
        }
        Err(e) => {
            nwg::modal_error_message(&app.window, "FTP 오류", &format!("연결 실패:\n{}", e));
            app.update_status();
        }
    }
}

/// 활성 패널을 로컬 홈으로 되돌린다 (원격 연결 종료)
pub fn disconnect(app: &Rc<App>) {
    app.active_panel().load(Location::Local(default_start_dir()));
    app.update_status();
}

/// 원격(활성) → 로컬(비활성) 다운로드
pub fn download(app: &Rc<App>, conn: FtpHandle, rdir: String) {
    let dest = match app.inactive_panel().local_dir() {
        Some(d) => d,
        None => {
            nwg::modal_info_message(&app.window, "다운로드", "대상은 로컬 폴더여야 합니다.");
            return;
        }
    };
    let files: Vec<String> = app
        .active_panel()
        .action_entries()
        .iter()
        .filter(|e| !e.is_dir)
        .map(|e| e.name.clone())
        .collect();
    if files.is_empty() {
        nwg::modal_info_message(&app.window, "다운로드", "다운로드할 파일을 선택하세요. (폴더는 지원 안 함)");
        return;
    }
    if !dialogs::confirm(
        app.window.handle,
        "다운로드",
        &format!("{}개 파일을 받을까요?\n→ {}", files.len(), dest.display()),
    ) {
        return;
    }
    let mut errors = Vec::new();
    for name in &files {
        app.status.set_text(&format!("다운로드: {}", name));
        let r = conn.lock().unwrap().download(&rdir, name, &dest.join(name));
        if let Err(e) = r {
            errors.push(format!("{}: {}", name, e));
        }
    }
    app.inactive_panel().reload();
    finish(app, "다운로드", errors);
}

/// 로컬(활성) → 원격(비활성) 업로드
pub fn upload(app: &Rc<App>, conn: FtpHandle, rdir: String) {
    let dir = match app.active_panel().local_dir() {
        Some(d) => d,
        None => return,
    };
    let files: Vec<String> = app
        .active_panel()
        .action_entries()
        .iter()
        .filter(|e| !e.is_dir)
        .map(|e| e.name.clone())
        .collect();
    if files.is_empty() {
        nwg::modal_info_message(&app.window, "업로드", "업로드할 파일을 선택하세요. (폴더는 지원 안 함)");
        return;
    }
    if !dialogs::confirm(
        app.window.handle,
        "업로드",
        &format!("{}개 파일을 올릴까요?\n→ ftp:{}", files.len(), rdir),
    ) {
        return;
    }
    let mut errors = Vec::new();
    for name in &files {
        app.status.set_text(&format!("업로드: {}", name));
        let r = conn.lock().unwrap().upload(&rdir, &dir.join(name), name);
        if let Err(e) = r {
            errors.push(format!("{}: {}", name, e));
        }
    }
    app.inactive_panel().reload();
    finish(app, "업로드", errors);
}

/// 결과 보고 + 상태 복구
fn finish(app: &Rc<App>, verb: &str, errors: Vec<String>) {
    if errors.is_empty() {
        app.status.set_text(&format!("{} 완료", verb));
    } else {
        nwg::modal_error_message(&app.window, verb, &errors.join("\n"));
    }
    app.update_status();
}

/// "host:port" 를 (host, port) 로 분리. 포트 없으면 21.
fn split_host_port(s: &str) -> (String, u16) {
    match s.rsplit_once(':') {
        Some((h, p)) => (h.to_string(), p.parse().unwrap_or(21)),
        None => (s.to_string(), 21),
    }
}
