; 안녕 커맨더 Inno Setup 스크립트
; 기본 설치 위치: C:\Program Files\AnnyeongCommander
; 컴파일: ISCC.exe installer\setup.iss  (작업 디렉터리 = 프로젝트 루트)

#define MyAppName "안녕 커맨더"
#define MyAppVersion "0.1.0"
#define MyAppPublisher "AEOK"
#define MyAppExeName "commander.exe"

[Setup]
AppId={{B7E4B0B2-9A4D-4C1E-9C2A-AC0D0A4F0E11}}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
; C:\Program Files 아래에 전용 폴더 생성
DefaultDirName={autopf}\AnnyeongCommander
DisableProgramGroupPage=yes
DefaultGroupName={#MyAppName}
UninstallDisplayIcon={app}\{#MyAppExeName}
UninstallDisplayName={#MyAppName}
; 64비트 전용 설치 (Program Files = 64비트 경로)
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
; Program Files 쓰기에는 관리자 권한 필요
PrivilegesRequired=admin
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
OutputDir=.
OutputBaseFilename=AnnyeongCommander-Setup-{#MyAppVersion}

[Languages]
Name: "korean"; MessagesFile: "compiler:Languages\Korean.isl"
Name: "english"; MessagesFile: "compiler:Default.isl"

[Files]
Source: "..\target\release\{#MyAppExeName}"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\README.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\안녕커맨더_사용설명서.docx"; DestDir: "{app}"; Flags: ignoreversion isreadme

[Icons]
Name: "{group}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"
Name: "{group}\사용설명서"; Filename: "{app}\안녕커맨더_사용설명서.docx"
Name: "{group}\{#MyAppName} 제거"; Filename: "{uninstallexe}"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; Tasks: desktopicon

[Tasks]
Name: "desktopicon"; Description: "바탕화면에 바로가기 만들기"; GroupDescription: "추가 아이콘:"; Flags: unchecked

[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "지금 실행"; Flags: nowait postinstall skipifsilent
