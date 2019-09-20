#!/bin/sh

trap "exit" INT
set -eu

cd "$(dirname "$0")"

docker-compose -f stack.yml down
