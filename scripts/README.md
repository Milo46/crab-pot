# Scripts Directory

This directory contains utility scripts for the crab-pot project.

## 📋 TODO Scanners

### Basic Scanner: `todo-scanner.sh`

Simple, colorful TODO scanner with basic functionality.

```bash
# Run from anywhere in the repository
./scripts/todo-scanner.sh

# Search for specific patterns
./scripts/todo-scanner.sh "todo|fix" "milo"
./scripts/todo-scanner.sh "urgent" "john"

# Show help
./scripts/todo-scanner.sh --help
```

### Advanced Scanner: `todo-advanced.sh`

Feature-rich scanner with multiple output formats.

```bash
# Text output (default)
./scripts/todo-advanced.sh

# JSON output
./scripts/todo-advanced.sh --format json

# CSV output to file
./scripts/todo-advanced.sh --format csv --output reports/todos.csv

# Filter by author and type
./scripts/todo-advanced.sh --author john --type "fix|bug"

# Show help
./scripts/todo-advanced.sh --help
```

## 🎯 TODO Format

Use this format in your code comments:

```
@{TYPE}(AUTHOR): Description
```

### Examples:

```rust
// @{todo}(milo): Implement user authentication
// @{fix}(milo): Handle errors properly instead of unwrap()
// @{hack}(milo): Temporary workaround, needs refactoring
// @{bug}(milo): Race condition in concurrent requests
// @{urgent}(milo): Security vulnerability needs immediate fix
```

```sql
-- @{todo}(milo): Add indexes for better performance
-- @{fix}(milo): Handle NULL values in user_email column
```

```dockerfile
# @{hack}(milo): Using latest tag, should pin to specific version
# @{todo}(milo): Add multi-stage build optimization
```

## ✨ Features

- **🔍 Repository-wide search** - Works from any directory
- **🎨 Color-coded output** by TODO type
- **📊 Multiple formats** - Text, JSON, CSV
- **👤 Author filtering** - Track TODOs by person
- **📁 Smart exclusions** - Skips build dirs, scripts, logs
- **📈 Summary statistics** - Count and breakdown
- **🔄 Git integration** - Auto-detects repository root

## 🚀 Integration

### Git Hooks

Add to `.git/hooks/pre-commit`:
```bash
#!/bin/bash
echo "Checking TODOs..."
./scripts/todo-scanner.sh
```

### CI/CD Pipeline

```yaml
- name: Generate TODO Report
  run: |
    ./scripts/todo-advanced.sh --format json --output reports/todos.json
    ./scripts/todo-advanced.sh --format csv --output reports/todos.csv
```

### Make/Justfile

```makefile
todos:
	@./scripts/todo-scanner.sh

todos-json:
	@./scripts/todo-advanced.sh --format json

todos-report:
	@./scripts/todo-advanced.sh --format csv --output reports/todos-$(shell date +%Y%m%d).csv
```
