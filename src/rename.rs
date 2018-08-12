extern crate tempfile;

use std::fs;
use std::fs::{File, metadata};
use std::io::{Read, Write};
use std::path;

pub struct Renaming {
    pub dir: String,
    pub editor: String,
    pub filter_dirs: bool,
    pub is_demo: bool,
}

#[derive(Debug)]
struct Rename {
    from: String,
    to: String,
    is_dir: bool,
}

impl Renaming {
    pub fn new(dir: &str, editor: &str, filter_dirs: bool) -> Renaming {
        Renaming {
            dir: dir.to_owned(),
            editor: editor.to_owned(),
            filter_dirs,
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

        if !ok {
            println!("Something went wrong!");
            return 0;
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

    // TODO: We need to make this a result
    fn files_in_dir(&self) -> Vec<String> {
        use std::fs::metadata;

        let mut files: Vec<String> = vec![];

        let re_paths = fs::read_dir(path::PathBuf::from(&self.dir));
        match re_paths {
            Ok(paths) =>
                for path in paths {
                    let file = path.unwrap().file_name().to_str().unwrap().to_owned();

                    let md = metadata(&file).unwrap();
                    if md.is_file() {
                        files.push(file);
                    } else if !self.filter_dirs {
                        files.push(file);
                    }
                },
            Err(e) => println!("{:?}", e),
        }

        return files;
    }

    // TODO: Add better error handling
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

    fn rename_files_to(&self, froms: &Vec<String>, tos: &Vec<String>) -> i32 {
        if froms.len() != tos.len() {
            println!("Error: renamed files does not match original files in length");
            return 0;
        }

        let mut renames: Vec<Rename> = vec![];
        for (from, to) in froms.iter().zip(tos.iter()) {
            if let Some(rename) = Rename::rename_for(from, to) {
                renames.push(rename);
            }
        }

        Rename::do_bulk_rename(&renames, false, self.is_demo)
    }
}

impl Rename {
    fn do_rename(&self, is_demo: bool) -> bool {
        if is_demo {
            println!("{} -> {}", self.from, self.to);
            true
        } else {
            let status = fs::rename(&self.from, &self.to);
            status.is_ok()
        }
    }

    // TODO: We need to be smarter about bulk renames
    // Some renames might have to be done earlier than others because when
    // directory renames are involved, there can be later dependencies later on for those
    fn do_bulk_rename(renames: &Vec<Rename>, _early_exit: bool, is_demo: bool) -> i32 {
        let mut count = 0;

        for rename in renames {
            let ok = rename.do_rename(is_demo);
            if ok {
                count += 1;
            } else {
                println!("Failed to rename: {:?}", rename);
            }
        }

        count
    }

    fn rename_for(from: &str, to: &str) -> Option<Rename> {
        if from.eq(to) {
            Option::None
        } else {
            match metadata(from) {
                Ok(md) => {
                    let rename = Rename {
                        from: from.to_owned(),
                        to: to.to_owned(),
                        is_dir: md.is_dir(),
                    };

                    Some(rename)
                },
                Err(_) => {
                    println!("Error requesting metadata: {}", from);
                    Option::None
                },
            }
        }
    }
}
