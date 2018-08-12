extern crate tempfile;

use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::path;

pub struct Renaming {
    pub dir: String,
    pub editor: String,
    pub filter_dirs: bool,
    pub is_demo: bool,
}

enum RenameStatus {
    Error,
    Same,
    Success,
}

impl Renaming {
    pub fn new(dir: &str, editor: &str, filter_dirs: bool) -> Renaming {
        Renaming {
            dir: dir.to_owned(),
            editor: editor.to_owned(),
            filter_dirs: filter_dirs,
            is_demo: false,
        }
    }

    pub fn filter_dirs(&mut self, filter: bool) -> &mut Renaming {
        self.filter_dirs = filter;
        self
    }

    pub fn demo(&mut self, demo: bool) -> &mut Renaming {
        self.is_demo = demo;
        self
    }

    pub fn rename_files(&self) -> i32 {
        let files = self.files_in_dir();
        let temp_file = self.write_temp_file(&files);

        let ok = self.open_file_with_editor(temp_file.path().to_str().unwrap(), &self.editor);

        if ok {
            println!("Yeah!")
        } else {
            println!("Something went wrong!")
        }

        let modified_files = self.read_files_from_file(temp_file.path());

        if files.len() != modified_files.len() {
            println!("File count does not match!");
            println!("Original: {:?}", files);
            println!("Modified: {:?}", modified_files);
            return 0;
        }

        self.rename_files_to(&files, &modified_files)
    }

    fn write_temp_file(&self, fnames: &Vec<String>) -> tempfile::NamedTempFile {
        let mut nfile = tempfile::NamedTempFile::new().unwrap();

        for name in fnames {
            write!(nfile, "{}\n", name).unwrap();
        }

        nfile.flush().unwrap();
        nfile
    }

    fn read_files_from_file(&self, path: &path::Path) -> Vec<String> {
        let mut f = File::open(path).expect("file not found");

        let mut contents = String::new();
        f.read_to_string(&mut contents)
            .expect("Something went wrong while reading file");

        let mut files: Vec<String> = vec![];
        for s in contents.split("\n") {
            if s != "" {
                files.push(s.to_owned());
            }
        }
        files
    }

    fn files_in_dir(&self) -> Vec<String> {
        use std::fs::metadata;

        let mut files: Vec<String> = vec![];

        let paths = fs::read_dir(&self.dir).unwrap();
        for path in paths {
            let file = path.unwrap().file_name().to_str().unwrap().to_owned();

            let md = metadata(&file).unwrap();
            if md.is_file() {
                files.push(file);
            } else if !self.filter_dirs {
                files.push(file);
            }
        }

        return files;
    }

    fn open_file_with_editor(&self, file: &str, editor: &str) -> bool {
        use std::process::{Command, ExitStatus};

        let editor_cmd = format!("{} {}", editor, file);
        println!("Editor command: {}", editor_cmd);

        let mut cmd = Command::new(editor);

        let mut exit_status: Option<ExitStatus> = Option::None;

        if let Ok(mut child) = cmd.arg(file).spawn() {
            exit_status = Some(child.wait().expect("Failed to execute command"));
            println!("Done with command")
        } else {
            println!("Command didn't start")
        }

        if let Some(status) = exit_status {
            status.success()
        } else {
            false
        }
    }

    fn rename_file(&self, orig: &String, rname: &String) -> RenameStatus {
        if orig.eq(rname) {
            RenameStatus::Same
        } else {
            if self.is_demo {
                println!("{} -> {}", orig, rname);

                RenameStatus::Success
            } else {
                let status = fs::rename(orig, rname);
                if status.is_ok() {
                    RenameStatus::Success
                } else {
                    RenameStatus::Error
                }
            }
        }
    }

    fn rename_files_to(&self, orig: &Vec<String>, rnames: &Vec<String>) -> i32 {
        let mut count = 0;

        if orig.len() != rnames.len() {
            println!("Error: renamed files does not match original files in length");
            return count;
        }

        for i in 0..orig.len() {
            let original = &orig[i];
            let new_name = &rnames[i];

            match self.rename_file(&original, &new_name) {
                RenameStatus::Success => count += 1,
                RenameStatus::Error => println!("Rename failed for: {} to {}", original, new_name),
                RenameStatus::Same => (),
            }
        }

        count
    }
}
