extern crate clap;
extern crate tempfile;
extern crate uuid;

mod app;
mod rename;

use clap::{App, Arg};

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
        "left" => if left.is_some() {
            Ok(app::RenameOp::from_left(left.unwrap(), editor, false))
        } else {
            Err("Left file arg is required for left mode".to_string())
        },
        "compare" => if left.is_some() && right.is_some() {
            Ok(app::RenameOp::from_compare(
                left.unwrap(),
                right.unwrap(),
                false,
            ))
        } else {
            Err("Left file and right file args are required for compare mode".to_string())
        },
        "dir" => if dir.is_some() {
            Ok(app::RenameOp::from_dir(
                dir.unwrap(),
                editor,
                !include_dirs,
                false,
            ))
        } else {
            Err("Directory argument required for dir mode".to_string())
        },
        "input" => Ok(app::RenameOp::from_stdin(editor, false)),
        _ => Err(format!("Unexpected mode: {}", mode)),
    };

    match renaming {
        Ok(app) => {
            let result = app.rename();

            match result {
                Ok(count) => println!("Renamed {} files", count),
                Err(msg) => {
                    println!("Error: {}", msg);
                    std::process::exit(1);
                }
            }
        }
        Err(msg) => {
            println!("{}", msg);
            std::process::exit(1);
        }
    }
}
