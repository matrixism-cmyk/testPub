// 안녕 커맨더 - 미드나잇 커맨더 스타일 듀얼 패널 파일 매니저
#![windows_subsystem = "windows"]

mod actions;
mod app;
mod config;
mod dialogs;
mod editor;
mod events;
mod fsops;
mod fsworker;
mod menus;
mod model;
mod progress;
mod remote;
mod state;
mod ui_funcbar;
mod ui_panel;
mod vfs;
mod vfs_ftp;
mod vfs_zip;
mod viewer;

use app::App;
use native_windows_gui as nwg;
use std::rc::Rc;

fn main() {
    nwg::init().expect("nwg init 실패");
    set_global_font();

    let app = Rc::new(App::build());

    // 이벤트 핸들러 등록
    let win = app.window.handle;
    let handler_app = app.clone();
    let handler = nwg::full_bind_event_handler(&win, move |evt, data, handle| {
        events::handle(&handler_app, evt, &data, handle);
    });

    // 최초 레이아웃 및 상태줄
    app.layout();
    app.update_status();
    app.left.list.set_focus();

    nwg::dispatch_thread_events();
    nwg::unbind_event_handler(&handler);
}

/// 한글이 깨지지 않도록 전역 기본 폰트 지정
fn set_global_font() {
    let mut font = nwg::Font::default();
    nwg::Font::builder()
        .size(16)
        .family("맑은 고딕")
        .build(&mut font)
        .expect("폰트 생성 실패");
    nwg::Font::set_global_default(Some(font));
}
