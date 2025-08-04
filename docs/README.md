# MultiQueue2 Documentation

This directory contains the GitHub Pages documentation for MultiQueue2.

## Structure

- `index.md` - Main documentation page
- `benchmarks/` - Benchmark reports and documentation
- `_config.yml` - Jekyll configuration for GitHub Pages

## Updating Documentation

To update the documentation with the latest benchmark results:

```bash
# Run the update script
./scripts/update-docs.sh
```

This will:
1. Run all benchmarks with native CPU optimizations
2. Copy Criterion HTML reports to `docs/benchmarks/reports/`
3. Generate markdown index pages for each benchmark
4. Update the main benchmarks index with links to all reports

## GitHub Pages Setup

The documentation is automatically deployed via GitHub Actions when pushing to the `master` branch.

To enable GitHub Pages for this repository:
1. Go to repository Settings → Pages
2. Select "GitHub Actions" as the source
3. The site will be available at `https://[username].github.io/multiqueue2/`

## Manual Local Testing

You can test the documentation locally using Jekyll:

```bash
# Install Jekyll (if not already installed)
gem install jekyll bundler

# Serve locally
cd docs
jekyll serve

# Visit http://localhost:4000
```