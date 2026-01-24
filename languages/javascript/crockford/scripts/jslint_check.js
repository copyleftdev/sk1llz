#!/usr/bin/env node
/**
 * jslint_check.js
 * A simplified Crockford-style JavaScript code checker.
 * Checks for patterns that Crockford considers harmful.
 *
 * Usage:
 *   node jslint_check.js file.js
 *   node jslint_check.js --demo
 */

const fs = require('fs');
const path = require('path');

const CHECKS = [
    {
        name: 'Use of ==',
        pattern: /[^=!<>]==[^=]/g,
        message: 'Use === instead of == for comparison',
        severity: 'error'
    },
    {
        name: 'Use of !=',
        pattern: /!=[^=]/g,
        message: 'Use !== instead of != for comparison',
        severity: 'error'
    },
    {
        name: 'Use of eval',
        pattern: /\beval\s*\(/g,
        message: 'eval is evil - avoid using it',
        severity: 'error'
    },
    {
        name: 'Use of with',
        pattern: /\bwith\s*\(/g,
        message: 'with statement is deprecated and confusing',
        severity: 'error'
    },
    {
        name: 'Missing strict mode',
        pattern: null, // Special handling
        message: "Add 'use strict'; at the beginning",
        severity: 'warning'
    },
    {
        name: 'Increment/Decrement operators',
        pattern: /\+\+|--/g,
        message: 'Consider using += 1 or -= 1 instead of ++ or --',
        severity: 'warning'
    },
    {
        name: 'new Array()',
        pattern: /new\s+Array\s*\(/g,
        message: 'Use array literal [] instead of new Array()',
        severity: 'warning'
    },
    {
        name: 'new Object()',
        pattern: /new\s+Object\s*\(/g,
        message: 'Use object literal {} instead of new Object()',
        severity: 'warning'
    },
    {
        name: 'Trailing comma in object/array',
        pattern: /,\s*[\]\}]/g,
        message: 'Trailing commas can cause issues in older browsers',
        severity: 'warning'
    },
    {
        name: 'Global variable (implicit)',
        pattern: /^\s*[a-zA-Z_$][a-zA-Z0-9_$]*\s*=[^=]/gm,
        message: 'Possible implicit global variable',
        severity: 'warning'
    },
    {
        name: 'Missing semicolon',
        pattern: /[^;\s{}]\s*\n\s*(var|let|const|function|if|for|while|return)/g,
        message: 'Missing semicolon',
        severity: 'warning'
    },
    {
        name: 'Function as block',
        pattern: /\bif\s*\([^)]+\)\s*function\b/g,
        message: 'Do not use function declarations in blocks',
        severity: 'error'
    },
    {
        name: 'Bitwise operator in boolean context',
        pattern: /if\s*\([^)]*[&|][^&|][^)]*\)/g,
        message: 'Did you mean && or || instead of & or |?',
        severity: 'warning'
    },
    {
        name: 'Void operator',
        pattern: /\bvoid\s*\(/g,
        message: 'Avoid using void operator',
        severity: 'warning'
    },
    {
        name: 'Label statement',
        pattern: /^\s*[a-zA-Z_$][a-zA-Z0-9_$]*\s*:\s*(for|while|do|switch)/gm,
        message: 'Labels make code harder to understand',
        severity: 'warning'
    }
];

function getLineNumber(code, index) {
    return code.substring(0, index).split('\n').length;
}

function checkStrictMode(code) {
    const issues = [];
    if (!code.includes("'use strict'") && !code.includes('"use strict"')) {
        issues.push({
            name: 'Missing strict mode',
            line: 1,
            message: "Add 'use strict'; at the beginning of the file or function",
            severity: 'warning'
        });
    }
    return issues;
}

