#!/bin/bash
set -euo pipefail
ENVIRONMENT=${1:-production}
echo "=== Deploying OMNIVERSA to ${ENVIRONMENT} ==="
docker-compose -f infra/docker/docker-compose.yml build
cd infra/terraform
terraform workspace select ${ENVIRONMENT} || terraform workspace new ${ENVIRONMENT}
terraform apply -auto-approve
echo "=== Deployment Complete ==="
