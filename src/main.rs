extern crate clap;
extern crate tempfile;
extern crate uuid;
extern crate walkdir;

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
            Arg::with_name("sort")
                .short("s")
                .required(false)
                .takes_value(true)
                .multiple(false)
                .help("Specify the sorting mode - none (default), alph or dir"),
        )
        .arg(
            Arg::with_name("sort-desc")
                .short("O")
                .required(false)
                .multiple(false)
                .help("Sorting descending order"),
        )
        .arg(
            Arg::with_name("exclude-dirs")
                .short("D")
                .required(false)
                .multiple(false)
                .help("Whether to exclude directories"),
        )
        .arg(
            Arg::with_name("recursive")
                .short("R")
                .required(false)
                .multiple(false)
                .help("Rename in subdirectories recursively"),
        )
        .get_matches();

    let mode = matches.value_of("mode").unwrap_or("dir");

    let dir = matches.value_of("directory");
    let left = matches.value_of("left");
    let right = matches.value_of("right");

    let editor = matches.value_of("editor").unwrap_or("vim");
    let exclude_dirs = matches.occurrences_of("exclude-dirs") > 0;
    let recursive = matches.occurrences_of("recursive") > 0;

    let sort_type = matches.value_of("sort").unwrap_or("none");
    let descending = matches.occurrences_of("sort-desc") > 0;

    let sort_option = match sort_type {
        "none" => Ok(None),
        "alph" | "alphabetical" | "file" => Ok(Some(app::SortOption {
            order: app::SortOrder::Alphabetical,
            ascending: !descending,
        })),
        "dir" | "directory" | "folder" => Ok(Some(app::SortOption {
            order: app::SortOrder::DirsFirst,
            ascending: !descending,
        })),
        _ => Err(()),
    };

    let sorting = sort_option.expect(&format!("Invalid sort option: {}", sort_type));

    let renaming = match mode {
        "left" => if left.is_some() {
            Ok(app::RenameOp::from_left(
                left.unwrap(),
                editor,
                false,
                sorting,
            ))
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
                recursive,
                exclude_dirs,
                false,
                sorting,
            ))
        } else {
            Err("Directory argument required for dir mode".to_string())
        },
        "input" => Ok(app::RenameOp::from_stdin(editor, false, sorting)),
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
