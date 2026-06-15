# 안녕 커맨더 사용설명서(.docx) 생성 스크립트 (Word COM 자동화)
$ErrorActionPreference = "Stop"
$out = "C:\Users\master\Desktop\work\commander\안녕커맨더_사용설명서.docx"

$word = New-Object -ComObject Word.Application
$word.Visible = $false
$doc = $word.Documents.Add()
$sel = $word.Selection

# 빌트인 스타일 (언어 독립적인 WdBuiltinStyle 인덱스)
$TITLE  = $doc.Styles.Item(-63)
$SUBTL  = $doc.Styles.Item(-74)
$H1     = $doc.Styles.Item(-2)
$H2     = $doc.Styles.Item(-3)
$NORMAL = $doc.Styles.Item(-1)
$NORMAL.Font.Name = "맑은 고딕"
$NORMAL.Font.Size = 10.5

function Title($t)    { $sel.Style = $TITLE; $sel.TypeText($t); $sel.TypeParagraph() }
function Subtitle($t) { $sel.Style = $SUBTL; $sel.TypeText($t); $sel.TypeParagraph() }
function H1($t)       { $sel.Style = $H1;    $sel.TypeText($t); $sel.TypeParagraph() }
function H2($t)       { $sel.Style = $H2;    $sel.TypeText($t); $sel.TypeParagraph() }
function P($t)        { $sel.Style = $NORMAL; $sel.TypeText($t); $sel.TypeParagraph() }
function Bullet($t)   { $sel.Style = $NORMAL; $sel.TypeText([char]0x2022 + " " + $t); $sel.TypeParagraph() }

function Table2($headers, $rows) {
    $ncols = $headers.Count
    $nrows = $rows.Count + 1
    $rng = $sel.Range
    $tbl = $doc.Tables.Add($rng, $nrows, $ncols)
    $tbl.Borders.Enable = $true
    $tbl.Range.Style = $NORMAL
    $tbl.Range.Font.Size = 10
    for ($c = 0; $c -lt $ncols; $c++) { $tbl.Cell(1, $c + 1).Range.Text = [string]$headers[$c] }
    for ($r = 0; $r -lt $rows.Count; $r++) {
        for ($c = 0; $c -lt $ncols; $c++) { $tbl.Cell($r + 2, $c + 1).Range.Text = [string]$rows[$r][$c] }
    }
    $tbl.Rows.Item(1).Range.Bold = $true
    $tbl.Rows.Item(1).Shading.BackgroundPatternColor = 15921906
    $word.Selection.EndKey(6) | Out-Null
    $word.Selection.TypeParagraph()
}

# ===== 표지 =====
Title "안녕 커맨더 사용설명서"
Subtitle "Annyeong Commander - 미드나잇 커맨더 스타일 듀얼 패널 파일 매니저"
P "버전 0.1.0   |   Windows 데스크톱 유틸리티   |   Rust + native-windows-gui"
P ""

# ===== 1. 소개 =====
H1 "1. 소개"
P "안녕 커맨더는 고전 파일 관리자 '미드나잇 커맨더(Midnight Commander)'의 사용 방식을 본떠 만든 Windows용 듀얼 패널 파일 매니저입니다. 화면을 좌·우 두 개의 패널로 나누어, 한쪽에서 다른 쪽으로 파일을 복사·이동하는 작업을 키보드 중심으로 빠르게 수행할 수 있습니다."
P "주요 특징은 다음과 같습니다."
Bullet "좌·우 듀얼 패널과 키보드 중심 조작 (Tab 전환, 방향키 이동, 함수키 명령)"
Bullet "복사·이동·삭제·새 폴더 만들기 (대용량 작업은 백그라운드 스레드 + 진행률 표시)"
Bullet "내장 파일 뷰어(F3, 텍스트/헥스)와 텍스트 편집기(F4)"
Bullet "ZIP 압축 파일을 일반 폴더처럼 열람하고 추출"
Bullet "FTP 원격 서버 접속 및 다운로드/업로드"
Bullet "하단 명령줄, 북마크, 폴더 히스토리, 설정 자동 저장"

# ===== 2. 설치 및 실행 =====
H1 "2. 설치 및 실행"
H2 "2.1 시스템 요구사항"
Bullet "운영체제: Windows 10 / 11 (64비트)"
Bullet "추가 런타임 불필요 (단일 실행 파일). 공통 컨트롤 v6 매니페스트가 실행 파일에 내장되어 있습니다."
H2 "2.2 실행 방법"
P "배포용 실행 파일 commander.exe 를 더블클릭하면 바로 실행됩니다. 개발 환경에서는 프로젝트 폴더에서 다음 명령으로 실행/빌드할 수 있습니다."
Bullet "cargo run  : 디버그 실행"
Bullet "cargo build --release  : 배포용 빌드 (target\release\commander.exe 생성)"
H2 "2.3 설치 관리자"
P "Inno Setup 으로 만든 설치 관리자를 통해 기본적으로 C:\Program Files 아래 전용 폴더에 설치할 수 있습니다. (설치 시 시작 메뉴 바로가기 생성)"

