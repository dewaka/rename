extern crate tempfile;

use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::path;
use rename;

enum RenameType {
    Directory {
        dir: String,
        editor: String,
        filter_dirs: bool,
    },
    LeftFile {
        file: String,
        editor: String,
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
    pub fn from_dir(dir: &str, editor: &str, filter_dirs: bool, is_demo: bool) -> RenameOp {
        RenameOp {
            is_demo,
            rename_type: RenameType::Directory { dir: dir.to_string(), editor: editor.to_string(), filter_dirs },
        }
    }

    pub fn from_left(file: &str, editor: &str, is_demo: bool) -> RenameOp {
        RenameOp {
            is_demo,
            rename_type: RenameType::LeftFile { file: file.to_string(), editor: editor.to_string() },
        }
    }

    pub fn from_compare(left: &str, right: &str, is_demo: bool) -> RenameOp {
        RenameOp {
            is_demo,
            rename_type: RenameType::FileCompare { left: left.to_string(), right: right.to_string() },
        }
    }

    fn directory_contents(&self, dir: &str, contents: &mut Vec<String>, filter_dirs: bool) {
        use std::fs::metadata;

        let re_paths = fs::read_dir(path::PathBuf::from(dir));
        match re_paths {
            Ok(paths) =>
                for path in paths {
                    let file = path.unwrap().file_name().to_str().unwrap().to_owned();

                    let md = metadata(&file).unwrap();
                    if md.is_file() {
                        contents.push(file);
                    } else if !filter_dirs {
                        contents.push(file);
                    }
                },
            Err(e) => println!("{:?}", e),
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

    fn from_editor(&self, froms: &Vec<String>, editor: &str, tos: &mut Vec<String>) {
        let temp_file = self.write_temp_file(&froms);
        let ok = self.open_file_with_editor(temp_file.path().to_str().unwrap(), editor);

        if !ok {
            println!("Something went wrong!");
            return;
        }

        self.from_file(temp_file.path().to_str().unwrap(), tos);
    }

    fn from_file(&self, file: &str, contents: &mut Vec<String>) {
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

    pub fn rename(&self) -> i32 {
        let mut froms: Vec<String> = vec![];
        let mut tos: Vec<String> = vec![];

        match self.rename_type {
            RenameType::Directory { ref dir, ref editor, filter_dirs } => {
                self.directory_contents(&dir, &mut froms, filter_dirs);
                self.from_editor(&froms, &editor, &mut tos);
            }
            RenameType::LeftFile { ref file, ref editor } => {
                self.from_file(&file, &mut froms);
                self.from_editor(&froms, &editor, &mut tos);
            }
            RenameType::FileCompare { ref left, ref right } => {
                self.from_file(&left, &mut froms);
                self.from_file(&right, &mut tos);
            }
        }

        rename::bulk_rename(&froms, &tos, self.is_demo)
    }
}

