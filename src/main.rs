extern crate tempfile;

mod rename;

use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: rename dir [editor]");
        return;
    }

    let dir = &args[1];
    let editor = if args.len() == 3 { &args[2] } else { "vim" };

    println!("Renaming in {} using editor {}", dir, editor);

    let renaming = rename::Renaming {
        dir: dir.to_owned(),
        editor: editor.to_owned(),
        filter_dirs: true,
        is_demo: true,
    };

    let count = renaming.rename_files();
    println!("Renamed {} files", count)

    // show_dir(&dir);
}
