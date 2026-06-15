// 백그라운드 파일 작업의 공유 진행 상태
use std::sync::{Arc, Mutex};

/// 작업 스레드와 UI 스레드가 공유하는 진행 상태
#[derive(Default)]
pub struct Progress {
    pub verb: String,       // "복사" / "이동" / "삭제"
    pub total: u64,         // 전체 바이트(또는 항목 수)
    pub done: u64,          // 처리된 양
    pub current: String,    // 현재 처리 중인 파일명
    pub finished: bool,     // 작업 종료 여부
    pub cancel: bool,       // 취소 요청
    pub error: Option<String>,
}

impl Progress {
    pub fn percent(&self) -> u64 {
        if self.total == 0 {
            if self.finished {
                100
            } else {
                0
            }
        } else {
            (self.done.min(self.total)) * 100 / self.total
        }
    }
}

/// 스레드 간 공유 핸들
pub type SharedProgress = Arc<Mutex<Progress>>;

/// verb 로 초기화된 공유 진행 상태 생성
pub fn new_shared(verb: &str) -> SharedProgress {
    Arc::new(Mutex::new(Progress {
        verb: verb.to_string(),
        ..Default::default()
    }))
}
