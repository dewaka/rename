use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::fs::{self, metadata};
use std::hash::{Hash, Hasher};
use std::path::PathBuf;

pub fn bulk_rename(froms: &Vec<String>, tos: &Vec<String>, is_demo: bool) -> Result<i32, String> {
    if froms.len() != tos.len() {
        return Err("Error: renamed files does not match original files in length".to_string());
    }

    let mut renames: Vec<Rename> = vec![];
    for (from, to) in froms.iter().zip(tos.iter()) {
        match Rename::rename_for(from, to) {
            Ok(rename) => renames.push(rename),
            Err(Some(msg)) => return Err(msg),
            Err(None) => (),
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
    is_dir: bool,
}

impl Rename {
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

    fn rename_for(from: &str, to: &str) -> Result<Rename, Option<String>> {
        if from.eq(to) {
            Err(Option::None)
        } else {
            match metadata(from) {
                Ok(md) => {
                    let rename = Rename {
                        from: from.to_owned(),
                        to: to.to_owned(),
                        is_dir: md.is_dir(),
                    };

                    Ok(rename)
                }
                Err(_) => Err(Option::Some(format!("Error requesting metadata: {}", from))),
            }
        }
    }

    fn with_from(&self, from: &str) -> Rename {
        Rename {
            from: from.to_string(),
            to: self.to.clone(),
            is_dir: self.is_dir,
        }
    }

    fn with_to(&self, to: &str) -> Rename {
        Rename {
            from: self.from.clone(),
            to: to.to_string(),
            is_dir: self.is_dir,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::fs::{self, File};
    use std::io::prelude::*;
    use std::panic;
    use std::path::PathBuf;

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
        fn with_temp_dir(d: &str) -> RenameTestSetup {
            let mut temp_path = env::temp_dir();
            temp_path.push(d);
            RenameTestSetup {
                dir: temp_path.to_str().unwrap().to_string(),
                file_contents: vec![],
            }
        }

        fn init(&self) -> &RenameTestSetup {
            fs::remove_dir_all(&self.dir).unwrap();
            fs::create_dir(&self.dir).unwrap();
            self
        }

        fn full_path(&self, file: &str) -> PathBuf {
            let mut temp_path: PathBuf = PathBuf::new();
            temp_path.push(&self.dir);
            temp_path.push(file);
            temp_path
        }

        fn add_file(&mut self, file: &str, contents: &str) -> &mut RenameTestSetup {
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
        let mut setup = RenameTestSetup::with_temp_dir("rename_test");
        setup.init();
        setup.add_file("A.txt", "hello");
        setup.add_file("B.txt", "hi");

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

}
