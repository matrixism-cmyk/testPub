// Windows 공통 컨트롤 v6 활성화 매니페스트를 실행파일에 임베드한다.
// (ListView 등이 사용하는 GetWindowSubclass 로드를 위해 필수)
use embed_manifest::{embed_manifest, new_manifest};

fn main() {
    if std::env::var_os("CARGO_CFG_WINDOWS").is_some() {
        embed_manifest(new_manifest("AnnyeongCommander"))
            .expect("매니페스트 임베드 실패");
    }
    println!("cargo:rerun-if-changed=build.rs");
}
