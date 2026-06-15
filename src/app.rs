// 앱 전체 조립 + 수동 레이아웃 + 패널 헬퍼
use crate::menus::Menus;
use crate::progress::SharedProgress;
use crate::state::{default_start_dir, Side};
use crate::ui_funcbar::FuncBar;
use crate::ui_panel::Panel;
use native_windows_gui as nwg;
use std::cell::{Cell, RefCell};

/// 애플리케이션 전체 상태
pub struct App {
    pub window: nwg::Window,
    pub menus: Menus,
    pub left: Panel,
    pub right: Panel,
    pub funcbar: FuncBar,
    pub status: nwg::Label,
    pub active: Cell<Side>,
    pub notice: nwg::Notice,
    pub job: RefCell<Option<SharedProgress>>,
}

impl App {
    /// 모든 컨트롤을 만들고 App 을 반환한다.
    pub fn build() -> App {
        let start = default_start_dir();

        let mut window = nwg::Window::default();
        nwg::Window::builder()
            .size((960, 640))
            .position((300, 150))
            .title("안녕 커맨더")
            .flags(
                nwg::WindowFlags::MAIN_WINDOW | nwg::WindowFlags::VISIBLE,
            )
            .build(&mut window)
            .expect("main window");

        let menus = Menus::build(&window);
        let left = Panel::build(&window, Side::Left, &start);
        let right = Panel::build(&window, Side::Right, &start);
        let funcbar = FuncBar::build(&window);

        let mut status = nwg::Label::default();
        nwg::Label::builder()
            .text("준비")
            .parent(&window)
            .build(&mut status)
            .expect("status label");

        let mut notice = nwg::Notice::default();
        nwg::Notice::builder()
            .parent(&window)
            .build(&mut notice)
            .expect("notice");

        App {
            window,
            menus,
            left,
            right,
            funcbar,
            status,
            active: Cell::new(Side::Left),
            notice,
            job: RefCell::new(None),
        }
    }

    /// 지정 쪽 패널 참조
    pub fn panel(&self, side: Side) -> &Panel {
        match side {
            Side::Left => &self.left,
            Side::Right => &self.right,
        }
    }

    /// 활성 패널
    pub fn active_panel(&self) -> &Panel {
        self.panel(self.active.get())
    }

    /// 비활성(반대쪽) 패널
    pub fn inactive_panel(&self) -> &Panel {
        self.panel(self.active.get().other())
    }

    /// 활성 패널 변경 + 상태줄 갱신
    pub fn set_active(&self, side: Side) {
        self.active.set(side);
        self.update_status();
    }

    /// 하단 상태줄 갱신
    pub fn update_status(&self) {
        let side = match self.active.get() {
            Side::Left => "왼쪽",
            Side::Right => "오른쪽",
        };
        let p = self.active_panel();
        let name = p
            .focused_entry()
            .map(|e| e.name)
            .unwrap_or_else(|| "-".to_string());
        self.status.set_text(&format!(
            "[{}] {}  ▶  {}",
            side,
            p.cwd().display(),
            name
        ));
        self.left.update_info();
        self.right.update_info();
    }

    /// 창 크기에 맞춰 모든 컨트롤 배치
    pub fn layout(&self) {
        let (w, h) = self.window.size();
        let (w, h) = (w as i32, h as i32);
        let m = 6;
        let status_h = 22;
        let func_h = 30;

        // 상태줄 (맨 아래)
        place(&self.status, 0, h - status_h, w, status_h);

        // 함수키 바
        let fb_y = h - status_h - func_h;
        let n = self.funcbar.len() as i32;
        let gap = 2;
        let total = (w - 2 * m - gap * (n - 1)).max(n);
        let bw = total / n;
        for (i, fbtn) in self.funcbar.buttons.iter().enumerate() {
            let x = m + (bw + gap) * i as i32;
            place(&fbtn.button, x, fb_y, bw, func_h - 2);
        }

        // 패널 영역
        let area_top = m;
        let area_bottom = fb_y - m;
        let area_h = (area_bottom - area_top).max(0);
        let panel_w = ((w - 3 * m) / 2).max(0);
        self.layout_panel(&self.left, m, area_top, panel_w, area_h);
        self.layout_panel(&self.right, m + panel_w + m, area_top, panel_w, area_h);
    }

    /// 패널 내부(경로/리스트/정보) 배치
    fn layout_panel(&self, p: &Panel, x: i32, y: i32, pw: i32, ph: i32) {
        let path_h = 20;
        let info_h = 18;
        place(&p.path_label, x, y, pw, path_h);
        let list_y = y + path_h;
        let list_h = (ph - path_h - info_h).max(0);
        place_list(&p.list, x, list_y, pw, list_h);
        place(&p.info_label, x, list_y + list_h, pw, info_h);
    }
}

/// 라벨/버튼 위치·크기 지정 (음수 방지)
fn place<W>(ctrl: &W, x: i32, y: i32, w: i32, h: i32)
where
    W: PlaceCtrl,
{
    ctrl.set_pos(x, y);
    ctrl.set_dim(w.max(0) as u32, h.max(0) as u32);
}

fn place_list(list: &nwg::ListView, x: i32, y: i32, w: i32, h: i32) {
    list.set_position(x, y);
    list.set_size(w.max(0) as u32, h.max(0) as u32);
}

/// set_position/set_size 를 공통으로 호출하기 위한 트레잇
trait PlaceCtrl {
    fn set_pos(&self, x: i32, y: i32);
    fn set_dim(&self, w: u32, h: u32);
}

impl PlaceCtrl for nwg::Label {
    fn set_pos(&self, x: i32, y: i32) {
        self.set_position(x, y);
    }
    fn set_dim(&self, w: u32, h: u32) {
        self.set_size(w, h);
    }
}

impl PlaceCtrl for nwg::Button {
    fn set_pos(&self, x: i32, y: i32) {
        self.set_position(x, y);
    }
    fn set_dim(&self, w: u32, h: u32) {
        self.set_size(w, h);
    }
}
