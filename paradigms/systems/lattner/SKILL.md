---
name: lattner-compiler-infrastructure
description: Write compiler and toolchain code in the style of Chris Lattner, creator of LLVM, Clang, Swift, and MLIR. Emphasizes modular compiler design, reusable infrastructure, progressive lowering, and pragmatic language evolution. Use when building compilers, language tools, or performance-critical infrastructure.
---

# Chris Lattner Style Guide

## Overview

Chris Lattner created LLVM (the compiler infrastructure that powers most modern compilers), Clang (the C/C++/Objective-C frontend), Swift (Apple's systems language), and MLIR (multi-level intermediate representation). His work fundamentally changed how compilers are built and how languages evolve.

## Core Philosophy

> "The key insight of LLVM is that compiler infrastructure should be reusable."

> "Good IR design is about finding the right level of abstraction."

> "Languages should evolve based on real-world usage, not theoretical purity."

Lattner believes in building robust, reusable infrastructure that enables an ecosystem of tools—not one-off solutions.

## Design Principles

1. **Modular Infrastructure**: Build reusable components, not monolithic systems.

2. **Progressive Lowering**: Transform through well-defined IR levels.

3. **Library-First Design**: Compilers are libraries, not just executables.

4. **Pragmatic Evolution**: Languages improve through real usage feedback.

## When Writing Compiler Code

### Always

- Design IRs with clear semantics and invariants
- Make passes composable and reusable
- Provide excellent diagnostics and error messages
- Build infrastructure others can extend
- Think about the entire compilation pipeline
- Document design decisions and tradeoffs

### Never

- Build closed, monolithic compiler architectures
- Sacrifice usability for implementation convenience
- Ignore error recovery and diagnostics
- Let optimization passes have hidden dependencies
- Couple frontend concerns with backend concerns
- Design IRs without considering transformations

### Prefer

- SSA form for optimization IRs
- Explicit type systems over implicit
- Library APIs over command-line tools
- Incremental compilation where possible
- Clear phase ordering over ad-hoc passes
- Compositional design over special cases

## Code Patterns

### LLVM IR Philosophy

```llvm
; LLVM IR: explicit, typed, SSA form
; Every value has exactly one definition
; Control flow is explicit

define i32 @factorial(i32 %n) {
entry:
  %cmp = icmp sle i32 %n, 1
  br i1 %cmp, label %base, label %recurse

base:
  ret i32 1

recurse:
  %n_minus_1 = sub i32 %n, 1
  %fact_sub = call i32 @factorial(i32 %n_minus_1)
  %result = mul i32 %n, %fact_sub
  ret i32 %result
}

; Key properties:
; - SSA: each %variable defined exactly once
; - Typed: every operation has explicit types
; - Explicit control flow: br, ret, etc.
; - No hidden state or side effects in IR
```

### Pass Infrastructure Design

```cpp
// LLVM-style pass infrastructure
// Passes are modular, composable, declarative

class MyOptimizationPass : public PassInfoMixin<MyOptimizationPass> {
public:
    PreservedAnalyses run(Function &F, FunctionAnalysisManager &AM) {
        // Get required analyses
        auto &DT = AM.getResult<DominatorTreeAnalysis>(F);
        auto &LI = AM.getResult<LoopAnalysis>(F);
        
        bool Changed = false;
        
        for (auto &BB : F) {
            Changed |= optimizeBlock(BB, DT, LI);
        }
        
        if (!Changed)
            return PreservedAnalyses::all();
        
        // Declare what we preserved
        PreservedAnalyses PA;
        PA.preserve<DominatorTreeAnalysis>();
        return PA;
    }
    
private:
    bool optimizeBlock(BasicBlock &BB, DominatorTree &DT, LoopInfo &LI);
};

// Register the pass
extern "C" LLVM_ATTRIBUTE_WEAK ::llvm::PassPluginLibraryInfo
llvmGetPassPluginInfo() {
    return {
        LLVM_PLUGIN_API_VERSION, "MyPass", "v0.1",
        [](PassBuilder &PB) {
            PB.registerPipelineParsingCallback(
                [](StringRef Name, FunctionPassManager &FPM,
                   ArrayRef<PassBuilder::PipelineElement>) {
                    if (Name == "my-opt") {
                        FPM.addPass(MyOptimizationPass());
                        return true;
                    }
                    return false;
                });
        }
    };
}
```

### Diagnostic Excellence

```cpp
// Swift/Clang-style diagnostics
// Errors should be helpful, not cryptic

class DiagnosticEngine {
public:
    // Structured diagnostics with fix-its
    void diagnose(SourceLoc Loc, Diagnostic Diag) {
        emitDiagnostic(Loc, Diag.getKind(), Diag.getMessage());
        
        // Show the source location
        emitSourceSnippet(Loc);
        
        // Provide fix-its when possible
        for (auto &FixIt : Diag.getFixIts()) {
            emitFixIt(FixIt);
        }
        
        // Add educational notes
        for (auto &Note : Diag.getNotes()) {
            emitNote(Note);
        }
    }
};

// Example diagnostic output:
// error: cannot convert value of type 'String' to expected type 'Int'
//     let x: Int = "hello"
//                  ^~~~~~~
// fix-it: did you mean to use Int(_:)?
//     let x: Int = Int("hello") ?? 0
```

### Progressive Lowering (MLIR Style)

```cpp
// MLIR: Multi-Level IR for progressive lowering
// High-level ops → Mid-level ops → Low-level ops → LLVM IR

// High-level: domain-specific operations
%result = linalg.matmul ins(%A, %B : tensor<4x8xf32>, tensor<8x16xf32>)
                        outs(%C : tensor<4x16xf32>) -> tensor<4x16xf32>

// After tiling transformation:
%tiled = scf.for %i = %c0 to %c4 step %c2 {
    %slice_a = tensor.extract_slice %A[%i, 0][2, 8][1, 1]
    %slice_c = tensor.extract_slice %C[%i, 0][2, 16][1, 1]
    %computed = linalg.matmul ins(%slice_a, %B) outs(%slice_c)
    scf.yield %computed
}

// After vectorization:
%vec = vector.contract {indexing_maps = [...], kind = #vector.kind<add>}
    %vec_a, %vec_b, %vec_c : vector<2x8xf32>, vector<8x16xf32> into vector<2x16xf32>

// Finally: LLVM IR
// Each level has clear semantics and transformations
```

### Type System Design

```swift
// Swift-style type system: expressive, safe, pragmatic

// Protocol-oriented design
protocol Numeric {
    static func +(lhs: Self, rhs: Self) -> Self
    static func *(lhs: Self, rhs: Self) -> Self
}

// Associated types for flexibility
protocol Collection {
    associatedtype Element
    associatedtype Index: Comparable
    
    var startIndex: Index { get }
    var endIndex: Index { get }
    subscript(position: Index) -> Element { get }
}

// Generics with constraints
func sum<T: Numeric>(_ values: [T]) -> T {
    values.reduce(.zero, +)
}

// Optionals as explicit nullability
func find<T: Equatable>(_ value: T, in array: [T]) -> Int? {
    for (index, element) in array.enumerated() {
        if element == value {
            return index
        }
    }
    return nil  // Explicit absence
}

// Result types for error handling
enum Result<Success, Failure: Error> {
    case success(Success)
    case failure(Failure)
}
```

### Compiler as Library

```cpp
// Clang as a library, not just a tool
// Enable building custom tools on compiler infrastructure

#include "clang/Frontend/CompilerInstance.h"
#include "clang/Frontend/FrontendActions.h"
#include "clang/Tooling/Tooling.h"

// Custom AST visitor
class FunctionFinder : public RecursiveASTVisitor<FunctionFinder> {
public:
    bool VisitFunctionDecl(FunctionDecl *FD) {
        if (FD->hasBody()) {
            llvm::outs() << "Found function: " << FD->getName() << "\n";
            analyzeComplexity(FD);
        }
        return true;
    }
    
private:
    void analyzeComplexity(FunctionDecl *FD);
};

// Build custom tools using Clang's libraries
int main(int argc, const char **argv) {
    auto ExpectedParser = CommonOptionsParser::create(argc, argv, MyCategory);
    if (!ExpectedParser) {
        llvm::errs() << ExpectedParser.takeError();
        return 1;
    }
    
    ClangTool Tool(ExpectedParser->getCompilations(),
                   ExpectedParser->getSourcePathList());
    
    return Tool.run(newFrontendActionFactory<MyFrontendAction>().get());
}
```

### Memory Ownership in Swift

```swift
// Swift's ownership model: safe by default, explicit when needed

// Default: automatic reference counting
class Node {
    var value: Int
    var children: [Node]
    
    init(value: Int) {
        self.value = value
        self.children = []
    }
}

// Explicit ownership for performance-critical code
func processBuffer(_ buffer: borrowing [UInt8]) -> Int {
    // borrowing: read-only access, no copy
    buffer.reduce(0, +)
}

func consumeBuffer(_ buffer: consuming [UInt8]) -> [UInt8] {
    // consuming: takes ownership, no copy
    var result = buffer
    result.append(0)
    return result
}

// Copy-on-write for value semantics with efficiency
struct LargeData {
    private var storage: Storage
    
    mutating func modify() {
        // Copy only if shared
        if !isKnownUniquelyReferenced(&storage) {
            storage = storage.copy()
        }
        storage.data[0] = 42
    }
}
```

## IR Design Principles

```
Intermediate Representation Design
══════════════════════════════════════════════════════════════

Level           Abstraction         Purpose
────────────────────────────────────────────────────────────
Source          Syntax trees        Parsing, early semantic
AST/HIR         Typed trees         Type checking, inference
MIR/SIL         Typed CFG           Optimization, ownership
LLVM IR         Typed SSA           Machine-independent opt
Machine IR      Target ops          Instruction selection
Assembly        Text                Final output

Key principles:
• Each level has ONE clear purpose
• Lowering is progressive and well-defined
• Analyses valid at one level may not be at another
• Transformations declare their requirements
```

## Mental Model

Lattner approaches compiler design by asking:

1. **What's the right abstraction level?** Different problems need different IRs
2. **Is this reusable?** Build infrastructure, not one-off tools
3. **What's the user experience?** Diagnostics, error recovery, tooling
4. **How will this evolve?** Design for change and extension
5. **Can others build on this?** Library-first, composable design

## Signature Lattner Moves

- **LLVM's pass manager**: Modular, composable optimization passes
- **Clang's diagnostics**: The gold standard for helpful error messages
- **Swift's optionals**: Explicit nullability without verbosity
- **MLIR's dialect system**: Multi-level IR with extensible operations
- **Library-first design**: Compilers as reusable infrastructure
- **Progressive lowering**: Clear transformation stages
- **SwiftUI's result builders**: Compiler magic that feels natural
