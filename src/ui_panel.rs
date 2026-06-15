// 단일 파일 패널: 경로 라벨 + ListView + 정보 라벨
use crate::model::{read_local_dir, sort_entries, Entry};
use crate::state::{Side, Sort};
use native_windows_gui as nwg;
use std::cell::RefCell;
use std::path::{Path, PathBuf};

/// 패널의 가변 상태
pub struct PanelState {
    pub cwd: PathBuf,
    pub entries: Vec<Entry>,
    pub sort: Sort,
}

/// 한 쪽 파일 패널
pub struct Panel {
    pub side: Side,
    pub path_label: nwg::Label,
    pub list: nwg::ListView,
    pub info_label: nwg::Label,
    pub state: RefCell<PanelState>,
}

impl Panel {
    /// 패널 컨트롤들을 주어진 부모(창) 위에 만든다.
    pub fn build(parent: &nwg::Window, side: Side, start: &Path) -> Panel {
        let mut path_label = nwg::Label::default();
        let mut list = nwg::ListView::default();
        let mut info_label = nwg::Label::default();

        nwg::Label::builder()
            .text(&start.to_string_lossy())
            .parent(parent)
            .build(&mut path_label)
            .expect("path label");

        nwg::ListView::builder()
            .parent(parent)
            .list_style(nwg::ListViewStyle::Detailed)
            .ex_flags(nwg::ListViewExFlags::FULL_ROW_SELECT | nwg::ListViewExFlags::GRID)
            .build(&mut list)
            .expect("list view");
        list.set_headers_enabled(true);
        add_columns(&list);

        nwg::Label::builder()
            .text("")
            .parent(parent)
            .build(&mut info_label)
            .expect("info label");

        let panel = Panel {
            side,
            path_label,
            list,
            info_label,
            state: RefCell::new(PanelState {
                cwd: start.to_path_buf(),
                entries: Vec::new(),
                sort: Sort::default(),
            }),
        };
        panel.load(start);
        panel
    }

    /// 주어진 경로를 읽어 패널을 채운다. 실패하면 현재 경로 유지.
    pub fn load(&self, path: &Path) {
        let sort = self.state.borrow().sort;
        let mut entries = match read_local_dir(path) {
            Ok(e) => e,
            Err(e) => {
                nwg::modal_error_message(
                    &self.list,
                    "오류",
                    &format!("디렉터리를 열 수 없습니다:\n{}\n\n{}", path.display(), e),
                );
                return;
            }
        };
        sort_entries(&mut entries, sort);
        self.populate(&entries);
        self.path_label.set_text(&path.to_string_lossy());
        {
            let mut st = self.state.borrow_mut();
            st.cwd = path.to_path_buf();
            st.entries = entries;
        }
        self.update_info();
        if self.list.len() > 0 {
            self.list.select_item(0, true);
        }
    }

    /// 현재 경로를 다시 읽는다.
    pub fn reload(&self) {
        let cwd = self.state.borrow().cwd.clone();
        self.load(&cwd);
    }

    /// 정렬을 적용해 다시 그린다.
    pub fn resort(&self) {
        let sort = self.state.borrow().sort;
        let mut entries = self.state.borrow().entries.clone();
        sort_entries(&mut entries, sort);
        self.populate(&entries);
        self.state.borrow_mut().entries = entries;
    }

    /// ListView 내용을 entries 로 교체한다.
    fn populate(&self, entries: &[Entry]) {
        self.list.set_redraw(false);
        self.list.clear();
        for (i, e) in entries.iter().enumerate() {
            let name = if e.is_dir && !e.is_parent {
                format!("{}\\", e.name)
            } else {
                e.name.clone()
            };
            self.list
                .insert_items_row(Some(i as i32), &[name, e.size_text(), e.time_text()]);
        }
        self.list.set_redraw(true);
    }

    /// 하단 정보 라벨 갱신 (항목 수 / 선택 수)
    pub fn update_info(&self) {
        let total = self.state.borrow().entries.len();
        let sel = self.list.selected_count();
        self.info_label
            .set_text(&format!("{}개 항목 · {}개 선택", total, sel));
    }

    /// 현재 커서가 놓인 항목
    pub fn focused_entry(&self) -> Option<Entry> {
        let idx = self.list.selected_item()?;
        self.state.borrow().entries.get(idx).cloned()
    }

    /// 선택된(체크) 항목들. 없으면 커서 항목 하나.
    pub fn action_entries(&self) -> Vec<Entry> {
        let st = self.state.borrow();
        let sel = self.list.selected_items();
        let mut out: Vec<Entry> = sel
            .iter()
            .filter_map(|&i| st.entries.get(i).cloned())
            .filter(|e| !e.is_parent)
            .collect();
        if out.is_empty() {
            if let Some(idx) = self.list.selected_item() {
                if let Some(e) = st.entries.get(idx) {
                    if !e.is_parent {
                        out.push(e.clone());
                    }
                }
            }
        }
        out
    }

    /// 현재 작업 디렉터리
    pub fn cwd(&self) -> PathBuf {
        self.state.borrow().cwd.clone()
    }

    /// 커서 항목의 전체 경로
    pub fn focused_path(&self) -> Option<PathBuf> {
        let e = self.focused_entry()?;
        Some(self.state.borrow().cwd.join(&e.name))
    }
}

/// ListView 에 3개 컬럼(이름/크기/수정시각)을 만든다.
fn add_columns(list: &nwg::ListView) {
    let cols = [
        ("이름", 240, nwg::ListViewColumnFlags::LEFT),
        ("크기", 90, nwg::ListViewColumnFlags::RIGHT),
        ("수정 시각", 130, nwg::ListViewColumnFlags::LEFT),
    ];
    for (i, (text, width, fmt)) in cols.iter().enumerate() {
        list.insert_column(nwg::InsertListViewColumn {
            index: Some(i as i32),
            fmt: Some(*fmt),
            width: Some(*width),
            text: Some(text.to_string()),
        });
    }
}
