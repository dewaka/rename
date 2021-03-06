use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;

pub fn bulk_rename(froms: &Vec<String>, tos: &Vec<String>, is_demo: bool) -> Result<i32, String> {
    if froms.len() != tos.len() {
        return Err("Error: renamed files does not match original files in length".to_string());
    }

    let mut renames: Vec<Rename> = vec![];
    for (from, to) in froms.iter().zip(tos.iter()) {
        match Rename::renames_for(from, to) {
            Ok(rs) => renames.extend(rs.iter().cloned()),
            Err(msg) => return Err(msg),
        }
    }

    let (conflicting, non_conflicting) = split_by_rename_conflicts(&renames);

    match do_bulk_rename(&non_conflicting, false, is_demo) {
        Ok(count1) => match do_bulk_rename(&with_temporary_moves(&conflicting), false, is_demo) {
            Ok(_) => Ok(count1 + conflicting.len() as i32 * 2),
            Err(_) => Ok(count1),
        },
        e => e,
    }
}

fn replace_filename(file_path: &str, name: &str) -> String {
    let mut path = PathBuf::new();
    path.push(file_path);
    path.pop();
    path.push(name);
    path.to_str().unwrap().to_string()
}

fn with_temporary_moves(renames: &Vec<(Rename, Rename)>) -> Vec<Rename> {
    use uuid::Uuid;

    let mut non_conflicting: Vec<Rename> = vec![];
    for (ref x, ref y) in renames {
        let temp_file_name = format!("{}", Uuid::new_v4());

        non_conflicting.push(x.with_to(&replace_filename(&x.to, &temp_file_name)));
        non_conflicting.push(y.clone());
        non_conflicting.push(x.with_from(&replace_filename(&x.from, &temp_file_name)));
    }

    non_conflicting
}

fn split_by_rename_conflicts(renames: &Vec<Rename>) -> (Vec<(Rename, Rename)>, Vec<Rename>) {
    let mut seen: HashMap<u64, Rename> = HashMap::new();
    let mut conflicting_set: HashSet<Rename> = HashSet::new();
    let mut conflicting: Vec<(Rename, Rename)> = vec![];

    for r in renames {
        let k = r.combined_hash();
        if seen.contains_key(&k) {
            let other = seen.get(&k).unwrap();

            if r != other {
                conflicting_set.insert(r.clone());
                conflicting_set.insert(other.clone());

                conflicting.push((r.clone(), other.clone()));
            }
        } else {
            seen.insert(k, r.clone());
        }
    }

    let non_conflicting: Vec<Rename> = renames
        .iter()
        .filter(|r| !conflicting_set.contains(&r))
        .map(|r| r.clone())
        .collect();

    (conflicting, non_conflicting)
}

// TODO: We need to be smarter about bulk renames
// Some renames might have to be done earlier than others because when
// directory renames are involved, there can be later dependencies later on for those
fn do_bulk_rename(renames: &Vec<Rename>, early_exit: bool, is_demo: bool) -> Result<i32, String> {
    let mut count = 0;

    for rename in renames {
        let ok = rename.do_rename(is_demo);
        if ok {
            count += 1;
        } else {
            if early_exit {
                return Err(format!("Failed to rename: {:?}", rename));
            } else {
                println!("Warning - failed to rename: {:?}", rename);
            }
        }
    }

    Ok(count)
}

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
struct Rename {
    from: String,
    to: String,
}

impl Rename {
    fn new(from: &str, to: &str) -> Self {
        Rename {
            from: from.to_owned(),
            to: to.to_owned(),
        }
    }

    fn combined_hash(&self) -> u64 {
        fn hash_str(s: &str) -> u64 {
            let mut h = DefaultHasher::new();
            s.hash(&mut h);
            h.finish()
        }

        let (combined, _) = hash_str(&self.from).overflowing_add(hash_str(&self.to));
        combined
    }

    fn do_rename(&self, is_demo: bool) -> bool {
        if is_demo {
            println!("{} -> {}", self.from, self.to);
            true
        } else {
            let status = fs::rename(&self.from, &self.to);
            status.is_ok()
        }
    }

    fn rename_sequence(from: &str, to: &str) -> Result<Vec<Rename>, String> {
        let mut renames: Vec<Rename> = vec![];

        let mut from_path = PathBuf::from(from);
        let mut to_path = PathBuf::from(to);
        let mut from_path_finished;
        let mut to_path_finished;

        loop {
            let f = from_path.clone();
            let t = to_path.clone();

            let f1 = f.file_name();
            let t1 = t.file_name();

            from_path_finished = f1.is_none();
            to_path_finished = t1.is_none();

            if from_path_finished || to_path_finished {
                break;
            }

            if f1 != t1 {
                // We need a rename from f1 to t1
                let p1 = from_path.parent();

                if p1.is_none() {
                    let fs1 = f1.unwrap().to_str().unwrap();
                    let ts1 = t1.unwrap().to_str().unwrap();

                    renames.push(Rename::new(fs1, ts1));
                } else {
                    let mut p = PathBuf::new();
                    p.push(p1.unwrap());
                    p.push(f1.unwrap());

                    let fs1 = p.to_str().unwrap().to_owned();
                    assert!(p.pop());

                    p.push(t1.unwrap());
                    let ts1 = p.to_str().unwrap();

                    renames.push(Rename::new(&fs1, ts1));
                }
            }

            from_path_finished = !from_path.pop();
            to_path_finished = !to_path.pop();

            if from_path_finished || to_path_finished {
                break;
            }
        }

        if from_path_finished == to_path_finished {
            Ok(renames)
        } else {
            Err(format!(
                "Incompatible paths for renaming: {} -> {}",
                from, to
            ))
        }
    }

