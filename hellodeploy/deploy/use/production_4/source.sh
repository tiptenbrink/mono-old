#!/bin/bash
# we want to auto-export all environment variables we set so docker compose can use them
set -a
echo "Waiting for secrets..."
while [ true ] 
do 
    # -p means if file exists and is named pipe
    # $1 is first argument to our script
    if [ -p "$1" ]; then
        echo "Loaded secret."
        . $1
    else
        sleep 1
    fi

    if [ -n "$TIDPLOY_READY" ]; then
        echo "Starting...."
        # ensure we have the latest version of our images
        docker compose pull
        # here we start our compose file as before
        docker compose -p hellodeploy up -d
        break
    fi
done