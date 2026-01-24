#!/usr/bin/env python3
"""
fuzz_harness_template.py
Generate fuzzing harness templates for different languages and targets.

Usage:
    python fuzz_harness_template.py --lang cpp --target parser
    python fuzz_harness_template.py --lang python --target api
    python fuzz_harness_template.py --list
"""

import argparse
from dataclasses import dataclass
from typing import Dict


@dataclass
class HarnessTemplate:
    """Fuzzing harness template."""
    language: str
    target_type: str
    template: str
    build_instructions: str


TEMPLATES: Dict[str, Dict[str, HarnessTemplate]] = {
    "cpp": {
        "parser": HarnessTemplate(
            language="cpp",
            target_type="parser",
            template='''// fuzz_parser.cpp
// LibFuzzer harness for parser fuzzing
//
// Build:
//   clang++ -g -fsanitize=fuzzer,address -o fuzz_parser fuzz_parser.cpp -I../include -L../lib -lmyparser

#include <stdint.h>
#include <stddef.h>
#include <string.h>

// Include your parser header
#include "parser.h"

extern "C" int LLVMFuzzerTestOneInput(const uint8_t *data, size_t size) {
    // Reject inputs that are too large
    if (size > 1024 * 1024) {
        return 0;
    }
    
    // Create null-terminated copy if needed
    char *buf = (char *)malloc(size + 1);
    if (!buf) return 0;
    memcpy(buf, data, size);
    buf[size] = '\\0';
    
    // Call the parsing function
    ParseResult *result = parse_input(buf, size);
    
    // Clean up
    if (result) {
        free_result(result);
    }
    free(buf);
    
    return 0;
}

// Optional: Custom mutator for structure-aware fuzzing
extern "C" size_t LLVMFuzzerCustomMutator(
    uint8_t *data, size_t size,
    size_t max_size, unsigned int seed) {
    // Implement grammar-aware mutations here
    return LLVMFuzzerMutate(data, size, max_size);
}
''',
            build_instructions='''# Build instructions for C++ parser fuzzer

## Prerequisites
- clang++ with fuzzer support
- AddressSanitizer (usually included with clang)

## Build command
```bash
clang++ -g -O1 \\
    -fsanitize=fuzzer,address,undefined \\
    -fno-omit-frame-pointer \\
    -o fuzz_parser fuzz_parser.cpp \\
    -I../include -L../lib -lmyparser
```

## Run
```bash
mkdir -p corpus
./fuzz_parser corpus/ -max_len=65536 -timeout=5
```

## With corpus and dictionary
```bash
./fuzz_parser corpus/ seeds/ -dict=parser.dict
```
'''
        ),
        "network": HarnessTemplate(
            language="cpp",
            target_type="network",
            template='''// fuzz_network.cpp
// Fuzzing harness for network protocol handling

#include <stdint.h>
#include <stddef.h>

#include "protocol.h"

// Mock network context
struct MockConnection {
    const uint8_t *data;
    size_t size;
    size_t pos;
};

static size_t mock_recv(void *ctx, uint8_t *buf, size_t len) {
    MockConnection *conn = (MockConnection *)ctx;
    size_t remaining = conn->size - conn->pos;
    size_t to_read = (len < remaining) ? len : remaining;
    memcpy(buf, conn->data + conn->pos, to_read);
    conn->pos += to_read;
    return to_read;
}

extern "C" int LLVMFuzzerTestOneInput(const uint8_t *data, size_t size) {
    MockConnection conn = {data, size, 0};
    
    // Create protocol handler with mock callbacks
    ProtocolHandler handler;
    handler.recv_callback = mock_recv;
    handler.context = &conn;
    
    // Process the "received" data
    process_protocol(&handler);
    
    cleanup_handler(&handler);
    return 0;
}
''',
            build_instructions='''# Build instructions for network protocol fuzzer

## Build
```bash
clang++ -g -O1 \\
    -fsanitize=fuzzer,address \\
    -o fuzz_network fuzz_network.cpp \\
    -I../include -L../lib -lprotocol
```

## Run with realistic seeds
```bash
# Capture real protocol traffic for seeds
tcpdump -w capture.pcap
# Extract payloads to corpus/
./fuzz_network corpus/
```
'''
        ),
    },
    "python": {
        "api": HarnessTemplate(
            language="python",
            target_type="api",
            template='''#!/usr/bin/env python3
"""
fuzz_api.py
Atheris-based fuzzer for Python API

Requirements:
    pip install atheris

Run:
    python fuzz_api.py
"""

import atheris
import sys

# Import the module to fuzz
with atheris.instrument_imports():
    import mymodule


def TestOneInput(data):
    """Fuzz target function."""
    fdp = atheris.FuzzedDataProvider(data)
    
    try:
        # Generate fuzzed inputs
        input_string = fdp.ConsumeUnicodeNoSurrogates(100)
        input_int = fdp.ConsumeIntInRange(0, 1000)
        input_bool = fdp.ConsumeBool()
        
        # Call the target function
        result = mymodule.process(
            data=input_string,
            count=input_int,
            validate=input_bool
        )
        
    except (ValueError, TypeError, KeyError) as e:
        # Expected exceptions - not bugs
        pass
    except Exception as e:
        # Unexpected exception - potential bug
        raise


def main():
    atheris.Setup(sys.argv, TestOneInput)
    atheris.Fuzz()


if __name__ == "__main__":
    main()
''',
            build_instructions='''# Python API Fuzzer

## Setup
```bash
pip install atheris
```

## Run
```bash
python fuzz_api.py -max_len=1000 -timeout=10
```

## With corpus
```bash
mkdir -p corpus
python fuzz_api.py corpus/
```
'''
        ),
        "json": HarnessTemplate(
            language="python",
            target_type="json",
            template='''#!/usr/bin/env python3
"""
fuzz_json_handler.py
Fuzz a JSON processing function
"""

import atheris
import sys
import json

with atheris.instrument_imports():
    import mymodule


def TestOneInput(data):
    """Fuzz JSON handling."""
    fdp = atheris.FuzzedDataProvider(data)
    
    # Try to create valid-ish JSON
    json_string = fdp.ConsumeUnicodeNoSurrogates(
        fdp.ConsumeIntInRange(0, 10000)
    )
    
    try:
        # First try parsing as JSON
        parsed = json.loads(json_string)
        
        # Then pass to target
        mymodule.handle_json(parsed)
        
    except json.JSONDecodeError:
        # Invalid JSON is expected
        pass
    except (ValueError, TypeError, KeyError):
        # Application-level expected errors
        pass


def main():
    atheris.Setup(sys.argv, TestOneInput)
    atheris.Fuzz()


if __name__ == "__main__":
    main()
''',
            build_instructions='''# JSON Handler Fuzzer

## Run with JSON dictionary
```bash
python fuzz_json_handler.py -dict=json.dict
```

## json.dict contents:
```
"{"
"}"
"["
"]"
":"
","
"true"
"false"
"null"
"\\"string\\""
```
'''
        ),
    },
    "go": {
        "parser": HarnessTemplate(
            language="go",
            target_type="parser",
            template='''// fuzz_test.go
// Go native fuzzing (Go 1.18+)

package mypackage

import (
    "testing"
)

func FuzzParser(f *testing.F) {
    // Add seed corpus
    f.Add([]byte("valid input"))
    f.Add([]byte("another valid input"))
    f.Add([]byte(""))
    
    f.Fuzz(func(t *testing.T, data []byte) {
        // Call the function under test
        result, err := Parse(data)
        
        if err != nil {
            // Errors are expected for invalid input
            return
        }
        
        // Optionally validate result
        if result == nil {
            t.Error("Parse returned nil without error")
        }
    })
}

func FuzzParserWithString(f *testing.F) {
    // Fuzzing with typed inputs
    f.Add("test", 100, true)
    
    f.Fuzz(func(t *testing.T, input string, count int, flag bool) {
        _ = ProcessWithOptions(input, count, flag)
    })
}
''',
            build_instructions='''# Go Native Fuzzing

## Run
```bash
go test -fuzz=FuzzParser -fuzztime=60s
```

## With specific corpus
```bash
go test -fuzz=FuzzParser -run=FuzzParser/seed
```

## Check coverage
```bash
go test -fuzz=FuzzParser -coverprofile=coverage.out
go tool cover -html=coverage.out
```
'''
        ),
    },
    "rust": {
        "parser": HarnessTemplate(
            language="rust",
            target_type="parser",
            template='''// fuzz/fuzz_targets/fuzz_parser.rs
// cargo-fuzz target

#![no_main]

use libfuzzer_sys::fuzz_target;

// Import your crate
use mycrate::parser;

fuzz_target!(|data: &[u8]| {
    // Try to parse as UTF-8 string
    if let Ok(input) = std::str::from_utf8(data) {
        // Call parser - ignore errors
        let _ = parser::parse(input);
    }
});

// Alternative: Structured fuzzing with Arbitrary
/*
use arbitrary::Arbitrary;

#[derive(Arbitrary, Debug)]
struct FuzzInput {
    data: String,
    options: ParserOptions,
}

fuzz_target!(|input: FuzzInput| {
    let _ = parser::parse_with_options(&input.data, &input.options);
});
*/
''',
            build_instructions='''# Rust Fuzzing with cargo-fuzz

## Setup
```bash
cargo install cargo-fuzz
cargo fuzz init
```

## Run
```bash
cargo +nightly fuzz run fuzz_parser
```

## With timeout and memory limit
```bash
cargo +nightly fuzz run fuzz_parser -- \\
    -max_total_time=3600 \\
    -rss_limit_mb=2048
```

## List crashes
```bash
cargo +nightly fuzz list
```
'''
        ),
    },
}


