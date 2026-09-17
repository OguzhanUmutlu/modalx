#!/usr/bin/env bash
# ==============================================================================
# modalx Documentation Build & Deployment Script (GitHub Pages)
# https://modalx.oguzhanumutlu.com
# ==============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "${SCRIPT_DIR}"

echo "==> Building VitePress documentation..."
npm run docs:build

echo ""
echo "=================================================================="
echo "[OK] modalx documentation built successfully in .vitepress/dist/"
echo "=================================================================="
echo " Target domain: https://modalx.oguzhanumutlu.com"
echo " Deploy engine: GitHub Actions (deploy-docs.yml)"
echo " Notice: Deployments are automated on git push to main."
echo "=================================================================="