# ===== 3. 화면 구성 =====
H1 "3. 화면 구성"
P "프로그램 창은 위에서부터 아래로 다음 요소로 구성됩니다."
Bullet "메뉴 바: 파일 / 명령 / 정렬 / 연결 / 설정 / 도움말"
Bullet "좌·우 패널: 각 패널은 경로 표시줄, 파일 목록(이름·크기·수정 시각 3개 컬럼), 항목 수 정보줄로 구성"
Bullet "명령줄: 활성 폴더 기준으로 명령을 입력해 실행 (Enter)"
Bullet "함수키 바: F1~F10 동작 버튼 (마우스 클릭으로도 실행 가능)"
Bullet "상태줄: 활성 패널 위치와 현재 커서 항목 표시"
P "창 크기는 자유롭게 조절할 수 있으며, 모든 요소가 자동으로 재배치됩니다."

# ===== 4. 기본 사용법 =====
H1 "4. 기본 사용법"
H2 "4.1 패널 전환과 이동"
Bullet "Tab 키 또는 마우스 클릭으로 활성 패널을 전환합니다."
Bullet "방향키(↑↓), PgUp/PgDn, Home/End 로 항목 사이를 이동합니다."
Bullet "Enter 로 폴더·압축 파일에 들어가고, 일반 파일은 연결된 기본 프로그램으로 엽니다."
Bullet "Backspace 또는 목록 맨 위의 '..' 항목으로 상위 폴더로 올라갑니다."
Bullet "Ctrl+클릭 / Shift+클릭 으로 여러 항목을 동시에 선택할 수 있습니다."
H2 "4.2 정렬"
P "'정렬' 메뉴에서 이름순 / 크기순 / 수정시각순 / 확장자순 을 고를 수 있습니다. 같은 기준을 다시 선택하면 오름차순 ↔ 내림차순이 토글됩니다. 폴더는 항상 파일보다 위에, '..' 항목은 항상 맨 위에 표시됩니다."

# ===== 5. 단축키 =====
H1 "5. 단축키"
Table2 @("키", "동작") @(
    @("Tab", "좌/우 패널 전환"),
    @("Enter", "폴더·압축 진입 / 파일 열기"),
    @("Backspace", "상위 폴더로 이동"),
    @("방향키 / PgUp·PgDn / Home·End", "항목 커서 이동"),
    @("Ctrl·Shift + 클릭", "다중 선택"),
    @("F1", "도움말"),
    @("F3", "보기 (뷰어: 텍스트/헥스)"),
    @("F4", "편집 (내장 편집기)"),
    @("F5", "복사 / 압축 추출 / 원격 다운로드·업로드"),
    @("F6", "이동 (이름 변경)"),
    @("F7", "새 폴더"),
    @("F8", "삭제"),
    @("F10", "종료")
)

# ===== 6. 파일 작업 =====
H1 "6. 파일 작업"
P "파일 작업은 '활성 패널에서 선택한 항목'을 대상으로 하며, 복사·이동의 목적지는 반대쪽(비활성) 패널의 폴더입니다."
Bullet "F5 복사: 선택 항목을 반대쪽 패널 폴더로 복사합니다."
Bullet "F6 이동: 선택 항목을 반대쪽 패널 폴더로 이동합니다. (같은 드라이브는 즉시 이동, 다른 드라이브는 복사 후 삭제)"
Bullet "F7 새 폴더: 이름을 입력받아 활성 폴더에 새 폴더를 만듭니다."
Bullet "F8 삭제: 선택 항목을 삭제합니다. (되돌릴 수 없으므로 확인 창이 표시됩니다)"
P "복사/이동/삭제는 백그라운드 스레드에서 수행되어 작업 중에도 창이 멈추지 않으며, 하단 상태줄에 진행률(%)과 현재 처리 중인 파일명이 표시됩니다. 작업이 끝나면 양쪽 패널이 자동으로 새로고침됩니다. 하위 폴더까지 재귀적으로 처리합니다."

# ===== 7. 보기와 편집 =====
H1 "7. 보기와 편집"
H2 "7.1 보기 (F3)"
P "커서가 놓인 파일을 읽기 전용 뷰어로 엽니다. 상단의 '텍스트/헥스 전환' 버튼으로 보기 모드를 바꿀 수 있습니다. 텍스트 모드는 내용을 그대로, 헥스 모드는 16진수 덤프(오프셋·바이트·아스키)를 보여줍니다. 최대 4MB까지 표시합니다."
H2 "7.2 편집 (F4)"
P "커서가 놓인 파일을 내장 텍스트 편집기로 엽니다. 내용을 수정한 뒤 '저장(Ctrl+S)' 버튼으로 파일에 다시 씁니다. (최대 8MB) 저장 시 줄바꿈은 LF로 정규화됩니다."

