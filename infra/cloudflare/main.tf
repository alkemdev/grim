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
  description = "Production branch name (metadata for the project)."
  type        = string
  default     = "main"
}

# Reads the API token from CLOUDFLARE_API_TOKEN in the environment.
provider "cloudflare" {}

data "cloudflare_zone" "apex" {
  name = var.zone_name
}

# A Direct-Upload Pages project: assets are pushed with `wrangler pages deploy` (see README and the
# `site-deploy` just recipe). No git source and no build_config, so Cloudflare never tries to build
# from the repo — deploys are explicit, and there is no GitHub Actions in the loop.
resource "cloudflare_pages_project" "site" {
  account_id        = var.account_id
  name              = var.pages_project_name
  production_branch = var.production_branch
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
