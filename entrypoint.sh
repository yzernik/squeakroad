#!/usr/bin/env bash

# exit from script if error was raised.
set -e

# Start using the run server command
if [ -f config.ini ]; then
    exec squeaknode \
	 --config config.ini \
	 --log-level INFO
else
    exec squeaknode \
	 --log-level INFO
fi
