use std::fs::{self, metadata};
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

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

    if !has_rename_conflicts(&renames) {
        do_bulk_rename(&renames, false, is_demo)
    } else {
        Err("There are rename conflicts!".to_string())
    }
}

fn has_rename_conflicts(renames: &Vec<Rename>) -> bool {
    use std::collections::HashMap;

    let mut conflicting: Vec<Rename> = vec![];
    let seen: HashMap<u64, Rename> = HashMap::new();

    for r in renames {
        let k = r.combined_hash();
        if seen.contains_key(&k) {
            let other = seen.get(&k).unwrap();

            if other.equals(&r) {
                conflicting.push(r.clone());
            } else {
                conflicting.push(r.clone());
                conflicting.push(other.clone());
            }

        }
    }

    println!("Conflicting: {:?}", conflicting);
    !conflicting.is_empty()
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

#[derive(Debug, PartialEq, Clone)]
struct Rename {
    from: String,
    to: String,
    is_dir: bool,
}

impl Rename {
    fn combined_hash(&self) -> u64 {
        let mut s = DefaultHasher::new();
        self.from.hash(&mut s);
        self.to.hash(&mut s);
        s.finish()
    }

    fn equals(&self, other: &Rename) -> bool {
        self.from == other.from && self.to == other.to && self.is_dir == other.is_dir
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
                },
                Err(_) => {
                    Err(Option::Some(format!("Error requesting metadata: {}", from)))
                },
            }
        }
    }
}
