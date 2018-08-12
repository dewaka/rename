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

    let renaming = rename::Renaming::new(dir, editor, true);
    let count = renaming.rename_files();
    println!("Renamed {} files", count)
}
