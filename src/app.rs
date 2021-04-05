use std::{
    path::{PathBuf, Path},
    env::current_dir,
};

pub struct App {
    cwd: PathBuf,
}
impl App {
    pub fn new(path: Option<String>) -> Self {
        let path = if let Some(s) = path {
            if Path::new(&s).exists() {
                PathBuf::from(s)
            } else { current_dir().unwrap() }
        } else { current_dir().unwrap() };
        App {
            cwd: path
        }
    }

    fn list_folder_inner(&self, path: &Path) -> Vec<String> {
        let mut result = Vec::new();
        for path in path.read_dir() {
            for item in path {
                if let Ok(i) = item {
                    if let Some(s) = i.path().to_str() {
                        let s = s.split('/').collect::<Vec<&str>>();
                        result.push(String::from(s[s.len() - 1]));
                    } else {
                        result.push(String::from("???"));
                    }
                } else {
                    result.push(String::from("???"));
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
        } else { Vec::new() }
    }

    pub fn current_folder(&self) -> String {
        if let Some(s) = self.cwd.as_os_str().to_str() {
            let s = s.split('/').collect::<Vec<&str>>();
            String::from(s[s.len() - 1])
        } else {
            String::from("???")
        }
    }

    pub fn parent_folder(&self) -> String {
        if let Some(p) = self.cwd.parent() {
            if let Some(s) = p.to_path_buf().as_os_str().to_str() {
                let s = s.split('/').collect::<Vec<&str>>();
                String::from(s[s.len() - 1])
            } else {
                String::from("???")
            }
        } else {
            String::from("???")
        }
    }

    pub fn up(&mut self) {
        if let Some(parent) = self.cwd.parent() {
            self.cwd = parent.to_path_buf();
        }
    }
}