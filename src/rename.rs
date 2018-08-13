use std::fs;
use std::fs::metadata;

pub fn bulk_rename(froms: &Vec<String>, tos: &Vec<String>, is_demo: bool) -> i32 {
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

    do_bulk_rename(&renames, false, is_demo)
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

#[derive(Debug)]
struct Rename {
    from: String,
    to: String,
    is_dir: bool,
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
