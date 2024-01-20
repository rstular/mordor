#!/bin/bash

set -euo pipefail

SOURCE=${BASH_SOURCE[0]}
while [ -L "$SOURCE" ]; do # resolve $SOURCE until the file is no longer a symlink
    DIR=$(cd -P "$(dirname "$SOURCE")" >/dev/null 2>&1 && pwd)
    SOURCE=$(readlink "$SOURCE")
    [[ $SOURCE != /* ]] && SOURCE=$DIR/$SOURCE # if $SOURCE was a relative symlink, we need to resolve it relative to the path where the symlink file was located
done
DIR=$(cd -P "$(dirname "$SOURCE")" >/dev/null 2>&1 && pwd)

CURRENT_DIR=$(pwd)

cleanup() {
    cd "$CURRENT_DIR"
}
trap cleanup EXIT

cd "$DIR"

echo "Building Mordor image"
sudo docker build . -t mordor
echo "Saving Mordor image"
sudo docker save mordor | gzip >mordor.tar.gz
