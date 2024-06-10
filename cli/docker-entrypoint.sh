#!/bin/sh

set -eu

# allow the container to be started with `--user`
if [ "$(id -u)" = '0' ]; then
    mkdir -p .grafbase
    chmod 777 .grafbase
    find .grafbase \! -user grafbase -exec chmod 777 '{}' +
    exec gosu grafbase "$0" "$@"
fi

# set an appropriate umask (if one isn't set already)
# - https://github.com/docker-library/redis/issues/305
# - https://github.com/redis/redis/blob/bb875603fb7ff3f9d19aad906bd45d7db98d9a39/utils/systemd-redis_server.service#L37
um="$(umask)"
if [ "$um" = '0022' ]; then
    umask 0077
fi

# Add grafbase as first command
if [ "${1:-}" != "/bin/grafbase" ]; then
    set -- "/bin/grafbase" "$@"
fi

exec "$@"
