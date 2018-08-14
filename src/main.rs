extern crate clap;
extern crate tempfile;

mod app;
mod rename;

use clap::{App, Arg};

// TODO: Add mode to support diff based renames
fn main() {
    let matches = App::new("rname: bulk rename")
        .version("0.1")
        .author("Chathura C. <dcdewaka@gmail.com>")
        .about("Renames files in bulk by delegating renaming to an editor")
        .arg(
            Arg::with_name("mode")
                .short("m")
                .required(false)
                .takes_value(true)
                .multiple(false)
                .help("Specify the renaming mode - directory, stdin, left or diff"),
        )
        .arg(
            Arg::with_name("left")
                .short("l")
                .required(false)
                .takes_value(true)
                .multiple(false)
                .help("Specify the left input to rename from"),
        )
        .arg(
            Arg::with_name("right")
                .short("r")
                .required(false)
                .takes_value(true)
                .multiple(false)
                .help("Specify the right input to rename to"),
        )
        .arg(
            Arg::with_name("directory")
                .index(1)
                .required(false)
                .takes_value(true)
                .help("Specify the directory to rename files in"),
        )
        .arg(
            Arg::with_name("editor")
                .short("e")
                .long("editor")
                .takes_value(true)
                .help("Specify the custom editor for editing file names"),
        )
        .arg(
            Arg::with_name("include-dirs")
                .short("d")
                .required(false)
                .multiple(false)
                .help("Whether to include directories"),
        )
        .get_matches();

    let mode = matches.value_of("mode").unwrap_or("dir");

    let dir = matches.value_of("directory");
    let left = matches.value_of("left");
    let right = matches.value_of("right");

    let editor = matches.value_of("editor").unwrap_or("vim");
    let include_dirs = matches.occurrences_of("include-dirs") > 0;

    let renaming = match mode {
        "left" =>
            if left.is_some() {
                Some(app::RenameOp::from_left(left.unwrap(), editor, false))
            } else {
                println!("Left file arg is required for left mode");
                Option::None
            },
        "compare" =>
            if left.is_some() && right.is_some() {
                Some(app::RenameOp::from_compare(left.unwrap(), right.unwrap(), false))
            } else {
                println!("Left file and right file args are required for compare mode");
                Option::None
            },
        "dir" =>
            if dir.is_some() {
                Some(app::RenameOp::from_dir(dir.unwrap(), editor, !include_dirs, false))
            } else {
                println!("Directory argument required for dir mode");
                Option::None
            },
        "input" => Some(app::RenameOp::from_stdin(editor, false)),
        _ => {
            println!("Unexpected mode: {}", mode);
            Option::None
        },
    };

    if renaming.is_none() {
        return;
    }

    let count = renaming.unwrap().rename();
    println!("Renamed {} files", count)
}