    pub fn renames_for(from: &str, to: &str) -> Result<Vec<Rename>, String> {
        Rename::rename_sequence(from, to)
    }

    fn with_from(&self, from: &str) -> Rename {
        Rename {
            from: from.to_string(),
            to: self.to.clone(),
        }
    }

    fn with_to(&self, to: &str) -> Rename {
        Rename {
            from: self.from.clone(),
            to: to.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Rename;
    use std::env;
    use std::fs::{self, File};
    use std::io::prelude::*;
    use std::panic;
    use std::path::{Path, PathBuf};

    // Run a test with given setup before the test and teardown after the test.
    // Should ensure that setup and teardown code does not panic
    #[allow(dead_code)]
    fn run_test_with_setup<S, C, T>(setup: S, teardown: C, test: T) -> ()
    where
        S: FnOnce() -> (),
        C: FnOnce() -> (),
        T: FnOnce() -> () + panic::UnwindSafe,
    {
        setup();
        let result = panic::catch_unwind(|| test());
        teardown();

        assert!(result.is_ok());
    }

    struct RenameTestSetup {
        dir: String,
        file_contents: Vec<(String, String)>,
    }

    impl RenameTestSetup {
        fn with_temp_dir(d: &str) -> Self {
            let mut temp_path = env::temp_dir();
            temp_path.push(d);
            RenameTestSetup {
                dir: temp_path.to_str().unwrap().to_string(),
                file_contents: vec![],
            }
        }

        fn init(self) -> Self {
            if Path::new(&self.dir).exists() {
                fs::remove_dir_all(&self.dir).unwrap();
            }
            fs::create_dir(&self.dir).unwrap();
            self
        }

        fn full_path(&self, file: &str) -> PathBuf {
            let mut temp_path: PathBuf = PathBuf::new();
            temp_path.push(&self.dir);
            temp_path.push(file);
            temp_path
        }

        fn add_file(mut self, file: &str, contents: &str) -> Self {
            let file_path = self.full_path(file);
            match fs::write(&file_path, contents) {
                Ok(_) => self.file_contents.push((
                    file_path.to_str().unwrap().to_string(),
                    contents.to_string(),
                )),
                Err(_) => (),
            }

            self
        }
    }

    fn read_all(file: &str) -> String {
        let mut contents = String::new();

        let mut f = File::open(file).expect("file not found");
        f.read_to_string(&mut contents)
            .expect("something went wrong reading the file");

        contents
    }

    #[test]
    fn rename_swap_test() {
        let setup = RenameTestSetup::with_temp_dir("rename_test")
            .init()
            .add_file("A.txt", "hello")
            .add_file("B.txt", "hi");

        println!("Files: {:?}", setup.file_contents);

        let (ref file_a, ref contents_a) = setup.file_contents[0];
        let (ref file_b, ref contents_b) = setup.file_contents[1];

        let res = super::bulk_rename(
            &vec![file_a.to_owned(), file_b.to_owned()],
            &vec![file_b.to_owned(), file_a.to_owned()],
            false,
        );

        assert!(res.is_ok());

        assert_eq!(read_all(&file_a), contents_b.to_owned());
        assert_eq!(read_all(&file_b), contents_a.to_owned());
    }

    #[test]
    fn rename_sequence_test() {
        // Successful renames
        {
            let result = Rename::rename_sequence("/x/y/z", "/a/b/c");

            match result {
                Ok(res) => {
                    assert_eq!(res.len(), 3);

                    assert_eq!(
                        res,
                        vec![
                            Rename::new("/x/y/z", "/x/y/c"),
                            Rename::new("/x/y", "/x/b"),
                            Rename::new("/x", "/a"),
                        ]
                    );
                }
                Err(_) => assert!(false),
            }
        }
        {
            let result = Rename::rename_sequence("/x", "/a");

            match result {
                Ok(res) => {
                    assert_eq!(res.len(), 1);
                    assert_eq!(res[0], Rename::new("/x", "/a"));
                }
                Err(_) => assert!(false),
            }
        }

        // Following ones should fail due to them not being compatible paths
        {
            let result = Rename::rename_sequence("/A/B/C", "/X/Y");
            assert!(result.is_err());
        }
        {
            let result = Rename::rename_sequence("/X/Y", "/A/B/C");
            assert!(result.is_err());
        }

        // This case should also return empty vector of renames since we don't need to rename anything here
        {
            let result = Rename::rename_sequence("/x", "/x");
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), vec![]);
        }
    }

}
