// 설정/북마크/히스토리 (Phase 6 에서 구현). 현재는 자리표시자.
use crate::app::App;
use native_windows_gui as nwg;
use std::rc::Rc;

pub fn open_bookmarks(app: &Rc<App>) {
    nwg::modal_info_message(&app.window, "북마크", "북마크: 다음 단계에서 구현됩니다.");
}

pub fn open_prefs(app: &Rc<App>) {
    nwg::modal_info_message(&app.window, "환경설정", "환경설정: 다음 단계에서 구현됩니다.");
}
