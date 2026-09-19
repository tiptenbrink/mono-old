#!/bin/bash

# Define function to stop the appp gracefully
stop_app() {
    echo "Stopping app.."
    kill $app_pid
    exit 0
}

# Trap SIGTERM signal and run stop_app function
trap 'stop_app' SIGTERM

node app.js &
app_pid=$!  # Save the PID of app process

wait "$app_pid"  # Wait for app