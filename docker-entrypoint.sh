#!/bin/sh
set -eu

if [ "$(id -u)" = "0" ]; then
  mkdir -p "$NUOFIELD_DATA_DIR"
  chown nuofield:nuofield "$NUOFIELD_DATA_DIR"
  exec gosu nuofield "$@"
fi

exec "$@"
