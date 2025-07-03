#!/usr/bin/env bash
set -e

echo "🔧 Setting up guten_split development environment..."

# Include repo-specific git aliases
git config --local include.path ../.gitaliases
echo "✅ Git aliases configured"

# Hide exploration tags from normal operations
git config --local transfer.hideRefs refs/tags/explore
echo "✅ Exploration tags hidden"

# Optional: Set up hooks path if .githooks exists
if [ -d ".githooks" ]; then
    git config --local core.hooksPath .githooks
    echo "✅ Git hooks configured"
fi

echo "🎉 Bootstrap complete! You can now use 'git finalize <commit>' for exploration workflows."
echo "📚 See docs/exploration-workflow.md for usage details."