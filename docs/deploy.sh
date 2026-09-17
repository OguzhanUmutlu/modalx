#!/usr/bin/env bash
# ==============================================================================
# modalx Documentation Deployment Script
# Builds and deploys documentation to https://modalx.oguzhanumutlu.com
# ==============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "${SCRIPT_DIR}"

echo "[1/2] Building VitePress documentation..."
npm run docs:build

echo "[2/2] Deploying to Cloudflare Pages (modalx.oguzhanumutlu.com)..."
npx wrangler deploy

echo "Done! modalx documentation is live at https://modalx.oguzhanumutlu.com"