function analyzeCode(code, filename) {
    const issues = [];
    
    // Check for strict mode
    issues.push(...checkStrictMode(code));
    
    // Run pattern-based checks
    for (const check of CHECKS) {
        if (!check.pattern) continue;
        
        let match;
        const pattern = new RegExp(check.pattern.source, check.pattern.flags);
        
        while ((match = pattern.exec(code)) !== null) {
            const line = getLineNumber(code, match.index);
            issues.push({
                name: check.name,
                line: line,
                message: check.message,
                severity: check.severity,
                match: match[0].trim().substring(0, 20)
            });
        }
    }
    
    // Sort by line number
    issues.sort((a, b) => a.line - b.line);
    
    return issues;
}

function formatReport(filename, issues) {
    const errors = issues.filter(i => i.severity === 'error');
    const warnings = issues.filter(i => i.severity === 'warning');
    
    let output = '\n' + '='.repeat(60) + '\n';
    output += `CROCKFORD STYLE CHECK: ${filename}\n`;
    output += '='.repeat(60) + '\n\n';
    
    if (issues.length === 0) {
        output += '✓ No issues found. The Good Parts approved!\n';
    } else {
        output += `Found ${errors.length} errors and ${warnings.length} warnings\n\n`;
        
        for (const issue of issues) {
            const symbol = issue.severity === 'error' ? '✗' : '⚠';
            output += `${symbol} Line ${issue.line}: ${issue.name}\n`;
            output += `  ${issue.message}\n`;
            if (issue.match) {
                output += `  Found: "${issue.match}..."\n`;
            }
            output += '\n';
        }
    }
    
    output += '='.repeat(60) + '\n';
    output += `Summary: ${errors.length} errors, ${warnings.length} warnings\n`;
    
    if (errors.length === 0 && warnings.length === 0) {
        output += '★★★★★ Excellent - Crockford would approve!\n';
    } else if (errors.length === 0) {
        output += '★★★★☆ Good - Minor style issues\n';
    } else if (errors.length < 3) {
        output += '★★★☆☆ Fair - Some issues to fix\n';
    } else {
        output += '★★☆☆☆ Needs work - Read "JavaScript: The Good Parts"\n';
    }
    
    return output;
}

function demo() {
    const sampleCode = `
// Example JavaScript with style issues

var x = 1
var y = 2

// Bad: Using ==
if (x == "1") {
    console.log("equal");
}

// Bad: Using eval
var result = eval("1 + 2");

// Bad: Using ++
for (var i = 0; i < 10; i++) {
    console.log(i);
}

// Bad: new Array
var arr = new Array(1, 2, 3);

// Bad: new Object
var obj = new Object();

// Good: Using ===
if (x === 1) {
    console.log("strict equal");
}

// Good: Array literal
var goodArr = [1, 2, 3];

// Good: Object literal
var goodObj = {a: 1, b: 2};

// Module pattern (Crockford approved)
var counter = (function () {
    'use strict';
    var count = 0;
    return {
        increment: function () {
            count += 1;
            return count;
        },
        value: function () {
            return count;
        }
    };
}());
`;

    console.log('\n=== CROCKFORD STYLE CHECK DEMO ===\n');
    console.log('Analyzing sample JavaScript code...\n');
    
    const issues = analyzeCode(sampleCode, 'demo.js');
    console.log(formatReport('demo.js', issues));
}

function main() {
    const args = process.argv.slice(2);
    
    if (args.length === 0 || args[0] === '--help') {
        console.log(`Usage: node jslint_check.js <file.js>
       node jslint_check.js --demo`);
        return;
    }
    
    if (args[0] === '--demo') {
        demo();
        return;
    }
    
    const filename = args[0];
    
    if (!fs.existsSync(filename)) {
        console.error(`File not found: ${filename}`);
        process.exit(1);
    }
    
    const code = fs.readFileSync(filename, 'utf-8');
    const issues = analyzeCode(code, filename);
    console.log(formatReport(filename, issues));
    
    // Exit with error code if there are errors
    const errors = issues.filter(i => i.severity === 'error');
    if (errors.length > 0) {
        process.exit(1);
    }
}

main();
