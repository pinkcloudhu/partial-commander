use std::{
    env::current_dir,
    path::{Path, PathBuf},
};

pub struct App {
    cwd: PathBuf,
}
impl App {
    pub fn new(path: Option<String>) -> Self {
        let path = if let Some(s) = path {
            if Path::new(&s).exists() {
                PathBuf::from(s)
            } else {
                current_dir().unwrap()
            }
        } else {
            current_dir().unwrap()
        };
        App { cwd: path }
    }

    fn list_folder_inner(&self, path: &Path) -> Vec<String> {
        let mut result = Vec::new();
        for path in path.read_dir() {
            for item in path {
                if let Ok(i) = item {
                    if let Ok(m) = i.metadata() {
                        if m.is_dir() { // if is a dir, add slashes on the end
                            if let Some(s) = i.path().to_str() {
                                let s = s.split('/').collect::<Vec<&str>>();
                                result.push(String::from(s[s.len() - 1]) + &String::from("/"));
                            } else {
                                result.push(String::from("???/"));
                            }
                        } else { // is a file
                            if let Some(s) = i.path().to_str() {
                                let s = s.split('/').collect::<Vec<&str>>();
                                result.push(String::from(s[s.len() - 1]));
                            } else {
                                result.push(String::from("???"));
                            }
                        }
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
        } else {
            Vec::new()
        }
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

    pub fn current_folder_parent_idx(&self) -> usize {
        let parent = self.list_parent();
        let curr = self.current_folder() + &String::from("/");
        for (i, item) in parent.iter().enumerate() {
            if item == &curr {
                return i;
            }
        }
        0
    }

    pub fn up(&mut self) {
        if let Some(parent) = self.cwd.parent() {
            self.cwd = parent.to_path_buf();
        }
    }

    pub fn child_is_folder(&self, idx: usize) -> bool {
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

    pub fn list_child(&self, idx: usize) -> Vec<String> {
        if self.child_is_folder(idx) {
            if let Ok(mut curr) = self.cwd.as_path().read_dir() {
                if let Some(Ok(item)) = curr.nth(idx) {
                    return self.list_folder_inner(item.path().as_path());
                }
            }
        }
        return vec![];
    }

    pub fn down(&mut self, idx: usize) -> Vec<String> {
        let old_folder = self.list_folder();

        if let Ok(mut read_dir) = self.cwd.as_path().read_dir() {
            if let Some(dir) = read_dir.nth(idx) {
                if let Ok(entry) = dir {
                    self.cwd = entry.path();
                }
            }
        }
        old_folder
    }
}
