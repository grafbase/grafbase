## Features

- Added support for CLI plugins (https://github.com/grafbase/grafbase/pull/3133).

  Any binary in your `$PATH` that starts with `grafbase-` will be automatically detected and usable as a subcommand. For example, if you have the `grafbase-postgres` plugin installed, you can run it with `grafbase postgres`. Subsequent arguments are forwarded to the plugin. A new `grafbase list-plugins` command is also introduced. If you are familiar with these, this is similar to how cargo and git plugins work.
