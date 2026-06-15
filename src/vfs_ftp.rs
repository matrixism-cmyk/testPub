// FTP 원격 세션: 연결 / 목록 / 다운로드 / 업로드
use crate::model::Entry;
use std::fs::File;
use std::io;
use std::path::Path;
use suppaftp::FtpStream;

/// 살아있는 FTP 연결 + 표시용 호스트
pub struct FtpSession {
    stream: FtpStream,
    #[allow(dead_code)] // 표시용으로 보관 (Location 이 별도 호스트 라벨 보유)
    pub host: String,
}

impl FtpSession {
    /// 서버에 접속 후 로그인
    pub fn connect(host: &str, port: u16, user: &str, pass: &str) -> io::Result<FtpSession> {
        let mut stream = FtpStream::connect((host, port)).map_err(ftp_io)?;
        stream.login(user, pass).map_err(ftp_io)?;
        Ok(FtpSession {
            stream,
            host: host.to_string(),
        })
    }

    /// 원격 디렉터리 목록 (".." 포함, "."/".." 항목 제거)
    pub fn list(&mut self, path: &str) -> io::Result<Vec<Entry>> {
        self.stream.cwd(path).map_err(ftp_io)?;
        let lines = self.stream.list(None).map_err(ftp_io)?;
        let mut out = vec![Entry::parent()];
        for line in lines {
            if let Ok(f) = line.parse::<suppaftp::list::File>() {
                let name = f.name().to_string();
                if name == "." || name == ".." {
                    continue;
                }
                out.push(Entry {
                    name,
                    is_dir: f.is_directory(),
                    is_parent: false,
                    size: f.size() as u64,
                    modified: Some(f.modified()),
                });
            }
        }
        Ok(out)
    }

    /// 원격 파일을 로컬로 다운로드
    pub fn download(&mut self, dir: &str, name: &str, local: &Path) -> io::Result<()> {
        self.stream.cwd(dir).map_err(ftp_io)?;
        let mut cur = self.stream.retr_as_buffer(name).map_err(ftp_io)?;
        let mut f = File::create(local)?;
        io::copy(&mut cur, &mut f)?;
        Ok(())
    }

    /// 로컬 파일을 원격으로 업로드
    pub fn upload(&mut self, dir: &str, local: &Path, name: &str) -> io::Result<()> {
        self.stream.cwd(dir).map_err(ftp_io)?;
        let mut f = File::open(local)?;
        self.stream.put_file(name, &mut f).map_err(ftp_io)?;
        Ok(())
    }
}

impl Drop for FtpSession {
    fn drop(&mut self) {
        let _ = self.stream.quit();
    }
}

fn ftp_io<E: std::fmt::Display>(e: E) -> io::Error {
    io::Error::new(io::ErrorKind::Other, format!("FTP: {}", e))
}
