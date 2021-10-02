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

struct AppError {
    msg: String
}
impl std::fmt::Debug for AppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
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
        let cwd = path
            .and_then(|p| {
                let pathbuf = PathBuf::from(p);
                if pathbuf.exists() {
                    Some(pathbuf)
                } else { None }
            })
            .or_else(|| {
                Some(current_dir().expect("could not determine current directory"))
            })
            .expect("Invalid path supplied");
        Ok(App {
            cwd: cwd,
            history: vec![],
            dirs_only,
        })
    }

    pub fn is_dirs_only(&self) -> bool {
        self.dirs_only
    }

    fn strip_path_string(&self, path: &Path) -> Result<String, Box<dyn Error>> {
        let p = &path
            .to_str()
            .ok_or(app_error("Couldn't convert path to string"))?
            .split(PATH_SEAPARATOR)
            .collect::<Vec<&str>>();
        let s = p[p.len() - 1];

        if path.metadata()?.is_dir() {
            return Ok(s.to_string() + PATH_SEAPARATOR)
        }
        Ok(s.to_string())
    }

    fn list_path_children(&self, path: &PathBuf) -> Result<Vec<PathBuf>, Box<dyn Error>> {
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

    fn list_path_children_names(&self, path: &PathBuf) -> Result<Vec<String>, Box<dyn Error>> {
        Ok(
            self.list_path_children(path)?
            .iter().map(|p| 
                self.strip_path_string(p)
                    .ok()
                    .unwrap_or("???".to_string())
            ).collect()
        )
    }

    fn path_nth_child(&self, path: &PathBuf, idx: usize) -> Result<PathBuf, Box<dyn Error>> {
        Ok(self.list_path_children(path)?
            .get(idx)
            .ok_or(app_error("No such child"))?.clone())
    }

    pub fn list_cwd_child_names(&self) -> Result<Vec<String>, Box<dyn Error>> {
        self.list_path_children_names(&self.cwd)
    }

    pub fn parent_children_names(&self) -> Result<Vec<String>, Box<dyn Error>> {
        self.list_path_children_names(
            &self.cwd.parent().ok_or(app_error("No parent while trying to list parent's children"))?.to_path_buf()
        )
    }

    pub fn current_folder_name(&self) -> Result<String, Box<dyn Error>> {
        self.strip_path_string(self.cwd.as_path())
    }

    pub fn parent_folder_name(&self) -> Result<String, Box<dyn Error>> {
        self.strip_path_string(
            self.cwd.parent().ok_or(app_error("No parent"))?
        )
    }

    pub fn list_cwd_nth_child_children_names(&self, idx: usize) -> Result<Vec<String>, Box<dyn Error>> {
        if !self.child_is_folder(idx) {
            return Err(app_error("Child is not a folder"));
        }
        let children = self.list_path_children(&self.cwd)?;
        let child = children.get(idx).ok_or(app_error("No such child"))?;
        self.list_path_children_names(&child.to_path_buf())
    }

    pub fn cwd_parent_idx(&self) -> Result<usize, Box<dyn Error>> {
        let parent = self.parent_children_names()?;
        let curr = self.current_folder_name()?;
        for (i, item) in parent.iter().enumerate() {
            if item == &curr {
                return Ok(i);
            }
        }
        Err(app_error("Not found"))
    }

    pub fn child_is_folder(&self, idx: usize) -> bool {
        if self.dirs_only { return true; }

        self.path_nth_child(&self.cwd, idx)
            .and_then(|p| {
                Ok(p.metadata()?.is_dir())
            })
            .unwrap_or(false)
    }

    pub fn up(&mut self, selected_idx: Option<usize>) -> Result<(), Box<dyn Error>> {
        self.path_nth_child(
            &self.cwd,
            selected_idx.unwrap_or_default()
        ).and_then(|i| {
                Ok(self.history.push((*i).to_path_buf()))
        }).ok();
        self.cwd = self.cwd.parent().ok_or(app_error("No parent"))?.to_path_buf();
        Ok(())
    }

    pub fn down(&mut self, idx: usize) -> Result<Vec<String>, Box<dyn Error>> {
        let parent = self.list_cwd_child_names()?;
        if parent.len() == 0 {
            return Err(app_error("No children"));
        }
        if !self.child_is_folder(idx) {
            return Err(app_error("Child is not a folder"));
        }
        if let Ok(child) = self.path_nth_child(&self.cwd, idx) {
            self.cwd = child.clone();
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
