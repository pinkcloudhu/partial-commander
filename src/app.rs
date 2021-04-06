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

    pub fn is_dirs_only(&self) -> bool {
        self.dirs_only
    }

    fn strip_path_string(&self, path: &Path) -> String {
        if let Some(p) = path.to_str() {
            let s = p.split(PATH_SEAPARATOR).collect::<Vec<&str>>();
            let result = String::from(s[s.len() - 1]);
            if let Ok(m) = path.metadata() {
                if m.is_dir() {
                    return String::from(result + PATH_SEAPARATOR)
                } else {
                    return result
                }
            }
        }
        return String::from("???");
    }

    fn list_folder_inner(&self, path: &Path) -> Result<Vec<PathBuf>, ErrorKind> {
        if let Ok(m) = path.metadata() {
            if !m.is_dir() { return Err(ErrorKind::InvalidInput) }
        }
        let mut result = vec!();
        for path in path.read_dir() {
            for item in path {
                if let Ok(item) = item {
                    let p = item.path();
                    if let Ok(m) = p.as_path().metadata() {
                        if !(self.dirs_only && !m.is_dir()) {
                            result.push(p);
                        }
                    }
                }
            }
        }
        return Ok(result);
    }

    fn list_folder_inner_str(&self, path: &Path) -> Vec<String> {
        let mut result = vec!();
        if let Ok(paths) = self.list_folder_inner(path) {
            result =  paths.iter().map(|p| self.strip_path_string(p)).collect();
        }
        return result;
    }

    pub fn list_folder_str(&self) -> Vec<String> {
        self.list_folder_inner_str(self.cwd.as_path())
    }

    pub fn list_parent_str(&self) -> Vec<String> {
        if let Some(parent) = self.cwd.parent() {
            self.list_folder_inner_str(parent)
        } else {
            Vec::new()
        }
    }

    pub fn current_folder(&self) -> String {
        self.strip_path_string(self.cwd.as_path())
    }

    pub fn parent_folder(&self) -> String {
        if let Some(parent) = self.cwd.parent() {
            self.strip_path_string(parent)
        } else { return String::from("???/"); }
    }

    pub fn list_child_str(&self, idx: usize) -> Result<Vec<String>, ErrorKind> {
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
                    return Ok(self.list_folder_inner_str((&current[idx]).path().as_path()));
                }
            }
        }
        return Err(ErrorKind::NotFound);
    }

    pub fn current_folder_parent_idx(&self) -> Option<usize> {
        let parent = self.list_parent_str();
        let curr = self.current_folder();
        for (i, item) in parent.iter().enumerate() {
            if item == &curr {
                return Some(i);
            }
        }
        None
    }

    pub fn child_is_folder(&self, idx: usize) -> bool {
        if self.dirs_only { return true; }

        if let Ok(mut curr) = self.cwd.as_path().read_dir() {
            if let Some(Ok(item)) = curr.nth(idx) {
                if let Ok(m) = item.metadata() {
                    return m.is_dir();
                }
            }
        }
        return false;
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

    pub fn down(&mut self, idx: usize) -> Result<Vec<String>, ErrorKind> {
        let parent = self.list_folder_str();
        if parent.len() == 0 {
            return Err(ErrorKind::NotFound);
        }
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
            if let Ok(m) = (&children[idx]).metadata() {
                if !m.is_dir() {
                    return Err(ErrorKind::InvalidInput)
                }
            }

            self.cwd = (&children[idx]).path();
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
                    if let Ok(_) = handle.read(buf) {
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
