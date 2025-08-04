#!/bin/bash

# MultiQueue2 Documentation Update Script
# This script runs benchmarks and copies Criterion reports to GitHub Pages docs

set -e

echo "🚀 Starting benchmark documentation update..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
DOCS_DIR="docs/benchmarks"
CRITERION_DIR="target/criterion"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

cd "$PROJECT_ROOT"

echo -e "${BLUE}📊 Running benchmarks...${NC}"

# Run benchmarks with native CPU optimizations
RUSTFLAGS='-C target-cpu=native' cargo bench

echo -e "${GREEN}✅ Benchmarks completed${NC}"

# Check if criterion reports exist
if [ ! -d "$CRITERION_DIR" ]; then
    echo -e "${RED}❌ No Criterion reports found at $CRITERION_DIR${NC}"
    echo "Make sure benchmarks ran successfully"
    exit 1
fi

echo -e "${BLUE}📋 Processing Criterion reports...${NC}"

# Create benchmark docs directory structure
mkdir -p "$DOCS_DIR/reports"

# Function to copy and process a single benchmark report
process_benchmark() {
    local bench_name="$1"
    local source_dir="$CRITERION_DIR/$bench_name"
    local dest_dir="$DOCS_DIR/reports/$bench_name"
    
    if [ -d "$source_dir" ]; then
        echo "  📄 Processing $bench_name..."
        
        # Create destination directory
        mkdir -p "$dest_dir"
        
        # Copy the entire report directory
        cp -r "$source_dir"/* "$dest_dir/"
        
        # Create a markdown file linking to the HTML report
        cat > "$dest_dir/index.md" << EOF
---
title: $bench_name Benchmark Report
---

# $bench_name Benchmark Report

[View Interactive HTML Report](report/index.html)

This benchmark report was generated using Criterion.rs and includes:
- Throughput measurements
- Latency distributions  
- Regression analysis
- Performance comparisons over time

## Quick Stats

The detailed statistics and interactive charts are available in the [HTML report](report/index.html).

---

*Last updated: $(date)*
EOF
        
        return 0
    else
        echo -e "${YELLOW}  ⚠️  Skipping $bench_name (not found)${NC}"
        return 1
    fi
}

# Process all common benchmark categories
benchmarks_found=0

# List all available benchmarks
echo -e "${BLUE}📁 Available benchmark reports:${NC}"
for dir in "$CRITERION_DIR"/*; do
    if [ -d "$dir" ]; then
        bench_name=$(basename "$dir")
        echo "  - $bench_name"
        if process_benchmark "$bench_name"; then
            ((benchmarks_found++))
        fi
    fi
done

if [ $benchmarks_found -eq 0 ]; then
    echo -e "${RED}❌ No benchmark reports were processed${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Processed $benchmarks_found benchmark reports${NC}"

# Update the main benchmarks index page
echo -e "${BLUE}📝 Updating benchmark index...${NC}"

# Generate links to all processed benchmarks
benchmark_links=""
for dir in "$DOCS_DIR/reports"/*; do
    if [ -d "$dir" ]; then
        bench_name=$(basename "$dir")
        benchmark_links="$benchmark_links
- [$bench_name](reports/$bench_name/) - [HTML Report](reports/$bench_name/report/index.html)"
    fi
done

# Update the benchmarks index page
cat > "$DOCS_DIR/index.md" << EOF
# MultiQueue2 Benchmarks

This page contains detailed performance benchmarks for MultiQueue2, generated using Criterion.rs.

## Benchmark Categories

### SPSC Comparison
Compares MPMC vs Broadcast queues in single-producer single-consumer scenarios.

### MPMC Queue
Tests various producer/consumer combinations (1p/1c, 1p/2c, 2p/1c, etc.).

### Broadcast Queue
Tests broadcast functionality with multiple streams and consumers.

### Futures Queue
Benchmarks async/await performance using tokio.

## How to Run Benchmarks

\`\`\`bash
# Run all benchmarks
cargo bench

# Run specific benchmark suites
cargo bench --bench quick_benchmark      # Quick benchmarks (smaller dataset)
cargo bench --bench multiqueue_benchmarks # Comprehensive benchmarks

# Run specific benchmark categories
RUSTFLAGS='-C target-cpu=native' cargo bench spsc
RUSTFLAGS='-C target-cpu=native' cargo bench mpmc
RUSTFLAGS='-C target-cpu=native' cargo bench broadcast
RUSTFLAGS='-C target-cpu=native' cargo bench futures
\`\`\`

## Benchmark Reports

The benchmark reports below are automatically generated from the latest Criterion output. Each report includes throughput measurements, latency distributions, and regression analysis.

$benchmark_links

---

*Last updated: $(date)*
*System: $(uname -sm)*
*Rust version: $(rustc --version)*
EOF

echo -e "${GREEN}✅ Documentation updated successfully!${NC}"
echo ""
echo -e "${BLUE}📋 Summary:${NC}"
echo -e "  • Processed ${GREEN}$benchmarks_found${NC} benchmark reports"
echo -e "  • Reports available in: ${BLUE}$DOCS_DIR/reports/${NC}"
echo -e "  • Main index updated: ${BLUE}$DOCS_DIR/index.md${NC}"
echo ""
echo -e "${YELLOW}💡 Next steps:${NC}"
echo -e "  • Commit and push changes to enable GitHub Pages"
echo -e "  • View docs at: ${BLUE}https://[username].github.io/multiqueue2/benchmarks/${NC}"
echo ""
echo -e "${GREEN}🎉 Documentation update complete!${NC}"