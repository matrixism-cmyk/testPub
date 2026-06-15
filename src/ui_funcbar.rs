// 하단 함수키 바 (F1~F10 버튼)
use crate::state::FuncAction;
use native_windows_gui as nwg;

/// 버튼 하나와 그 동작
pub struct FuncButton {
    pub button: nwg::Button,
    pub action: FuncAction,
}

/// 함수키 바 전체
pub struct FuncBar {
    pub buttons: Vec<FuncButton>,
}

/// (라벨, 동작) 정의. MC 의 하단 키 배치를 따른다.
const DEFS: &[(&str, FuncAction)] = &[
    ("F1 도움말", FuncAction::Help),
    ("F3 보기", FuncAction::View),
    ("F4 편집", FuncAction::Edit),
    ("F5 복사", FuncAction::Copy),
    ("F6 이동", FuncAction::Move),
    ("F7 폴더", FuncAction::Mkdir),
    ("F8 삭제", FuncAction::Delete),
    ("F9 메뉴", FuncAction::Menu),
    ("F10 종료", FuncAction::Quit),
];

impl FuncBar {
    /// 함수키 버튼들을 만든다.
    pub fn build(parent: &nwg::Window) -> FuncBar {
        let mut buttons = Vec::with_capacity(DEFS.len());
        for (text, action) in DEFS {
            let mut button = nwg::Button::default();
            nwg::Button::builder()
                .text(text)
                .parent(parent)
                .build(&mut button)
                .expect("func button");
            buttons.push(FuncButton {
                button,
                action: *action,
            });
        }
        FuncBar { buttons }
    }

    /// 버튼 핸들로 동작을 찾는다.
    pub fn action_of(&self, handle: nwg::ControlHandle) -> Option<FuncAction> {
        self.buttons
            .iter()
            .find(|b| b.button.handle == handle)
            .map(|b| b.action)
    }

    /// 버튼 개수
    pub fn len(&self) -> usize {
        self.buttons.len()
    }
}
