// 상단 메뉴 바 (파일/명령/정렬/설정/도움말)
use native_windows_gui as nwg;

/// 메뉴 바 전체. 클릭 처리에 필요한 항목만 pub 으로 노출.
#[allow(dead_code)] // 최상위 메뉴/구분선은 보관만 함
pub struct Menus {
    file: nwg::Menu,
    pub file_refresh: nwg::MenuItem,
    file_sep: nwg::MenuSeparator,
    pub file_quit: nwg::MenuItem,

    cmd: nwg::Menu,
    pub cmd_view: nwg::MenuItem,
    pub cmd_edit: nwg::MenuItem,
    pub cmd_copy: nwg::MenuItem,
    pub cmd_move: nwg::MenuItem,
    pub cmd_mkdir: nwg::MenuItem,
    pub cmd_delete: nwg::MenuItem,

    sort: nwg::Menu,
    pub sort_name: nwg::MenuItem,
    pub sort_size: nwg::MenuItem,
    pub sort_time: nwg::MenuItem,
    pub sort_ext: nwg::MenuItem,

    settings: nwg::Menu,
    pub settings_bookmarks: nwg::MenuItem,
    pub settings_prefs: nwg::MenuItem,

    help: nwg::Menu,
    pub help_about: nwg::MenuItem,
}

impl Menus {
    pub fn build(window: &nwg::Window) -> Menus {
        let mut m = Menus {
            file: nwg::Menu::default(),
            file_refresh: nwg::MenuItem::default(),
            file_sep: nwg::MenuSeparator::default(),
            file_quit: nwg::MenuItem::default(),
            cmd: nwg::Menu::default(),
            cmd_view: nwg::MenuItem::default(),
            cmd_edit: nwg::MenuItem::default(),
            cmd_copy: nwg::MenuItem::default(),
            cmd_move: nwg::MenuItem::default(),
            cmd_mkdir: nwg::MenuItem::default(),
            cmd_delete: nwg::MenuItem::default(),
            sort: nwg::Menu::default(),
            sort_name: nwg::MenuItem::default(),
            sort_size: nwg::MenuItem::default(),
            sort_time: nwg::MenuItem::default(),
            sort_ext: nwg::MenuItem::default(),
            settings: nwg::Menu::default(),
            settings_bookmarks: nwg::MenuItem::default(),
            settings_prefs: nwg::MenuItem::default(),
            help: nwg::Menu::default(),
            help_about: nwg::MenuItem::default(),
        };

        top(window, "파일(&F)", &mut m.file);
        item(&m.file, "새로고침\tCtrl+R", &mut m.file_refresh);
        sep(&m.file, &mut m.file_sep);
        item(&m.file, "종료\tF10", &mut m.file_quit);

        top(window, "명령(&C)", &mut m.cmd);
        item(&m.cmd, "보기\tF3", &mut m.cmd_view);
        item(&m.cmd, "편집\tF4", &mut m.cmd_edit);
        item(&m.cmd, "복사\tF5", &mut m.cmd_copy);
        item(&m.cmd, "이동\tF6", &mut m.cmd_move);
        item(&m.cmd, "새 폴더\tF7", &mut m.cmd_mkdir);
        item(&m.cmd, "삭제\tF8", &mut m.cmd_delete);

        top(window, "정렬(&O)", &mut m.sort);
        item(&m.sort, "이름순", &mut m.sort_name);
        item(&m.sort, "크기순", &mut m.sort_size);
        item(&m.sort, "수정시각순", &mut m.sort_time);
        item(&m.sort, "확장자순", &mut m.sort_ext);

        top(window, "설정(&S)", &mut m.settings);
        item(&m.settings, "북마크", &mut m.settings_bookmarks);
        item(&m.settings, "환경설정", &mut m.settings_prefs);

        top(window, "도움말(&H)", &mut m.help);
        item(&m.help, "정보", &mut m.help_about);

        m
    }
}

fn top(window: &nwg::Window, text: &str, out: &mut nwg::Menu) {
    nwg::Menu::builder()
        .text(text)
        .parent(window)
        .build(out)
        .expect("top menu");
}

fn item(parent: &nwg::Menu, text: &str, out: &mut nwg::MenuItem) {
    nwg::MenuItem::builder()
        .text(text)
        .parent(parent)
        .build(out)
        .expect("menu item");
}

fn sep(parent: &nwg::Menu, out: &mut nwg::MenuSeparator) {
    nwg::MenuSeparator::builder()
        .parent(parent)
        .build(out)
        .expect("menu sep");
}
