## Improvements

- The assets directory in ~/.grafbase is now cleaned up every time a new assets version is unpacked.

## Fixes

- `grafbase list-plugins` would list plugins that were installed in multiple directories in `$PATH` multiple times. That list is now dedpuplicated. (https://github.com/grafbase/grafbase/pull/3140)
