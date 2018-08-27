# Renaming Tool

Tool to bulk rename files on the command line using an editor.

This tools works in a similar way how Emacs `dir-ed` mode does renames.

# How to use the tool

Current command line options are as follows,

```
rename: bulk rename 0.1
Chathura C. <...@gmail.com>
Renames files in bulk by delegating renaming to an editor

USAGE:
    rename [FLAGS] [OPTIONS] [directory]

FLAGS:
    -E               Whether to exclude directories
    -h, --help       Prints help information
    -R               Rename in subdirectories recursively
    -O               Sorting descending order
    -V, --version    Prints version information

OPTIONS:
    -n <depth>               Specify sub-directory depth for recursive option
    -e, --editor <editor>    Specify the custom editor for editing file names
    -l <left>                Specify the left input to rename from
    -m <mode>                Specify the renaming mode - directory, stdin, left or diff
    -r <right>               Specify the right input to rename to
    -s <sort>                Specify the sorting mode - none (default), alph or dir

ARGS:
    <directory>    Specify the directory to rename files in
```

## Directory Mode

Directory mode is the default mode. One can explicitly specify directory mode by `-m` option specifying `dir`.

To rename all the files in the current directory with default editor (vim).
```
rename .
```

Specify the editor by giving `-e` option,
```
rename . -e nano
```

## Diff Mode

TODO: Write up!

