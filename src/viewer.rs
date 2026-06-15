// F3 파일 뷰어 (텍스트/헥스, 읽기 전용). 별도 스레드 창.
use crate::app::App;
use crate::dialogs::set_dialog_font;
use native_windows_gui as nwg;
use std::cell::Cell;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::thread;

const MAX_VIEW: usize = 4 * 1024 * 1024; // 4MB 까지만

/// 활성 패널의 커서 파일을 뷰어로 연다.
pub fn view_focused(app: &Rc<App>) {
    let path = match app.active_panel().focused_path() {
        Some(p) => p,
        None => return,
    };
    if path.is_dir() {
        return;
    }
    thread::spawn(move || run_viewer(path));
}

fn run_viewer(path: PathBuf) {
    let _ = nwg::init();
    set_dialog_font();

    let mut data = std::fs::read(&path).unwrap_or_default();
    data.truncate(MAX_VIEW);
    let data = Rc::new(data);
    let hex = Rc::new(Cell::new(!looks_text(&data)));

    let mut window = nwg::Window::default();
    let mut toggle = nwg::Button::default();
    let mut text = nwg::TextBox::default();

    nwg::Window::builder()
        .size((760, 560))
        .position((360, 180))
        .title(&format!("보기 - {}", file_name(&path)))
        .flags(nwg::WindowFlags::MAIN_WINDOW | nwg::WindowFlags::VISIBLE)
        .build(&mut window)
        .expect("viewer window");
    nwg::Button::builder()
        .text("텍스트/헥스 전환")
        .position((8, 6))
        .size((160, 28))
        .parent(&window)
        .build(&mut toggle)
        .expect("toggle");
    nwg::TextBox::builder()
        .flags(
            nwg::TextBoxFlags::VISIBLE
                | nwg::TextBoxFlags::VSCROLL
                | nwg::TextBoxFlags::HSCROLL
                | nwg::TextBoxFlags::AUTOHSCROLL,
        )
        .readonly(true)
        .parent(&window)
        .build(&mut text)
        .expect("viewer text");

    let text = Rc::new(text);
    let window = Rc::new(window);
    render(&text, &data, hex.get());
    layout_view(&window, &text);

    let (win_h, tog_h) = (window.handle, toggle.handle);
    let (t2, d2, h2, w2) = (text.clone(), data.clone(), hex.clone(), window.clone());
    let handler = nwg::full_bind_event_handler(&window.handle, move |evt, _d, handle| {
        use nwg::Event as E;
        match evt {
            E::OnButtonClick if handle == tog_h => {
                h2.set(!h2.get());
                render(&t2, &d2, h2.get());
            }
            E::OnResize | E::OnWindowMaximize if handle == w2.handle => layout_view(&w2, &t2),
            E::OnWindowClose if handle == win_h => nwg::stop_thread_dispatch(),
            _ => {}
        }
    });

    nwg::dispatch_thread_events();
    nwg::unbind_event_handler(&handler);
}

/// 토글 버튼 아래로 텍스트박스를 채운다.
fn layout_view(window: &nwg::Window, text: &nwg::TextBox) {
    let (w, h) = window.size();
    let top = 40;
    text.set_position(8, top);
    let tw = (w as i32 - 16).max(0) as u32;
    let th = (h as i32 - top - 8).max(0) as u32;
    text.set_size(tw, th);
}

/// 데이터를 텍스트/헥스로 렌더링해 표시
fn render(text: &nwg::TextBox, data: &[u8], hex: bool) {
    if hex {
        text.set_text(&hex_dump(data));
    } else {
        let s = String::from_utf8_lossy(data);
        text.set_text_unix2dos(&s);
    }
}

/// 헥스 덤프 문자열 생성
fn hex_dump(data: &[u8]) -> String {
    let mut out = String::with_capacity(data.len() * 4);
    for (i, chunk) in data.chunks(16).enumerate() {
        let mut hexs = String::new();
        let mut asc = String::new();
        for b in chunk {
            hexs.push_str(&format!("{:02X} ", b));
            asc.push(if b.is_ascii_graphic() || *b == b' ' {
                *b as char
            } else {
                '.'
            });
        }
        out.push_str(&format!("{:08X}  {:<48} {}\r\n", i * 16, hexs, asc));
    }
    out
}

/// 텍스트 파일로 보이는지 (NUL 바이트가 없으면 텍스트로 간주)
fn looks_text(data: &[u8]) -> bool {
    let sample = &data[..data.len().min(8000)];
    !sample.contains(&0)
}

fn file_name(p: &Path) -> String {
    p.file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default()
}
