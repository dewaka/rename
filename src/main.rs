extern crate clap;
extern crate tempfile;

mod rename;

use clap::{App, Arg};

// TODO: Add sorting options
// TODO: Improve error handling with relative paths

fn main() {
    let matches = App::new("rname: bulk rename")
        .version("0.1")
        .author("Chathura C. <dcdewaka@gmail.com>")
        .about("Renames files in bulk by delegating renaming to an editor")
        .arg(
            Arg::with_name("directory")
                .short("d")
                .long("dirr")
                .help("Specify the directory to rename files in")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("editor")
                .short("e")
                .long("editor")
                .help("Specify the custom editor for editing file names")
                .takes_value(true),
        )
        .get_matches();

    let dir = matches.value_of("directory").unwrap_or(".");
    let editor = matches.value_of("editor").unwrap_or("vim");

    let renaming = rename::Renaming::new(dir, editor, true);
    let count = renaming.rename_files();
    println!("Renamed {} files", count)
}
