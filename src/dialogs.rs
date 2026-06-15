// 모달 다이얼로그: 확인(예/아니오), 정보, 텍스트 입력
use native_windows_gui as nwg;
use std::cell::RefCell;
use std::rc::Rc;
use std::thread;

/// 예/아니오 확인. Yes 면 true.
pub fn confirm(parent: nwg::ControlHandle, title: &str, content: &str) -> bool {
    let params = nwg::MessageParams {
        title,
        content,
        buttons: nwg::MessageButtons::YesNo,
        icons: nwg::MessageIcons::Question,
    };
    nwg::modal_message(parent, &params) == nwg::MessageChoice::Yes
}

/// 텍스트 입력 대화상자. 확인 시 입력값, 취소 시 None.
/// 별도 스레드에서 자체 이벤트 루프를 돌려 결과를 반환한다.
pub fn prompt(title: &str, label: &str, default: &str) -> Option<String> {
    let (t, l, d) = (title.to_string(), label.to_string(), default.to_string());
    thread::spawn(move || run_prompt(&t, &l, &d))
        .join()
        .unwrap_or(None)
}

fn run_prompt(title: &str, label: &str, default: &str) -> Option<String> {
    let _ = nwg::init();
    set_dialog_font();

    let mut window = nwg::Window::default();
    let mut lbl = nwg::Label::default();
    let mut input = nwg::TextInput::default();
    let mut ok = nwg::Button::default();
    let mut cancel = nwg::Button::default();

    nwg::Window::builder()
        .size((380, 150))
        .position((520, 360))
        .title(title)
        .flags(nwg::WindowFlags::WINDOW | nwg::WindowFlags::VISIBLE)
        .build(&mut window)
        .expect("dialog window");
    nwg::Label::builder()
        .text(label)
        .position((12, 14))
        .size((356, 22))
        .parent(&window)
        .build(&mut lbl)
        .expect("dialog label");
    nwg::TextInput::builder()
        .text(default)
        .position((12, 42))
        .size((356, 26))
        .parent(&window)
        .build(&mut input)
        .expect("dialog input");
    nwg::Button::builder()
        .text("확인")
        .position((196, 84))
        .size((84, 32))
        .parent(&window)
        .build(&mut ok)
        .expect("dialog ok");
    nwg::Button::builder()
        .text("취소")
        .position((284, 84))
        .size((84, 32))
        .parent(&window)
        .build(&mut cancel)
        .expect("dialog cancel");
    input.set_focus();

    let result: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));
    let input = Rc::new(input);

    let (win_h, ok_h, cancel_h) = (window.handle, ok.handle, cancel.handle);
    let res = result.clone();
    let inp = input.clone();
    let handler = nwg::full_bind_event_handler(&window.handle, move |evt, data, handle| {
        use nwg::Event as E;
        match evt {
            E::OnButtonClick if handle == ok_h => accept(&res, &inp),
            E::OnButtonClick if handle == cancel_h => nwg::stop_thread_dispatch(),
            E::OnWindowClose if handle == win_h => nwg::stop_thread_dispatch(),
            E::OnKeyPress => {
                if let nwg::EventData::OnKey(k) = data {
                    match k {
                        0x0D => accept(&res, &inp), // Enter
                        0x1B => nwg::stop_thread_dispatch(), // Esc
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    });

    nwg::dispatch_thread_events();
    nwg::unbind_event_handler(&handler);

    let out = result.borrow().clone();
    out
}

/// 확인: 값 저장 후 루프 종료
fn accept(res: &Rc<RefCell<Option<String>>>, input: &Rc<nwg::TextInput>) {
    let text = input.text();
    if !text.trim().is_empty() {
        *res.borrow_mut() = Some(text);
    }
    nwg::stop_thread_dispatch();
}

/// 다이얼로그 스레드용 한글 폰트 설정 (별도 스레드 GUI 공용)
pub fn set_dialog_font() {
    let mut font = nwg::Font::default();
    if nwg::Font::builder()
        .size(16)
        .family("맑은 고딕")
        .build(&mut font)
        .is_ok()
    {
        nwg::Font::set_global_default(Some(font));
    }
}
