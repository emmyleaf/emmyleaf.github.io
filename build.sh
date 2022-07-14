#!/bin/sh
set -e
cd "$(dirname $0)/leafcodes_ssg"
exec cargo run --release
