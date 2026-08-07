# Deployment Guide

## Prerequisites
- Rust 1.78+
- Python 3.11+
- Docker 24.0+
- Terraform 1.7+

## Local Development
```bash
cargo build --release
cargo test --workspace
docker-compose -f infra/docker/docker-compose.yml up -d
cd python/orchestrator && pip install -r requirements.txt && python main.py
```

## Production Deployment
```bash
cd infra/terraform
terraform init
terraform plan
terraform apply
aws eks update-kubeconfig --region us-east-1 --name omniversa-sovereign
kubectl apply -f k8s/
```
