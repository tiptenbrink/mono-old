#!/bin/bash
# we want to auto-export all environment variables we set so docker compose can use them
set -a
echo "Waiting for secrets..."
while [ true ] 
do 
    # if file exists and is named pipe
    if [ -p "$1" ]; then
        . $1
        if [ -n "$TIDPLOY_READY" ]; then
            echo "Starting...."
            # ensure we have the latest version of our images
            docker compose --env-file production.env pull
            # here we start our compose file as before
            docker compose --env-file production.env -p hellodeploy up -d
            break
        else
            echo "Secrets loaded."
        fi
    # if pipe doesn't exist we don't want to run too many loops
    else
        sleep 1
    fi
done