#!/bin/bash
# Setup script for Python benchmarks using uv

set -e

echo "Setting up Python benchmarks for seams comparison..."

# Check if uv is installed
if ! command -v uv &> /dev/null; then
    echo "ERROR: uv is not installed. Install it with:"
    echo "curl -LsSf https://astral.sh/uv/install.sh | sh"
    exit 1
fi

# Create virtual environment
echo "Creating virtual environment..."
uv venv

# Install dependencies
echo "Installing Python dependencies..."
uv pip install -e .

echo "Setup complete!"
echo ""
echo "To run benchmarks:"
echo "  source .venv/bin/activate"
echo "  python run_comparison.py /path/to/gutenberg/files"