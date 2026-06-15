// 이벤트 디스패치: 키/메뉴/버튼/리스트 → 액션
use crate::actions;
use crate::app::App;
use crate::state::{FuncAction, Side, SortKey};
use native_windows_gui as nwg;
use std::rc::Rc;

/// 최상위 이벤트 핸들러
pub fn handle(app: &Rc<App>, evt: nwg::Event, data: &nwg::EventData, handle: nwg::ControlHandle) {
    use nwg::Event as E;
    match evt {
        E::OnWindowClose if handle == app.window.handle => {
            crate::config::save_dirs(app);
            nwg::stop_thread_dispatch();
        }
        E::OnResize | E::OnWindowMaximize if handle == app.window.handle => app.layout(),
        E::OnListViewFocus => {
            if let Some(s) = side_of(app, handle) {
                app.set_active(s);
            }
        }
        E::OnListViewItemChanged => app.update_status(),
        E::OnListViewItemActivated => {
            if let Some(s) = side_of(app, handle) {
                actions::activate(app, s);
            }
        }
        E::OnButtonClick => {
            if let Some(a) = app.funcbar.action_of(handle) {
                actions::func(app, a);
            }
        }
        E::OnNotice if handle == app.notice.handle => crate::fsops::on_progress(app),
        E::OnMenuItemSelected => menu(app, handle),
        E::OnKeyPress => {
            if let nwg::EventData::OnKey(k) = data {
                if handle == app.cmdline.handle {
                    if *k == 0x0D {
                        actions::run_cmdline(app);
                    }
                } else {
                    keys(app, *k);
                }
            }
        }
        _ => {}
    }
}

/// 리스트뷰 핸들로 어느 쪽 패널인지 판별
fn side_of(app: &Rc<App>, handle: nwg::ControlHandle) -> Option<Side> {
    if handle == app.left.list.handle {
        Some(Side::Left)
    } else if handle == app.right.list.handle {
        Some(Side::Right)
    } else {
        None
    }
}

/// 키 입력 처리 (가상 키 코드)
fn keys(app: &Rc<App>, vk: u32) {
    const VK_BACK: u32 = 0x08;
    match vk {
        VK_BACK => actions::go_up(app, app.active.get()),
        0x70 => actions::func(app, FuncAction::Help),   // F1
        0x72 => actions::func(app, FuncAction::View),   // F3
        0x73 => actions::func(app, FuncAction::Edit),   // F4
        0x74 => actions::func(app, FuncAction::Copy),   // F5
        0x75 => actions::func(app, FuncAction::Move),   // F6
        0x76 => actions::func(app, FuncAction::Mkdir),  // F7
        0x77 => actions::func(app, FuncAction::Delete), // F8
        0x79 => actions::func(app, FuncAction::Quit),   // F10
        _ => {}
    }
}

/// 메뉴 항목 선택 처리
fn menu(app: &Rc<App>, handle: nwg::ControlHandle) {
    let m = &app.menus;
    if handle == m.file_quit.handle {
        crate::config::save_dirs(app);
        nwg::stop_thread_dispatch();
    } else if handle == m.file_refresh.handle {
        actions::refresh(app);
    } else if handle == m.file_back.handle {
        actions::go_back(app);
    } else if handle == m.cmd_view.handle {
        actions::func(app, FuncAction::View);
    } else if handle == m.cmd_edit.handle {
        actions::func(app, FuncAction::Edit);
    } else if handle == m.cmd_copy.handle {
        actions::func(app, FuncAction::Copy);
    } else if handle == m.cmd_move.handle {
        actions::func(app, FuncAction::Move);
    } else if handle == m.cmd_mkdir.handle {
        actions::func(app, FuncAction::Mkdir);
    } else if handle == m.cmd_delete.handle {
        actions::func(app, FuncAction::Delete);
    } else if handle == m.sort_name.handle {
        actions::set_sort(app, SortKey::Name);
    } else if handle == m.sort_size.handle {
        actions::set_sort(app, SortKey::Size);
    } else if handle == m.sort_time.handle {
        actions::set_sort(app, SortKey::Modified);
    } else if handle == m.sort_ext.handle {
        actions::set_sort(app, SortKey::Ext);
    } else if handle == m.net_ftp.handle {
        crate::remote::connect_ftp(app);
    } else if handle == m.net_disconnect.handle {
        crate::remote::disconnect(app);
    } else if handle == m.settings_add_bookmark.handle {
        crate::config::add_bookmark(app);
    } else if handle == m.settings_bookmarks.handle {
        crate::config::open_bookmarks(app);
    } else if handle == m.settings_prefs.handle {
        crate::config::open_prefs(app);
    } else if handle == m.help_about.handle {
        actions::about(app);
    }
}
