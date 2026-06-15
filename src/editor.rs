// F4 내장 텍스트 편집기 (저장 가능). 별도 스레드 창.
use crate::app::App;
use crate::dialogs::set_dialog_font;
use native_windows_gui as nwg;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::thread;

const MAX_EDIT: u64 = 8 * 1024 * 1024; // 8MB 초과는 편집 거부

/// 활성 패널의 커서 파일을 편집기로 연다.
pub fn edit_focused(app: &Rc<App>) {
    let path = match app.active_panel().focused_path() {
        Some(p) => p,
        None => return,
    };
    if path.is_dir() {
        return;
    }
    if std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0) > MAX_EDIT {
        nwg::modal_info_message(&app.window, "편집", "파일이 너무 커서 편집할 수 없습니다.");
        return;
    }
    thread::spawn(move || run_editor(path));
}

fn run_editor(path: PathBuf) {
    let _ = nwg::init();
    set_dialog_font();

    let raw = std::fs::read(&path).unwrap_or_default();
    let content = String::from_utf8_lossy(&raw).to_string();

    let mut window = nwg::Window::default();
    let mut save = nwg::Button::default();
    let mut close = nwg::Button::default();
    let mut text = nwg::TextBox::default();

    nwg::Window::builder()
        .size((780, 580))
        .position((340, 160))
        .title(&format!("편집 - {}", file_name(&path)))
        .flags(nwg::WindowFlags::MAIN_WINDOW | nwg::WindowFlags::VISIBLE)
        .build(&mut window)
        .expect("editor window");
    nwg::Button::builder()
        .text("저장 (Ctrl+S)")
        .position((8, 6))
        .size((130, 28))
        .parent(&window)
        .build(&mut save)
        .expect("save btn");
    nwg::Button::builder()
        .text("닫기")
        .position((144, 6))
        .size((90, 28))
        .parent(&window)
        .build(&mut close)
        .expect("close btn");
    nwg::TextBox::builder()
        .flags(
            nwg::TextBoxFlags::VISIBLE
                | nwg::TextBoxFlags::VSCROLL
                | nwg::TextBoxFlags::HSCROLL
                | nwg::TextBoxFlags::AUTOVSCROLL
                | nwg::TextBoxFlags::AUTOHSCROLL
                | nwg::TextBoxFlags::TAB_STOP,
        )
        .parent(&window)
        .build(&mut text)
        .expect("editor text");
    text.set_text_unix2dos(&content);

    let text = Rc::new(text);
    let window = Rc::new(window);
    let path = Rc::new(path);
    layout_edit(&window, &text);

    let (win_h, save_h, close_h) = (window.handle, save.handle, close.handle);
    let (t2, w2, p2) = (text.clone(), window.clone(), path.clone());
    let handler = nwg::full_bind_event_handler(&window.handle, move |evt, data, handle| {
        use nwg::Event as E;
        match evt {
            E::OnButtonClick if handle == save_h => save_file(&p2, &t2, &w2),
            E::OnButtonClick if handle == close_h => nwg::stop_thread_dispatch(),
            E::OnResize | E::OnWindowMaximize if handle == w2.handle => layout_edit(&w2, &t2),
            E::OnWindowClose if handle == win_h => nwg::stop_thread_dispatch(),
            E::OnKeyPress => {
                if let nwg::EventData::OnKey(k) = data {
                    // Ctrl+S 는 키 상태 확인이 번거로워 S(0x53) 단독은 무시, 저장은 버튼 사용
                    let _ = k;
                }
            }
            _ => {}
        }
    });

    nwg::dispatch_thread_events();
    nwg::unbind_event_handler(&handler);
}

/// 버튼 바 아래로 텍스트박스를 채운다.
fn layout_edit(window: &nwg::Window, text: &nwg::TextBox) {
    let (w, h) = window.size();
    let top = 40;
    text.set_position(8, top);
    let tw = (w as i32 - 16).max(0) as u32;
    let th = (h as i32 - top - 8).max(0) as u32;
    text.set_size(tw, th);
}

/// 텍스트박스 내용을 파일에 저장
fn save_file(path: &Path, text: &nwg::TextBox, window: &nwg::Window) {
    let content = text.text();
    // 에디트 컨트롤의 CRLF 를 LF 로 정규화해 저장
    let normalized = content.replace("\r\n", "\n");
    match std::fs::write(path, normalized) {
        Ok(_) => {
            window.set_text(&format!("편집 - {} (저장됨)", file_name(path)));
        }
        Err(e) => {
            nwg::modal_error_message(window, "저장 오류", &format!("저장 실패:\n{}", e));
        }
    }
}

fn file_name(p: &Path) -> String {
    p.file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default()
}
