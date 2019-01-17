# Name
  spellhold - a daemon to capture stdout and displays it on command

## Synopsis
```
  spellcli

  USAGE:
      spellcli [FLAGS] [SUBCOMMAND]

  FLAGS:
      -h, --help       Prints help information
      -q, --quite      whether should run quite
      -V, --version    Prints version information

  SUBCOMMANDS:
      daemon     [aliases: d]
      help      Prints this message or the help of the given subcommand(s)
      stdin      [aliases: s]
      tui        [aliases: t]


```

## Description
  spellhold will take stdout from a command and both log it to a file then
  display it to the client if connected


## Example
  this will log to the date and time it was run
    rsync -r dir/path/ to/path | spellhold s

  this will log to /path/to/logs/rsync_cmd
    rsync -r dir/path/ to/path | spellhold stdout -n rsync_cmd
