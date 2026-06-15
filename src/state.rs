// 앱 전반에서 쓰이는 공통 상태/열거 타입
use std::path::PathBuf;

/// 활성 패널 구분 (왼쪽/오른쪽)
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Side {
    Left,
    Right,
}

impl Side {
    pub fn other(self) -> Side {
        match self {
            Side::Left => Side::Right,
            Side::Right => Side::Left,
        }
    }
}

/// 정렬 기준
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum SortKey {
    Name,
    Size,
    Modified,
    Ext,
}

/// 정렬 방향
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum SortOrder {
    Asc,
    Desc,
}

impl SortOrder {
    pub fn toggled(self) -> SortOrder {
        match self {
            SortOrder::Asc => SortOrder::Desc,
            SortOrder::Desc => SortOrder::Asc,
        }
    }
}

/// 현재 정렬 설정
#[derive(Copy, Clone, Debug)]
pub struct Sort {
    pub key: SortKey,
    pub order: SortOrder,
}

impl Default for Sort {
    fn default() -> Self {
        Sort {
            key: SortKey::Name,
            order: SortOrder::Asc,
        }
    }
}

/// 함수키바 동작 식별자 (F1~F10)
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum FuncAction {
    Help,    // F1
    View,    // F3
    Edit,    // F4
    Copy,    // F5
    Move,    // F6
    Mkdir,   // F7
    Delete,  // F8
    Menu,    // F9
    Quit,    // F10
}

/// 기본 시작 경로 (사용자 홈, 없으면 C 드라이브)
pub fn default_start_dir() -> PathBuf {
    directories::UserDirs::new()
        .map(|d| d.home_dir().to_path_buf())
        .unwrap_or_else(|| PathBuf::from("C:\\"))
}