def list_templates():
    """List all available templates."""
    print("Available fuzzing harness templates:\n")
    for lang, targets in TEMPLATES.items():
        print(f"  {lang}:")
        for target in targets:
            print(f"    - {target}")
    print()


def generate_template(language: str, target_type: str):
    """Generate a fuzzing harness template."""
    if language not in TEMPLATES:
        print(f"Error: Unknown language '{language}'")
        print(f"Available: {list(TEMPLATES.keys())}")
        return
    
    if target_type not in TEMPLATES[language]:
        print(f"Error: Unknown target type '{target_type}' for {language}")
        print(f"Available: {list(TEMPLATES[language].keys())}")
        return
    
    template = TEMPLATES[language][target_type]
    
    print("=" * 60)
    print(f"FUZZING HARNESS: {language.upper()} {target_type.upper()}")
    print("=" * 60)
    print("\n--- HARNESS CODE ---\n")
    print(template.template)
    print("\n--- BUILD INSTRUCTIONS ---\n")
    print(template.build_instructions)


def main():
    parser = argparse.ArgumentParser(
        description="Generate fuzzing harness templates"
    )
    parser.add_argument("--lang", "-l", help="Target language")
    parser.add_argument("--target", "-t", help="Target type")
    parser.add_argument("--list", action="store_true", 
                       help="List available templates")
    
    args = parser.parse_args()
    
    if args.list:
        list_templates()
    elif args.lang and args.target:
        generate_template(args.lang, args.target)
    else:
        parser.print_help()


if __name__ == "__main__":
    main()
