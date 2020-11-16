#!/usr/bin/env bash

TRACKER_BASE="$HOME/.tracker"
TRACKER_INIT="$TRACKER_BASE/.tracker.rc"

if [[ ! -d "$TRACKER_BASE" ]]; then
    mkdir "$TRACKER_BASE"
fi

cp ./startup-files/tracker.rc "$TRACKER_INIT"

cargo install --path .

echo "Installed. Please run 'tracker'."