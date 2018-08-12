extern crate clap;
extern crate tempfile;

mod rename;

use clap::{App, Arg};

fn main() {
    let matches = App::new("rname: bulk rename")
        .version("0.1")
        .author("Chathura C. <dcdewaka@gmail.com>")
        .about("Renames files in bulk by delegating renaming to an editor")
        .arg(
            Arg::with_name("directory")
                .index(1)
                .required(true)
                .takes_value(true)
                .help("Specify the directory to rename files in")
        )
        .arg(
            Arg::with_name("editor")
                .short("e")
                .long("editor")
                .takes_value(true)
                .help("Specify the custom editor for editing file names")
        )
        .arg(
            Arg::with_name("include-dirs")
                .short("d")
                .required(false)
                .multiple(false)
                .help("Whether to include directories")
        )
        .get_matches();

    let dir = matches.value_of("directory").unwrap();
    let editor = matches.value_of("editor").unwrap_or("vim");
    let include_dirs = matches.occurrences_of("include-dirs") > 0;

    let renaming = rename::Renaming::new(dir, editor, !include_dirs);
    let count = renaming.rename_files();
    println!("Renamed {} files", count)
}
