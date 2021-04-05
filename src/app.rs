use std::fs::DirEntry;
use std::io::ErrorKind;
use std::{
    env::current_dir,
    path::{Path, PathBuf},
};

#[cfg(target_family = "unix")]
pub const PATH_SEAPARATOR: &str = "/";

#[cfg(target_family = "windows")]
pub const PATH_SEAPARATOR: &str = "\\";

pub struct App {
    cwd: PathBuf,
    history: Vec<PathBuf>,
    dirs_only: bool,
}
impl App {
    pub fn new(path: Option<String>, dirs_only: bool) -> Self {
        let path = if let Some(s) = path {
            if Path::new(&s).exists() {
                PathBuf::from(s)
            } else {
                current_dir().unwrap()
            }
        } else {
            current_dir().unwrap()
        };
        App {
            cwd: path,
            history: vec![],
            dirs_only,
        }
    }

    fn strip_path_string(&self, s: &str, is_dir: bool) -> String {
        let s = s.split(PATH_SEAPARATOR).collect::<Vec<&str>>();
        let result = String::from(s[s.len() - 1]);
        if is_dir {
            String::from(result + PATH_SEAPARATOR)
        } else {
            result
        }
    }

    fn list_folder_inner(&self, path: &Path) -> Vec<String> {
        let mut result = Vec::new();
        for path in path.read_dir() {
            for item in path {
                if let Ok(item) = item {
                    if let Ok(m) = item.metadata() {
                        if !(self.dirs_only && !m.is_dir()) {
                            if let Some(s) = item.path().to_str() {
                                result.push(self.strip_path_string(s, m.is_dir()));
                            } else {
                                result.push(String::from("??? unknown"));
                            }
                        }
                    }
                } else {
                    result.push(String::from("??? unknown"));
                }
            }
        }
        result
    }

    pub fn list_folder(&self) -> Vec<String> {
        self.list_folder_inner(self.cwd.as_path())
    }

    pub fn list_parent(&self) -> Vec<String> {
        if let Some(parent) = self.cwd.parent() {
            self.list_folder_inner(parent)
        } else {
            Vec::new()
        }
    }

    pub fn current_folder(&self) -> String {
        if let Some(s) = self.cwd.as_os_str().to_str() {
            self.strip_path_string(s, true)
        } else {
            String::from("???")
        }
    }

    pub fn parent_folder(&self) -> String {
        if let Some(p) = self.cwd.parent() {
            if let Some(s) = p.to_path_buf().as_os_str().to_str() {
                self.strip_path_string(s, true)
            } else {
                String::from("???")
            }
        } else {
            String::from("???")
        }
    }

    pub fn current_folder_parent_idx(&self) -> Option<usize> {
        let parent = self.list_parent();
        let curr = self.current_folder();
        for (i, item) in parent.iter().enumerate() {
            if item == &curr {
                return Some(i);
            }
        }
        None
    }

    pub fn up(&mut self, selected_idx: Option<usize>) -> Result<(), ()> {
        if let Some(parent) = self.cwd.parent() {
            if let Some(idx) = selected_idx {
                if let Ok(mut items) = self.cwd.read_dir() {
                    if let Some(Ok(item)) = items.nth(idx) {
                        self.history.push(item.path());
                    }
                }
            }
            self.cwd = parent.to_path_buf();
        } else {
            return Err(());
        }
        return Ok(());
    }

    pub fn child_is_folder(&self, idx: usize) -> bool {
        if self.dirs_only {
            return true;
        }
        if let Ok(mut curr) = self.cwd.as_path().read_dir() {
            if let Some(Ok(item)) = curr.nth(idx) {
                if let Ok(m) = item.metadata() {
                    return m.is_dir();
                } else {
                    return false;
                }
            } else {
                return false;
            }
        } else {
            return false;
        }
    }

    pub fn list_child(&self, idx: usize) -> Result<Vec<String>, ErrorKind> {
        if self.child_is_folder(idx) {
            if let Ok(curr) = self.cwd.as_path().read_dir() {
                let mut current: Vec<DirEntry> = vec![];
                for dir in curr {
                    if let Ok(item) = dir {
                        if self.dirs_only {
                            if let Ok(m) = item.path().metadata() {
                                if m.is_dir() {
                                    current.push(item);
                                }
                            }
                        } else {
                            current.push(item);
                        }
                    }
                }
                if current.len() == 0 {
                    return Err(ErrorKind::NotFound);
                } else {
                    return Ok(self.list_folder_inner((&current[idx]).path().as_path()));
                }
            }
        }
        return Err(ErrorKind::NotFound);
    }

    pub fn down(&mut self, idx: usize) -> Result<Vec<String>, ErrorKind> {
        let parent = self.list_folder();
        if let Ok(read_dir) = self.cwd.as_path().read_dir() {
            let mut children: Vec<DirEntry> = vec![];
            for dir in read_dir {
                if let Ok(item) = dir {
                    if self.dirs_only {
                        if let Ok(m) = item.path().metadata() {
                            if m.is_dir() {
                                children.push(item);
                            }
                        }
                    } else {
                        children.push(item);
                    }
                }
            }
            if children.len() == 0 {
                return Err(ErrorKind::NotFound);
            }

            self.cwd = (&children[idx]).path();
            if self.list_folder().len() == 0 {
                self.cwd = self.cwd.parent().expect("We just moved down, this should exist").to_path_buf();
                return Err(ErrorKind::NotFound);
            }
        }
        return Ok(parent);
    }

    pub fn pop_last_visited_idx(&mut self) -> Option<usize> {
        if let Some(last_item) = self.history.pop() {
            if let Ok(dir) = self.cwd.as_path().read_dir() {
                for (i, item) in dir.enumerate() {
                    if let Ok(item) = item {
                        if item.path() == last_item {
                            return Some(i);
                        }
                    }
                }
            }
        }
        self.history = vec![];
        None
    }

    pub fn current_path(&self) -> &Path {
        return self.cwd.as_path();
    }

    pub fn is_dirs_only(&self) -> bool {
        self.dirs_only
    }

    pub fn read_child_file(&self, idx: usize) -> Option<Vec<String>> {
        use std::fs::File;
        use std::io::{self, Read, BufRead, Seek, SeekFrom};
        use content_inspector::{inspect, ContentType};
        use itertools::Itertools;

        if self.child_is_folder(idx) { return None }

        // First, this hunk of mess determines if a file is text, so it can be displayed on-screen
        // based on the first 512 bytes of it.
        // Then, it seeks to the start of the file and reads 20 lines and sends it off for displaying

        if let Ok(mut curr) = self.cwd.as_path().read_dir() {
            if let Some(Ok(child)) = curr.nth(idx) {
                let child_path = child.path();
                if let Ok(mut handle) = File::open(child_path) {
                    let buf: &mut [u8] = &mut [0; 512];
                    if let Ok(n) = handle.read(buf) {
                        handle.seek(SeekFrom::Start(0)).expect("Should be able to seek to the start of a stream");
                        if inspect(buf) != ContentType::BINARY {
                            let mut s = vec!();
                            for lines in &io::BufReader::new(handle).lines().chunks(20) {
                                for (_, line) in lines.enumerate() {
                                    if let Ok(line) = line {
                                        s.push(line);
                                    }
                                }
                            }
                            return Some(s);
                        }
                    }
                }
            }
        }
        None
    }
}
