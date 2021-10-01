use core::fmt::Formatter;
use std::error::Error;
use std::{
    env::current_dir,
    path::{Path, PathBuf},
};

#[cfg(target_family = "unix")]
pub const PATH_SEAPARATOR: &str = "/";

#[cfg(target_family = "windows")]
pub const PATH_SEAPARATOR: &str = "\\";

#[derive(Debug)]
struct AppError {
    msg: String
}
impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}
impl std::error::Error for AppError {}

fn app_error(msg: &str) -> Box<AppError> {
    Box::new(AppError { msg: msg.to_string() })
}

pub struct App {
    cwd: PathBuf,
    history: Vec<PathBuf>,
    dirs_only: bool,
}
impl App {
    pub fn new(path: Option<String>, dirs_only: bool) -> Result<Self, Box<dyn Error>> {
        let cwd;
        if let Some(s) = path {
            if Path::new(&s).exists() {
                cwd = PathBuf::from(s)
            } else {
                cwd = current_dir().expect("could not determine current directory")
            }
        } else {
            return Err(app_error("Invalid path supplied"));
        };
        Ok(App {
            cwd: cwd,
            history: vec![],
            dirs_only,
        })
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

    fn list_path_children(&self, path: &Path) -> Result<Vec<PathBuf>, Box<dyn Error>> {
        if let Ok(m) = path.metadata() {
            if !m.is_dir() { return Err(app_error("Path is not a directory")) }
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

    fn list_path_children_names(&self, path: &Path) -> Vec<String> {
        let mut result = vec!();
        if let Ok(paths) = self.list_path_children(path) {
            result =  paths.iter().map(|p| self.strip_path_string(p)).collect();
        }
        return result;
    }

    fn path_nth_child(&self, path: &Path, idx: usize) -> Result<PathBuf, Box<dyn Error>> {
        if let Ok(item) = self.list_path_children(path) {
            if item.len() != 0 {
                return Ok(item[idx].clone())
            }
        }
        return Err(app_error("Not found"))
    }

    pub fn list_cwd_child_names(&self) -> Vec<String> {
        self.list_path_children_names(self.cwd.as_path())
    }

    pub fn parent_name(&self) -> Vec<String> {
        if let Some(parent) = self.cwd.parent() {
            self.list_path_children_names(parent)
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

    pub fn list_cwd_nth_child_children_names(&self, idx: usize) -> Result<Vec<String>, Box<dyn Error>> {
        if self.child_is_folder(idx) {
            if let Ok(curr) = self.list_path_children(self.cwd.as_path()) {
                if curr.len() == 0 {
                    return Err(app_error("Not found"));
                }
                return Ok(self.list_path_children_names(&curr[idx]));
            }
        }
        return Err(app_error("Invalid index"));
    }

    pub fn cwd_parent_idx(&self) -> Option<usize> {
        let parent = self.parent_name();
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

        if let Ok(item) = self.path_nth_child(self.cwd.as_path(), idx) {
            if let Ok(m) = item.metadata() {
                return m.is_dir();
            }
        }
        return false;
    }

    pub fn up(&mut self, selected_idx: Option<usize>) -> Result<(), Box<dyn Error>> {
        if let Some(parent) = self.cwd.parent() {
            if let Some(idx) = selected_idx {
                if let Ok(item) = self.path_nth_child(self.cwd.as_path(), idx) {
                    self.history.push(item);
                }
            }
            self.cwd = parent.to_path_buf();
        } else {
            return Err(app_error("No parent"));
        }
        return Ok(());
    }

    pub fn down(&mut self, idx: usize) -> Result<Vec<String>, Box<dyn Error>> {
        let parent = self.list_cwd_child_names();
        if parent.len() == 0 {
            return Err(app_error("No children"));
        }
        if !self.child_is_folder(idx) {
            return Err(app_error("Child is not a folder"));
        }
        if let Ok(child) = self.path_nth_child(self.cwd.as_path(), idx) {
            self.cwd = child;
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
