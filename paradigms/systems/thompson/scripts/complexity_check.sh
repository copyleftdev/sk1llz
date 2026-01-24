#!/bin/bash
# complexity_check.sh
# Thompson-style complexity analysis
# Flags code that violates Unix philosophy principles

set -euo pipefail

usage() {
    echo "Usage: $0 <source_file_or_directory>"
    echo "Analyzes code for Thompson-style simplicity violations"
    exit 1
}

[[ $# -lt 1 ]] && usage

TARGET="$1"
WARNINGS=0

warn() {
    echo "⚠️  $1"
    ((WARNINGS++))
}

info() {
    echo "ℹ️  $1"
}

# Check file length (Thompson: small programs)
check_file_length() {
    local file="$1"
    local lines
    lines=$(wc -l < "$file")
    
    if [[ $lines -gt 500 ]]; then
        warn "$file: $lines lines - consider splitting (Thompson: small, focused programs)"
    elif [[ $lines -gt 300 ]]; then
        info "$file: $lines lines - getting large"
    fi
}

# Check function length
check_function_length() {
    local file="$1"
    
    # For C files
    if [[ "$file" == *.c || "$file" == *.h ]]; then
        awk '
        /^[a-zA-Z_][a-zA-Z0-9_]*\s*\(/ { fname=$1; start=NR; in_func=1 }
        in_func && /^}/ { 
            len = NR - start
            if (len > 50) print "Function too long: " fname " (" len " lines)"
            in_func=0
        }
        ' "$file"
    fi
}

# Check for complexity indicators
check_complexity() {
    local file="$1"
    
    # Deeply nested code (Thompson: flat is better)
    if grep -n '^\s\{16,\}' "$file" > /dev/null 2>&1; then
        warn "$file: Deep nesting detected - flatten control flow"
    fi
    
    # Long lines (Thompson: clarity)
    if awk 'length > 100' "$file" | grep -q .; then
        warn "$file: Lines over 100 chars - consider breaking up"
    fi
    
    # Magic numbers
    if grep -E '\b[0-9]{4,}\b' "$file" | grep -v -E '(0x|year|date|port)' > /dev/null 2>&1; then
        info "$file: Large magic numbers detected - consider named constants"
    fi
}

# Check for Unix philosophy violations
check_unix_philosophy() {
    local file="$1"
    
    # Binary format indicators (Thompson: text streams)
    if grep -l 'fwrite.*sizeof\|struct.*__attribute__.*packed' "$file" > /dev/null 2>&1; then
        info "$file: Binary I/O detected - consider text format if possible"
    fi
    
    # Global state (Thompson: explicit is better)
    if grep -c '^static [^(]*;' "$file" 2>/dev/null | grep -v '^0$' > /dev/null; then
        info "$file: Static globals detected - consider passing explicitly"
    fi
}

# Main analysis
analyze_file() {
    local file="$1"
    
    [[ -f "$file" ]] || return
    
    case "$file" in
        *.c|*.h|*.go|*.py|*.sh|*.js)
            check_file_length "$file"
            check_complexity "$file"
            check_unix_philosophy "$file"
            ;;
    esac
}

# Process target
if [[ -d "$TARGET" ]]; then
    find "$TARGET" -type f \( -name "*.c" -o -name "*.h" -o -name "*.go" -o -name "*.py" -o -name "*.sh" \) | while read -r file; do
        analyze_file "$file"
    done
else
    analyze_file "$TARGET"
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Thompson Simplicity Check Complete"
echo "Warnings: $WARNINGS"
echo ""
echo "Remember:"
echo "  • When in doubt, use brute force"
echo "  • Small programs that do one thing well"
echo "  • Text streams as universal interface"
echo "  • Throwing away code can be productive"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