# ===== 8. 압축 파일 =====
H1 "8. 압축 파일(ZIP) 탐색"
P "파일 목록에서 .zip 파일에 커서를 두고 Enter 를 누르면, 압축 파일 내부를 일반 폴더처럼 탐색할 수 있습니다. 내부 폴더로 진입하거나 '..' 로 빠져나올 수 있고, 압축 루트에서 '..' 를 누르면 다시 로컬 폴더로 돌아옵니다."
P "압축 내부에서 항목을 선택하고 F5(복사) 를 누르면, 반대쪽 패널의 로컬 폴더로 추출됩니다. 추출 트리는 현재 보고 있던 위치를 기준으로 구성됩니다."

# ===== 9. FTP 원격 =====
H1 "9. FTP 원격 접속"
P "'연결 > FTP 연결...' 메뉴를 선택하면 호스트, 사용자 이름, 비밀번호를 차례로 입력받아 FTP 서버에 접속합니다. 호스트는 'host' 또는 'host:port' 형식으로 입력하며, 포트를 생략하면 21번을 사용합니다. 연결되면 활성 패널이 원격 위치(ftp://...)로 전환됩니다."
Bullet "원격 폴더는 로컬과 동일하게 방향키·Enter·Backspace 로 탐색합니다."
Bullet "원격에서 파일을 선택하고 F5 를 누르면 반대쪽 로컬 폴더로 다운로드합니다."
Bullet "로컬에서 파일을 선택하고 반대쪽이 원격일 때 F5 를 누르면 원격으로 업로드합니다."
Bullet "'연결 > 연결 끊기' 또는 원격 루트에서 '..' 를 누르면 연결을 종료하고 로컬 홈으로 돌아옵니다."
P "참고: 현재 원격 전송은 파일 단위로 지원되며(폴더 일괄 전송 제외), 전송 중에는 창이 잠시 대기할 수 있습니다."

# ===== 10. 명령줄 =====
H1 "10. 명령줄"
P "함수키 바 위의 입력란에 명령을 입력하고 Enter 를 누르면, 활성 패널의 현재 폴더를 작업 디렉터리로 하여 'cmd /C <명령>' 형태로 실행합니다. 실행 후 패널이 자동 새로고침됩니다."

# ===== 11. 북마크·히스토리·설정 =====
H1 "11. 북마크 · 히스토리 · 설정"
Bullet "북마크 추가: '설정 > 북마크 추가'로 현재 활성 폴더를 즐겨찾기에 등록합니다."
Bullet "북마크 열기: '설정 > 북마크 열기...'에서 목록을 골라 해당 폴더로 이동합니다."
Bullet "히스토리 뒤로: '파일 > 뒤로'로 직전에 머물던 위치로 되돌아갑니다."
Bullet "설정 저장: 프로그램 종료 시 양쪽 패널의 폴더 위치와 북마크가 자동 저장되어, 다음 실행 때 복원됩니다."
P "설정 파일 위치는 '설정 > 환경설정'에서 확인할 수 있습니다. (%APPDATA%\AnnyeongCommander\config.toml)"

# ===== 12. 메뉴 구조 =====
H1 "12. 메뉴 구조"
Table2 @("메뉴", "항목") @(
    @("파일", "새로고침 · 뒤로(이전 폴더) · 종료"),
    @("명령", "보기 · 편집 · 복사 · 이동 · 새 폴더 · 삭제"),
    @("정렬", "이름순 · 크기순 · 수정시각순 · 확장자순"),
    @("연결", "FTP 연결... · 연결 끊기"),
    @("설정", "북마크 추가 · 북마크 열기... · 환경설정"),
    @("도움말", "정보")
)

# ===== 13. 참고사항 =====
H1 "13. 참고사항"
Bullet "삭제는 휴지통을 거치지 않고 즉시 삭제되므로 주의하세요."
Bullet "압축/원격 위치에서는 새 폴더 만들기·삭제가 제한됩니다."
Bullet "한 번에 하나의 백그라운드 파일 작업만 실행됩니다. (작업 중 새 작업 시 안내)"

# ===== 14. 제작 정보 =====
H1 "14. 제작 정보"
P "안녕 커맨더는 Rust 언어와 native-windows-gui 라이브러리로 작성되었습니다. 소스 코드는 모듈별로 분리되어 있으며(파일당 코드 250줄 소프트 / 400줄 하드 제한 준수), 라이선스는 MIT 입니다."
P "GitHub: https://github.com/matrixism-cmyk/testPub"

# 저장
$doc.SaveAs2($out)
$doc.Close()
$word.Quit()
[System.Runtime.InteropServices.Marshal]::ReleaseComObject($sel) | Out-Null
[System.Runtime.InteropServices.Marshal]::ReleaseComObject($doc) | Out-Null
[System.Runtime.InteropServices.Marshal]::ReleaseComObject($word) | Out-Null
"SAVED: $out"
