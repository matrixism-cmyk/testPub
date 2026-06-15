# 안녕 커맨더 (Annyeong Commander)

미드나잇 커맨더 스타일의 **듀얼 패널 파일 매니저** — Rust + native-windows-gui 로 만든 Windows 데스크톱 유틸리티.

## 기능

- **듀얼 패널 탐색**: 좌우 ListView 패널, Tab 으로 패널 전환, Enter 진입, Backspace 상위 이동
- **정렬**: 이름 / 크기 / 수정시각 / 확장자 (메뉴 또는 클릭, 같은 키 재선택 시 방향 토글)
- **파일 작업**: F5 복사 · F6 이동 · F7 새 폴더 · F8 삭제 (백그라운드 스레드 + 진행률 표시)
- **뷰어 (F3)**: 텍스트 / 헥스 모드 전환, 읽기 전용
- **편집기 (F4)**: 내장 텍스트 편집기, 저장
- **압축 파일 탐색**: zip 아카이브를 패널에서 폴더처럼 진입 *(개발 중)*
- **원격 탐색**: FTP *(개발 중)*
- **편의 기능**: 명령줄 · 북마크 · 히스토리 *(개발 중)*

## 빌드 & 실행

```sh
cargo run            # 디버그 실행
cargo build --release # 배포용 빌드 (target/release/commander.exe)
```

Windows 전용입니다. 공통 컨트롤 v6 매니페스트는 `build.rs` 에서 자동 임베드됩니다.

### 인스톨러

[Inno Setup](https://jrsoftware.org/isinfo.php) 으로 설치 파일을 만듭니다. (기본 설치 위치: `C:\Program Files\AnnyeongCommander`)

```sh
cargo build --release
ISCC installer\setup.iss     # → installer\AnnyeongCommander-Setup-0.1.0.exe
```

## 단축키

| 키 | 동작 |
|----|------|
| Tab | 패널 전환 |
| Enter | 폴더 진입 / 파일 열기 |
| Backspace | 상위 폴더 |
| F3 / F4 | 보기 / 편집 |
| F5 / F6 | 복사 / 이동 |
| F7 / F8 | 새 폴더 / 삭제 |
| F10 | 종료 |

## 구조

모듈은 파일당 코드 250줄(소프트) / 400줄(하드) 제한을 지켜 분리되어 있습니다.

| 모듈 | 역할 |
|------|------|
| `app` | 앱 조립 + 레이아웃 |
| `ui_panel` / `ui_funcbar` / `menus` | 패널 · 함수키 바 · 메뉴 |
| `events` / `actions` | 이벤트 디스패치 · 동작 |
| `model` / `state` | 파일 모델 · 공통 상태 |
| `fsops` / `fsworker` / `progress` | 파일 작업 · 워커 · 진행 상태 |
| `viewer` / `editor` / `dialogs` | 뷰어 · 편집기 · 다이얼로그 |

## 라이선스

MIT
