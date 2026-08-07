terraform {
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
  }
}

provider "aws" {
  region = var.aws_region
}

module "eks" {
  source  = "terraform-aws-modules/eks/aws"
  version = "~> 20.0"
  cluster_name    = "omniversa-sovereign"
  cluster_version = "1.29"
  vpc_id     = module.vpc.vpc_id
  subnet_ids = module.vpc.private_subnets
  eks_managed_node_groups = {
    sovereign_compute = {
      desired_size = 3
      min_size     = 2
      max_size     = 10
      instance_types = ["c6i.4xlarge"]
      capacity_type  = "ON_DEMAND"
    }
    sovereign_gpu = {
      desired_size = 2
      min_size     = 1
      max_size     = 4
      instance_types = ["g5.xlarge"]
      capacity_type  = "ON_DEMAND"
    }
  }
}

module "vpc" {
  source  = "terraform-aws-modules/vpc/aws"
  version = "~> 5.0"
  name = "omniversa-vpc"
  cidr = "10.0.0.0/16"
  azs             = ["${var.aws_region}a", "${var.aws_region}b", "${var.aws_region}c"]
  private_subnets = ["10.0.1.0/24", "10.0.2.0/24", "10.0.3.0/24"]
  public_subnets  = ["10.0.101.0/24", "10.0.102.0/24", "10.0.103.0/24"]
  enable_nat_gateway = true
}

resource "aws_s3_bucket" "tick_archive" {
  bucket = "omniversa-tick-archive-${var.environment}"
}

variable "aws_region" {
  default = "us-east-1"
}

variable "environment" {
  default = "production"
}
