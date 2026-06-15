// 단일 파일 패널: 경로 라벨 + ListView + 정보 라벨 (Location 기반)
use crate::model::{sort_entries, Entry};
use crate::state::{Side, Sort};
use crate::vfs::Location;
use native_windows_gui as nwg;
use std::cell::RefCell;
use std::path::PathBuf;

/// 패널의 가변 상태
pub struct PanelState {
    pub loc: Location,
    pub entries: Vec<Entry>,
    pub sort: Sort,
}

/// 한 쪽 파일 패널
pub struct Panel {
    #[allow(dead_code)] // 패널 식별용(향후 사용)
    pub side: Side,
    pub path_label: nwg::Label,
    pub list: nwg::ListView,
    pub info_label: nwg::Label,
    pub state: RefCell<PanelState>,
    pub history: RefCell<Vec<Location>>,
}

impl Panel {
    /// 패널 컨트롤들을 주어진 부모(창) 위에 만든다.
    pub fn build(parent: &nwg::Window, side: Side, start: Location) -> Panel {
        let mut path_label = nwg::Label::default();
        let mut list = nwg::ListView::default();
        let mut info_label = nwg::Label::default();

        nwg::Label::builder()
            .text(&start.display())
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
                loc: start.clone(),
                entries: Vec::new(),
                sort: Sort::default(),
            }),
            history: RefCell::new(Vec::new()),
        };
        panel.set_location(start);
        panel
    }

    /// 새 위치로 이동 (현재 위치를 히스토리에 기록)
    pub fn load(&self, loc: Location) {
        let cur = self.state.borrow().loc.clone();
        self.history.borrow_mut().push(cur);
        if self.history.borrow().len() > 100 {
            self.history.borrow_mut().remove(0);
        }
        self.set_location(loc);
    }

    /// 히스토리에서 이전 위치로 되돌아간다 (기록하지 않음)
    pub fn go_back(&self) {
        let prev = self.history.borrow_mut().pop();
        if let Some(loc) = prev {
            self.set_location(loc);
        }
    }

    /// 주어진 위치를 읽어 패널을 채운다. 실패하면 현재 위치 유지.
    fn set_location(&self, loc: Location) {
        let sort = self.state.borrow().sort;
        let mut entries = match loc.list() {
            Ok(e) => e,
            Err(e) => {
                nwg::modal_error_message(
                    &self.list,
                    "오류",
                    &format!("열 수 없습니다:\n{}\n\n{}", loc.display(), e),
                );
                return;
            }
        };
        sort_entries(&mut entries, sort);
        self.populate(&entries);
        self.path_label.set_text(&loc.display());
        {
            let mut st = self.state.borrow_mut();
            st.loc = loc;
            st.entries = entries;
        }
        self.update_info();
        if self.list.len() > 0 {
            self.list.select_item(0, true);
        }
    }

    /// 현재 위치를 다시 읽는다.
    pub fn reload(&self) {
        let loc = self.location();
        self.set_location(loc);
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

    /// 선택된 항목들. 없으면 커서 항목 하나. (".." 제외)
    pub fn action_entries(&self) -> Vec<Entry> {
        let st = self.state.borrow();
        let mut out: Vec<Entry> = self
            .list
            .selected_items()
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

    /// 현재 위치(복제)
    pub fn location(&self) -> Location {
        self.state.borrow().loc.clone()
    }

    /// 로컬 디렉터리면 그 경로
    pub fn local_dir(&self) -> Option<PathBuf> {
        self.state.borrow().loc.as_local().cloned()
    }

    /// 커서 항목의 로컬 전체 경로 (로컬 위치에서만)
    pub fn focused_path(&self) -> Option<PathBuf> {
        let e = self.focused_entry()?;
        if e.is_parent {
            return None;
        }
        Some(self.local_dir()?.join(&e.name))
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
