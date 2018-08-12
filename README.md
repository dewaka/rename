# Renaming Tool

This tool renames stuff.

## Todo

1. Write directory contents to a temporary file.
2. Open the file using system editor, and wait for the exit signal.
3. Depending on the exit signal, continue to process.
4. If the exit signal indicates success, then read the temporary file.
   a. Read all the contents of the temp file into a vector of strings.
   b. If they have same amount of files as the original vector from memory,
      then continue to renaming step.
