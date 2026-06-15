// 사용자 동작 처리: 네비게이션 / 정렬 / 함수키 분기
use crate::app::App;
use crate::state::{FuncAction, Side, SortKey, SortOrder};
use native_windows_gui as nwg;
use std::path::Path;
use std::rc::Rc;

/// 함수키/버튼 동작 분기
pub fn func(app: &Rc<App>, action: FuncAction) {
    use FuncAction::*;
    match action {
        Help => help(app),
        View => crate::viewer::view_focused(app),
        Edit => crate::editor::edit_focused(app),
        Copy => crate::fsops::copy_action(app),
        Move => crate::fsops::move_action(app),
        Mkdir => crate::fsops::mkdir_action(app),
        Delete => crate::fsops::delete_action(app),
        Menu => {} // F9: 메뉴 포커스 (윈도우 메뉴는 Alt 로 접근)
        Quit => nwg::stop_thread_dispatch(),
    }
}

/// 항목 활성화(Enter/더블클릭): 디렉터리·압축이면 진입, 파일이면 기본 앱 실행
pub fn activate(app: &Rc<App>, side: Side) {
    let p = app.panel(side);
    let entry = match p.focused_entry() {
        Some(e) => e,
        None => return,
    };
    let loc = p.location();
    if let Some(new_loc) = loc.enter(&entry) {
        p.load(new_loc);
        app.update_status();
    } else if !entry.is_dir {
        if let Some(path) = p.focused_path() {
            open_default(&path);
        }
    }
}

/// 상위 위치로 이동
pub fn go_up(app: &Rc<App>, side: Side) {
    let p = app.panel(side);
    if let Some(parent) = p.location().parent() {
        p.load(parent);
        app.update_status();
    }
}

/// 활성 패널 정렬 변경 (같은 키 재선택 시 방향 토글)
pub fn set_sort(app: &Rc<App>, key: SortKey) {
    let p = app.active_panel();
    {
        let mut st = p.state.borrow_mut();
        if st.sort.key == key {
            st.sort.order = st.sort.order.toggled();
        } else {
            st.sort.key = key;
            st.sort.order = SortOrder::Asc;
        }
    }
    p.resort();
}

/// 활성 패널 새로고침
pub fn refresh(app: &Rc<App>) {
    app.active_panel().reload();
    app.update_status();
}

/// 히스토리 뒤로 가기
pub fn go_back(app: &Rc<App>) {
    app.active_panel().go_back();
    app.update_status();
}

/// 명령줄 실행: 활성 폴더에서 `cmd /C <명령>` 수행
pub fn run_cmdline(app: &Rc<App>) {
    let text = app.cmdline.text();
    let cmd = text.trim();
    if cmd.is_empty() {
        return;
    }
    let mut c = std::process::Command::new("cmd");
    c.args(["/C", cmd]);
    if let Some(dir) = app.active_panel().local_dir() {
        c.current_dir(dir);
    }
    let _ = c.spawn();
    app.cmdline.set_text("");
    app.active_panel().reload();
    app.update_status();
}

/// 도움말
pub fn help(app: &Rc<App>) {
    nwg::modal_info_message(
        &app.window,
        "도움말",
        "안녕 커맨더 단축키\n\n\
         Tab: 패널 전환\n\
         Enter: 폴더 진입 / 파일 열기\n\
         Backspace: 상위 폴더\n\
         Insert: 선택 토글\n\
         F3 보기 · F4 편집 · F5 복사 · F6 이동\n\
         F7 새 폴더 · F8 삭제 · F10 종료",
    );
}

/// 정보
pub fn about(app: &Rc<App>) {
    nwg::modal_info_message(
        &app.window,
        "정보",
        "안녕 커맨더 v0.1.0\n미드나잇 커맨더 스타일 듀얼 패널 파일 매니저\nRust + native-windows-gui",
    );
}

/// 파일을 기본 연결 프로그램으로 연다.
fn open_default(path: &Path) {
    let _ = std::process::Command::new("cmd")
        .args(["/C", "start", "", &path.to_string_lossy()])
        .spawn();
}
