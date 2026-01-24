#!/bin/bash
#
# kernel_style_check.sh
# Check C code against Linux Kernel coding style (Torvalds style)
#
# Usage:
#   ./kernel_style_check.sh file.c
#   ./kernel_style_check.sh --demo

set -e

# Colors for output
RED='\033[0;31m'
YELLOW='\033[1;33m'
GREEN='\033[0;32m'
NC='\033[0m' # No Color

print_header() {
    echo ""
    echo "============================================================"
    echo " LINUX KERNEL STYLE CHECK"
    echo " Based on Linus Torvalds' coding style"
    echo "============================================================"
    echo ""
}

check_file() {
    local file="$1"
    local errors=0
    local warnings=0
    
    echo "Checking: $file"
    echo "------------------------------------------------------------"
    
    # Check 1: Lines longer than 80 characters
    echo "Checking line length (max 80 chars)..."
    while IFS= read -r line_info; do
        if [ -n "$line_info" ]; then
            echo -e "${YELLOW}⚠ Line too long:${NC} $line_info"
            ((warnings++))
        fi
    done < <(awk 'length > 80 {print NR": "length" chars"}' "$file")
    
    # Check 2: Tabs vs spaces (kernel uses tabs for indentation)
    echo "Checking indentation (should use tabs)..."
    spaces_indent=$(grep -n '^ \{1,7\}[^ ]' "$file" 2>/dev/null | head -5)
    if [ -n "$spaces_indent" ]; then
        echo -e "${RED}✗ Using spaces instead of tabs:${NC}"
        echo "$spaces_indent" | head -3
        ((errors++))
    fi
    
    # Check 3: Space before opening brace
    echo "Checking brace style..."
    bad_brace=$(grep -n 'if.*){' "$file" 2>/dev/null | head -3)
    if [ -n "$bad_brace" ]; then
        echo -e "${RED}✗ Missing space before {:${NC}"
        echo "$bad_brace"
        ((errors++))
    fi
    
    # Check 4: else on same line as closing brace
    bad_else=$(grep -n '}\s*$' "$file" 2>/dev/null | while read -r line; do
        ln=$(echo "$line" | cut -d: -f1)
        next_ln=$((ln + 1))
        next_content=$(sed -n "${next_ln}p" "$file")
        if echo "$next_content" | grep -q '^\s*else'; then
            echo "Line $ln: else should be on same line as }"
        fi
    done)
    if [ -n "$bad_else" ]; then
        echo -e "${RED}✗ else should be: } else {${NC}"
        echo "$bad_else" | head -3
        ((errors++))
    fi
    
    # Check 5: Function brace on new line
    echo "Checking function definitions..."
    func_bad_brace=$(grep -n '^[a-zA-Z_][a-zA-Z0-9_]*.*(.*).*{' "$file" 2>/dev/null | head -3)
    if [ -n "$func_bad_brace" ]; then
        echo -e "${YELLOW}⚠ Function opening brace should be on new line:${NC}"
        echo "$func_bad_brace"
        ((warnings++))
    fi
    
    # Check 6: Trailing whitespace
    echo "Checking trailing whitespace..."
    trailing=$(grep -n ' $' "$file" 2>/dev/null | wc -l)
    if [ "$trailing" -gt 0 ]; then
        echo -e "${YELLOW}⚠ Found $trailing lines with trailing whitespace${NC}"
        ((warnings++))
    fi
    
    # Check 7: Multiple empty lines
    echo "Checking consecutive blank lines..."
    multi_blank=$(grep -n '^$' "$file" | awk -F: 'prev && $1 == prev + 1 {print prev+1} {prev=$1}' | wc -l)
    if [ "$multi_blank" -gt 0 ]; then
        echo -e "${YELLOW}⚠ Found $multi_blank instances of multiple blank lines${NC}"
        ((warnings++))
    fi
    
    # Check 8: goto labels indentation
    echo "Checking goto labels..."
    bad_goto=$(grep -n '^[a-zA-Z_][a-zA-Z0-9_]*:$' "$file" 2>/dev/null | head -3)
    # Labels should be at column 0 or indented one level less than code
    
    # Check 9: Naming conventions
    echo "Checking naming conventions..."
    camel_case=$(grep -oE '\b[a-z]+[A-Z][a-zA-Z]*\b' "$file" 2>/dev/null | sort -u | head -5)
    if [ -n "$camel_case" ]; then
        echo -e "${YELLOW}⚠ Possible CamelCase (use snake_case):${NC}"
        echo "$camel_case" | tr '\n' ' '
        echo ""
        ((warnings++))
    fi
    
    # Check 10: Comment style
    echo "Checking comment style..."
    cpp_comments=$(grep -n '//' "$file" 2>/dev/null | wc -l)
    if [ "$cpp_comments" -gt 0 ]; then
        echo -e "${YELLOW}⚠ Found $cpp_comments C++ style comments (// comments)${NC}"
        echo "  Kernel style prefers /* */ comments"
        ((warnings++))
    fi
    
    # Summary
    echo ""
    echo "============================================================"
    echo " SUMMARY"
    echo "============================================================"
    echo -e "Errors:   ${RED}$errors${NC}"
    echo -e "Warnings: ${YELLOW}$warnings${NC}"
    
    if [ $errors -eq 0 ] && [ $warnings -eq 0 ]; then
        echo -e "${GREEN}★★★★★ Excellent - Linus would approve!${NC}"
    elif [ $errors -eq 0 ]; then
        echo -e "${GREEN}★★★★☆ Good - Minor style issues${NC}"
    elif [ $errors -lt 3 ]; then
        echo -e "${YELLOW}★★★☆☆ Fair - Some issues to fix${NC}"
    else
        echo -e "${RED}★★☆☆☆ Needs work - Read Documentation/process/coding-style.rst${NC}"
    fi
    
    return $errors
}

demo() {
    print_header
    
    # Create a temporary file with example code
    local tmp_file
    tmp_file=$(mktemp /tmp/kernel_style_demo.XXXXXX.c)
    
    cat > "$tmp_file" << 'EOF'
/* Example C code with various style issues */

#include <stdio.h>

// This is a C++ style comment (bad)

int badCamelCase = 0;  /* CamelCase naming (bad) */

int foo(int x){        /* Missing space before { */
    if(x > 0){         /* Missing spaces */
        return x;
    }
}
else {                 /* else on wrong line */
    return 0;
}

/* Good function definition */
int
good_function(int value)
{
	if (value > 0) {
		return value;
	} else {
		return 0;
	}
}

/* This line is way too long and should be broken into multiple lines for better readability */

int main(void)
{
    int x = 1;         /* Spaces for indentation (should be tabs) */
    
    
    /* Multiple blank lines above (bad) */
    
    printf("Hello\n");
    return 0;
}
EOF

    echo "Demo file created with intentional style issues..."
    echo ""
    
    check_file "$tmp_file"
    
    # Cleanup
    rm -f "$tmp_file"
}

# Main
if [ $# -eq 0 ] || [ "$1" = "--help" ]; then
    echo "Usage: $0 <file.c>"
    echo "       $0 --demo"
    echo ""
    echo "Checks C code against Linux Kernel coding style."
    exit 0
fi

if [ "$1" = "--demo" ]; then
    demo
    exit 0
fi

if [ ! -f "$1" ]; then
    echo "Error: File not found: $1"
    exit 1
fi

print_header
check_file "$1"
