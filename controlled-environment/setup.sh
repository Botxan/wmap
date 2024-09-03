#!/bin/bash
export TERM=xterm-color

if [ ! `which composer` ]; then
    echo "composer not found."
    exit 1;
fi

if [ ! `which php` ]; then
    echo "php not found."
    exit 1;
fi

if [ ! `which curl` ]; then
    echo "curl not found."
    exit 1;
fi

. ./config

echo "Target frameworks: $frameworks_list"

for fw in $param_targets
do
    if [ -d "$fw" ]; then
        echo -e "\n\n"
        echo "/------- $fw: setting up -------/"
        cd "$fw"
        if [ -f "_wmap/setup.sh" ]; then
            echo "Running setup for $fw"
            . "_wmap/setup.sh"
        else
            echo "Setup script for $fw not found"
        fi
        cd ..
    else
        echo "Directory $fw does not exist"
    fi
done

find . -name ".htaccess" -exec rm -rf {} \;
