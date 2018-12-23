# Name
  spellhold - a daemon to capture stdout and displays it on command

## Synopsis
  spellhold -l

## Description
  spellhold wil ltake stdout from a command and both log it to a file trhen
  display it to the client if connected

## Options
  -l --log-file=FILE    write to a named log file

## Example
  this will log to the date and time it was run
    rsync -r dir/path/ to/path | spellhold

  this will log to
    rsync -r dir/path/ to/path | spellhold -l log_name
