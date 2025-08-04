#!/bin/bash

# MultiQueue2 Documentation Update Script
# Simple version: just copy criterion reports and create index

set -e

# Parse command line arguments
RUN_BENCHMARKS=true
for arg in "$@"; do
    case $arg in
        --no-bench|--skip-bench)
            RUN_BENCHMARKS=false
            shift
            ;;
        -h|--help)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --no-bench, --skip-bench    Skip running benchmarks, just generate docs from existing reports"
            echo "  -h, --help                  Show this help message"
            echo ""
            echo "Examples:"
            echo "  $0                          Run benchmarks and generate docs"
            echo "  $0 --no-bench              Generate docs from existing benchmark reports"
            exit 0
            ;;
        *)
            echo "Unknown option: $arg"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

echo "🚀 Starting benchmark documentation update..."

DOCS_DIR="docs"
CRITERION_DIR="target/criterion"
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

cd "$PROJECT_ROOT"

if [ "$RUN_BENCHMARKS" = true ]; then
    echo "📊 Running benchmarks..."
    RUSTFLAGS='-C target-cpu=native' cargo bench
else
    echo "📊 Skipping benchmark run (using existing reports)..."
fi

echo "📋 Copying Criterion reports..."

# Clean and recreate docs directory
rm -rf "$DOCS_DIR"
mkdir -p "$DOCS_DIR"

# Copy entire criterion directory
if [ -d "$CRITERION_DIR" ]; then
    cp -r "$CRITERION_DIR"/* "$DOCS_DIR/"
    echo "✅ Copied all benchmark reports to docs/"
else
    echo "❌ No Criterion reports found at $CRITERION_DIR"
    exit 1
fi

# Generate index.html
cat > "$DOCS_DIR/index.html" << 'EOF'
<!DOCTYPE html>
<html>
<head>
    <title>MultiQueue2 Benchmarks</title>
    <style>
        body { 
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
            max-width: 800px; 
            margin: 40px auto; 
            padding: 20px;
            background: #fafafa;
        }
        .container {
            background: white;
            padding: 40px;
            border-radius: 8px;
            box-shadow: 0 2px 10px rgba(0,0,0,0.1);
        }
        h1 { 
            color: #2c3e50; 
            border-bottom: 2px solid #3498db;
            padding-bottom: 10px;
        }
        h2 { 
            color: #34495e; 
            margin-top: 30px;
        }
        .benchmark-list {
            display: grid;
            gap: 15px;
            margin: 20px 0;
        }
        .benchmark-item {
            background: #f8f9fa;
            border: 1px solid #e9ecef;
            border-radius: 6px;
            padding: 15px;
            transition: all 0.2s ease;
        }
        .benchmark-item:hover {
            background: #e3f2fd;
            border-color: #2196f3;
            transform: translateY(-1px);
        }
        .benchmark-item a {
            text-decoration: none;
            color: #1976d2;
            font-weight: 500;
        }
        .benchmark-item a:hover {
            color: #0d47a1;
        }
        .description {
            color: #6c757d;
            font-size: 0.9em;
            margin-top: 5px;
        }
        .footer {
            margin-top: 40px;
            padding-top: 20px;
            border-top: 1px solid #dee2e6;
            color: #6c757d;
            font-size: 0.9em;
        }
        .stats {
            background: #e8f5e8;
            border-left: 4px solid #28a745;
            padding: 15px;
            margin: 20px 0;
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>🚀 MultiQueue2 Benchmarks</h1>
        
        <div class="stats">
            <strong>Performance Overview:</strong>
            <ul>
                <li>~300ns per operation for SPSC scenarios</li>
                <li>~400ns per operation for MPMC scenarios</li>
                <li>No allocations on push/pop operations</li>
            </ul>
        </div>

        <h2>📊 Benchmark Reports</h2>
        <div class="benchmark-list">
EOF

# Generate benchmark links dynamically
for dir in "$DOCS_DIR"/*; do
    if [ -d "$dir" ] && [ -f "$dir/report/index.html" ]; then
        benchmark_name=$(basename "$dir")
        
        # Create description based on benchmark name
        case "$benchmark_name" in
            "SPSC_Comparison")
                description="Compares MPMC vs Broadcast queues in single-producer single-consumer scenarios"
                ;;
            "MPMC_Queue")
                description="Tests various MPMC producer/consumer combinations (1p/1c, 1p/2c, 2p/1c, etc.)"
                ;;
            "Broadcast_Queue")
                description="Tests broadcast functionality with multiple streams and consumers"
                ;;
            "Futures_Queue")
                description="Benchmarks async/await performance using tokio"
                ;;
            "Throughput_Variations")
                description="Performance impact of different buffer sizes and configurations"
                ;;
            *)
                description="Detailed performance analysis and measurements"
                ;;
        esac
        
        cat >> "$DOCS_DIR/index.html" << EOF
            <div class="benchmark-item">
                <a href="$benchmark_name/report/index.html">$benchmark_name</a>
                <div class="description">$description</div>
            </div>
EOF
    fi
done

# Complete the HTML
cat >> "$DOCS_DIR/index.html" << 'EOF'
        </div>

        <h2>🔧 How to Run Benchmarks</h2>
        <pre style="background: #f8f9fa; padding: 15px; border-radius: 4px; overflow-x: auto;"><code># Run all benchmarks
cargo bench

# Run with native CPU optimizations  
RUSTFLAGS='-C target-cpu=native' cargo bench

# Run specific categories
cargo bench spsc
cargo bench mpmc
cargo bench broadcast
cargo bench futures</code></pre>

        <div class="footer">
            <p><strong>MultiQueue2</strong> - Fast MPMC broadcast queue implementation in Rust</p>
            <p>
                <a href="https://github.com/abbychau/multiqueue2">GitHub</a> | 
                <a href="https://docs.rs/multiqueue2">API Docs</a> | 
                <a href="https://crates.io/crates/multiqueue2">Crates.io</a>
            </p>
EOF

echo "            <p>Last updated: $(date)</p>" >> "$DOCS_DIR/index.html"

cat >> "$DOCS_DIR/index.html" << 'EOF'
        </div>
    </div>
</body>
</html>
EOF

echo "✅ Created index.html with links to all benchmark reports"
echo ""
echo "📋 Summary:"
echo "  • Copied all Criterion reports to docs/"
echo "  • Created docs/index.html with navigation"
echo "  • Ready for GitHub Pages deployment"
echo ""
echo "🎉 Documentation update complete!"