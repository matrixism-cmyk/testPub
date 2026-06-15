// 설정/북마크/히스토리 영속화 + 북마크 UI
use crate::app::App;
use crate::dialogs;
use crate::vfs::Location;
use native_windows_gui as nwg;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::rc::Rc;

/// 디스크에 저장되는 설정
#[derive(Serialize, Deserialize, Default, Clone)]
pub struct Config {
    pub left_dir: Option<String>,
    pub right_dir: Option<String>,
    #[serde(default)]
    pub bookmarks: Vec<String>,
}

/// 설정 파일 경로 (%APPDATA%\AnnyeongCommander\config.toml)
fn config_path() -> Option<PathBuf> {
    directories::ProjectDirs::from("", "", "AnnyeongCommander")
        .map(|d| d.config_dir().join("config.toml"))
}

/// 설정 로드 (없으면 기본값)
pub fn load() -> Config {
    let path = match config_path() {
        Some(p) => p,
        None => return Config::default(),
    };
    match std::fs::read_to_string(&path) {
        Ok(s) => toml::from_str(&s).unwrap_or_default(),
        Err(_) => Config::default(),
    }
}

/// 설정 저장
pub fn save(cfg: &Config) {
    let path = match config_path() {
        Some(p) => p,
        None => return,
    };
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(s) = toml::to_string_pretty(cfg) {
        let _ = std::fs::write(&path, s);
    }
}

/// 종료 시 현재 로컬 디렉터리를 저장
pub fn save_dirs(app: &Rc<App>) {
    {
        let mut cfg = app.config.borrow_mut();
        if let Some(d) = app.left.local_dir() {
            cfg.left_dir = Some(d.to_string_lossy().to_string());
        }
        if let Some(d) = app.right.local_dir() {
            cfg.right_dir = Some(d.to_string_lossy().to_string());
        }
    }
    save(&app.config.borrow());
}

/// 활성 패널의 현재 로컬 폴더를 북마크에 추가
pub fn add_bookmark(app: &Rc<App>) {
    let dir = match app.active_panel().local_dir() {
        Some(d) => d.to_string_lossy().to_string(),
        None => {
            nwg::modal_info_message(&app.window, "북마크", "로컬 폴더만 북마크할 수 있습니다.");
            return;
        }
    };
    {
        let mut cfg = app.config.borrow_mut();
        if !cfg.bookmarks.contains(&dir) {
            cfg.bookmarks.push(dir.clone());
        }
    }
    save(&app.config.borrow());
    nwg::modal_info_message(&app.window, "북마크", &format!("추가됨:\n{}", dir));
}

/// 북마크 목록에서 골라 활성 패널을 이동
pub fn open_bookmarks(app: &Rc<App>) {
    let items = app.config.borrow().bookmarks.clone();
    if items.is_empty() {
        nwg::modal_info_message(&app.window, "북마크", "저장된 북마크가 없습니다.\n설정 > 북마크 추가 로 등록하세요.");
        return;
    }
    if let Some(i) = dialogs::choose("북마크 이동", items.clone()) {
        if let Some(path) = items.get(i) {
            app.active_panel().load(Location::Local(PathBuf::from(path)));
            app.update_status();
        }
    }
}

/// 환경설정: 설정 파일 위치/현황 표시
pub fn open_prefs(app: &Rc<App>) {
    let path = config_path()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| "(알 수 없음)".to_string());
    let count = app.config.borrow().bookmarks.len();
    nwg::modal_info_message(
        &app.window,
        "환경설정",
        &format!("설정 파일:\n{}\n\n북마크 {}개\n패널 폴더는 종료 시 자동 저장됩니다.", path, count),
    );
}
