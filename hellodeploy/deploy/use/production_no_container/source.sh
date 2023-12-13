#!/bin/bash
set -a
secrets=$(nu secrets.nu)
eval "$secrets"
docker compose pull
docker compose -p hellodeploy up -d