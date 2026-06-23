terraform {
  required_version = ">= 1.6.0"

  required_providers {
    cloudflare = {
      source  = "cloudflare/cloudflare"
      version = "~> 4.0"
    }
  }
}

variable "account_id" {
  description = "Cloudflare account ID that owns the zone and Pages project."
  type        = string
}

variable "zone_name" {
  description = "Apex zone the site lives under."
  type        = string
  default     = "alkem.dev"
}

variable "pages_project_name" {
  description = "Cloudflare Pages project name."
  type        = string
  default     = "grim"
}

variable "subdomain" {
  description = "Subdomain record name within the zone."
  type        = string
  default     = "grim"
}

variable "pages_custom_domain" {
  description = "Custom domain bound to the Pages project."
  type        = string
  default     = "grim.alkem.dev"
}

variable "production_branch" {
  description = "Production branch Cloudflare builds and serves."
  type        = string
  default     = "main"
}

variable "github_owner" {
  description = "GitHub org/user that owns the repo Cloudflare builds from."
  type        = string
  default     = "alkemdev"
}

variable "github_repo" {
  description = "GitHub repo name."
  type        = string
  default     = "grim"
}

# Reads the API token from CLOUDFLARE_API_TOKEN in the environment.
provider "cloudflare" {}

data "cloudflare_zone" "apex" {
  name = var.zone_name
}

# Git-integrated Pages project: Cloudflare itself builds the Astro site from the repo and deploys on
# every push to the production branch. No GitHub Actions are involved — Cloudflare does the building.
resource "cloudflare_pages_project" "site" {
  account_id        = var.account_id
  name              = var.pages_project_name
  production_branch = var.production_branch

  source {
    type = "github"
    config {
      owner             = var.github_owner
      repo_name         = var.github_repo
      production_branch = var.production_branch
    }
  }

  build_config {
    # The site lives in web/ (Astro + Starlight). `npm run build` runs the docs-sync, then astro
    # build, emitting web/dist.
    root_dir        = "web"
    build_command   = "npm ci && npm run build"
    destination_dir = "dist"
  }
}

resource "cloudflare_pages_domain" "site" {
  account_id   = var.account_id
  project_name = cloudflare_pages_project.site.name
  domain       = var.pages_custom_domain
}

# grim.alkem.dev -> <project>.pages.dev, proxied through Cloudflare.
resource "cloudflare_record" "site" {
  zone_id = data.cloudflare_zone.apex.id
  type    = "CNAME"
  name    = var.subdomain
  content = cloudflare_pages_project.site.subdomain
  proxied = true
  ttl     = 1
}

output "pages_project_name" {
  value = cloudflare_pages_project.site.name
}

output "pages_project_subdomain" {
  value = cloudflare_pages_project.site.subdomain
}

output "custom_domain" {
  value = var.pages_custom_domain
}

output "custom_domain_status" {
  value = cloudflare_pages_domain.site.status
}
