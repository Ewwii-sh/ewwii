#!/bin/sh
while true; do
    # read the last line
    if [ -s /tmp/selected_emoji.txt ]; then
        tail -n 1 /tmp/selected_emoji.txt
        # truncate the file after reading
        : > /tmp/selected_emoji.txt
    fi
    sleep 0.1
done
