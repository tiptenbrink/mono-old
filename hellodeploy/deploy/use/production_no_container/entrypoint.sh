#!/bin/bash
set -a
secrets=$(nu secrets.nu)
# $? contains the exit code of the subprocess called above
# (( )) is arithmetic expression, and 0 evaluates to false while 1 (or other numbers) evaluate to true
(( $? )) && echo "${secrets}" && exit 1
eval "$secrets"
docker compose pull
docker compose -p hellodeploy up -d