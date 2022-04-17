use std::path::PathBuf;

pub struct PathIter{
    buf: PathBuf,
    end: bool,
    last: PathBuf,
}

impl PathIter {
    pub fn new(path: PathBuf) -> Self{
        Self{
            buf: path,
            end: false,
            last: PathBuf::from("/"),
        }
    }
    /*pub fn current() -> std::io::Result<Self> {
        let buf = std::env::current_dir()?;
        Ok(Self::new(buf))
    }*/
}

impl Iterator for PathIter {
    type Item = PathBuf;
    fn next(&mut self) -> Option<Self::Item> {
        loop{
            if self.end { return None }
            let mut ret = self.buf.clone();
            if let Some("/") = ret.to_str() {
                ret = PathBuf::from("/etc")
            }
            self.end = !self.buf.pop();
            if ret == self.last {
                continue
            } else {
                self.last = ret.clone();
            }
            return Some(ret)
        }
    }
}
