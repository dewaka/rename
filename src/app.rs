extern crate tempfile;

use std::fs::{self, File};
use std::io::{self, BufRead, Read, Write};

use rename;

pub enum SortOrder {
    Alphabetical,
    DirsFirst,
}

pub struct SortOption {
    pub order: SortOrder,
    pub ascending: bool,
}

enum RenameType {
    Directory {
        dir: String,
        editor: String,
        filter_dirs: bool,
        depth: Option<usize>,
        sorting: Option<SortOption>,
    },
    LeftFile {
        file: String,
        editor: String,
        sorting: Option<SortOption>,
    },
    StdinInput {
        editor: String,
        sorting: Option<SortOption>,
    },
    FileCompare {
        left: String,
        right: String,
    },
}

pub struct RenameOp {
    is_demo: bool,
    rename_type: RenameType,
}

impl RenameOp {
    pub fn from_dir(
        dir: &str,
        editor: &str,
        depth: Option<usize>,
        filter_dirs: bool,
        is_demo: bool,
        sorting: Option<SortOption>,
    ) -> RenameOp {
        RenameOp {
            is_demo,
            rename_type: RenameType::Directory {
                dir: dir.to_string(),
                editor: editor.to_string(),
                depth,
                filter_dirs,
                sorting,
            },
        }
    }

    pub fn from_stdin(editor: &str, is_demo: bool, sorting: Option<SortOption>) -> RenameOp {
        RenameOp {
            is_demo,
            rename_type: RenameType::StdinInput {
                editor: editor.to_string(),
                sorting,
            },
        }
    }

    pub fn from_left(
        file: &str,
        editor: &str,
        is_demo: bool,
        sorting: Option<SortOption>,
    ) -> RenameOp {
        RenameOp {
            is_demo,
            rename_type: RenameType::LeftFile {
                file: file.to_string(),
                editor: editor.to_string(),
                sorting,
            },
        }
    }

    pub fn from_compare(left: &str, right: &str, is_demo: bool) -> RenameOp {
        RenameOp {
            is_demo,
            rename_type: RenameType::FileCompare {
                left: left.to_string(),
                right: right.to_string(),
            },
        }
    }

    fn directory_contents(
        &self,
        dir: &str,
        contents: &mut Vec<String>,
        depth: Option<usize>,
        filter_dirs: bool,
    ) {
        use std::fs::metadata;
        use walkdir::WalkDir;;

        let walker = if let Some(n) = depth {
            WalkDir::new(dir).follow_links(false).max_depth(n)
        } else {
            WalkDir::new(dir).follow_links(false)
        };

        for entry in walker {
            match entry {
                Ok(p) => match metadata(p.path()) {
                    Ok(md) => {
                        let file = p.path().to_str().unwrap();
                        if md.is_file() {
                            contents.push(file.to_owned());
                        } else if !filter_dirs {
                            contents.push(file.to_owned());
                        }
                    }
                    Err(e) => println!("Error reading metadata: {}", e),
                },
                Err(e) => println!("Error: {}", e),
            }
        }
    }

    fn write_temp_file(&self, fnames: &Vec<String>) -> tempfile::NamedTempFile {
        let mut nfile = tempfile::NamedTempFile::new().unwrap();

        for name in fnames {
            write!(nfile, "{}\n", name).unwrap();
        }

        nfile.flush().unwrap();
        nfile
    }

    fn open_file_with_editor(&self, file: &str, editor: &str) -> bool {
        use std::process::{Command, ExitStatus};

        let editor_cmd = format!("{} {}", editor, file);
        if self.is_demo {
            println!("Editor command: {}", editor_cmd);
        }

        let mut cmd = Command::new(editor);
        let mut exit_status: Option<ExitStatus> = Option::None;

        if let Ok(mut child) = cmd.arg(file).spawn() {
            exit_status = Some(child.wait().expect("Failed to execute command"));
        } else {
            println!("Error - failed to run: {}", editor_cmd);
        }

        if let Some(status) = exit_status {
            status.success()
        } else {
            false
        }
    }

    fn read_from_editor(&self, froms: &Vec<String>, editor: &str, tos: &mut Vec<String>) {
        if froms.is_empty() {
            println!("Nothing to rename!");
            return;
        }

        let temp_file = self.write_temp_file(&froms);
        let ok = self.open_file_with_editor(temp_file.path().to_str().unwrap(), editor);

        if !ok {
            println!("Something went wrong!");
            return;
        }

        self.read_from_file(temp_file.path().to_str().unwrap(), tos);
    }

    fn read_from_file(&self, file: &str, contents: &mut Vec<String>) {
        let mut f = File::open(file).expect("file not found");

        let mut lines = String::new();
        f.read_to_string(&mut lines)
            .expect("Something went wrong while reading file");

        for s in lines.split("\n") {
            if s != "" {
                contents.push(s.to_owned());
            }
        }
    }

    fn read_from_stdin(&self, contents: &mut Vec<String>) {
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            let file = line.unwrap();
            if file.is_empty() {
                break;
            } else {
                contents.push(file);
            }
        }
    }

    fn sort_alphabetical(&self, files: &mut Vec<String>, ascending: bool) {
        files.sort_by(|x, y| {
            if ascending {
                x.cmp(y)
            } else {
                y.cmp(x)
            }
        });
    }

    fn sort_dirs_first(&self, files: &mut Vec<String>, ascending: bool) {
        let mut folders: Vec<String> = vec![];
        let mut normal_files: Vec<String> = vec![];

        for ref s in files.iter() {
            match fs::metadata(s) {
                Ok(m) => if m.is_dir() {
                    folders.push(s.to_string());
                } else {
                    normal_files.push(s.to_string());
                },
                Err(_) => normal_files.push(s.to_string()),
            }
        }

        self.sort_alphabetical(&mut folders, ascending);
        self.sort_alphabetical(&mut normal_files, ascending);

        files.clear();
        files.append(&mut folders);
        files.append(&mut normal_files);
    }

    fn sort_files(&self, files: &mut Vec<String>, sort_option: &Option<SortOption>) {
        match sort_option {
            Some(SortOption {
                order: SortOrder::Alphabetical,
                ascending,
            }) => self.sort_alphabetical(files, *ascending),
            Some(SortOption {
                order: SortOrder::DirsFirst,
                ascending,
            }) => self.sort_dirs_first(files, *ascending),
            None => (),
        }
    }

    pub fn rename(&self) -> Result<i32, String> {
        let mut froms: Vec<String> = vec![];
        let mut tos: Vec<String> = vec![];

        match self.rename_type {
            RenameType::Directory {
                ref dir,
                ref editor,
                depth,
                filter_dirs,
                ref sorting,
            } => {
                self.directory_contents(&dir, &mut froms, depth, filter_dirs);
                self.sort_files(&mut froms, sorting);
                self.read_from_editor(&froms, &editor, &mut tos);
            }
            RenameType::LeftFile {
                ref file,
                ref editor,
                ref sorting,
            } => {
                self.read_from_file(&file, &mut froms);
                self.sort_files(&mut froms, sorting);
                self.read_from_editor(&froms, &editor, &mut tos);
            }
            RenameType::FileCompare {
                ref left,
                ref right,
            } => {
                self.read_from_file(&left, &mut froms);
                self.read_from_file(&right, &mut tos);
            }
            RenameType::StdinInput {
                ref editor,
                ref sorting,
            } => {
                self.read_from_stdin(&mut froms);
                self.sort_files(&mut froms, sorting);
                self.read_from_editor(&froms, &editor, &mut tos);
            }
        }

        rename::bulk_rename(&froms, &tos, self.is_demo)
    }
}
