# Fast Forth System Diagrams
**Visual Architecture Reference**

This document contains mermaid diagrams visualizing the Fast Forth architecture. These diagrams can be rendered in GitHub, GitLab, or any mermaid-compatible viewer.

---

## 1. Overall System Architecture

```mermaid
graph TB
    subgraph "Frontend Pipeline"
        A[Source Code] --> B[Lexer]
        B --> C[Parser]
        C --> D[AST Builder]
        D --> E[Type Checker]
    end

    subgraph "IR Pipeline"
        E --> F[HIR Builder]
        F --> G[HIR Optimizer]
        G --> H[MIR Lowering]
        H --> I[MIR Optimizer]
        I --> J[LIR Lowering]
    end

    subgraph "Backend Selection"
        J --> K{Backend?}
        K -->|Fast Compile| L[Threaded Code]
        K -->|Balanced| M[LLVM -O1]
        K -->|Performance| N[LLVM -O3]
    end

    subgraph "Execution"
        L --> O[Interpreter]
        M --> P[Native Code]
        N --> P
        O --> Q[Runtime System]
        P --> Q
    end

    style A fill:#e1f5ff
    style Q fill:#ffe1e1
    style F fill:#fff4e1
    style H fill:#fff4e1
    style J fill:#fff4e1
```

---

## 2. Type Inference Flow

```mermaid
graph LR
    A[AST with Stack Comments] --> B[Constraint Generation]
    B --> C[Type Variables]
    B --> D[Equality Constraints]
    B --> E[Type Class Constraints]

    C --> F[Unification Engine]
    D --> F
    E --> F

    F --> G{Solvable?}
    G -->|Yes| H[Substitution]
    G -->|No| I[Type Error]

    H --> J[Concrete Types]
    J --> K[Generalization]
    K --> L[Type Scheme]

    style A fill:#e1f5ff
    style I fill:#ffe1e1
    style L fill:#e1ffe1
```

---

## 3. IR Transformation Pipeline

```mermaid
graph TD
    A[Forth Source] --> B[HIR]

    subgraph "HIR Level"
        B --> B1[Word Inlining]
        B1 --> B2[Constant Propagation]
        B2 --> B3[Dead Definition Elimination]
    end

    B3 --> C[MIR]

    subgraph "MIR Level"
        C --> C1[Stack Caching]
        C1 --> C2[Superinstruction Formation]
        C2 --> C3[Constant Folding]
        C3 --> C4[CSE]
        C4 --> C5[DCE]
        C5 --> C6[LICM]
        C6 --> C7[Specialization]
        C7 --> C8[Plugin Optimizations]
    end

    C8 --> D[LIR]

    subgraph "LIR Level"
        D --> D1[Register Allocation]
        D1 --> D2[Instruction Selection]
        D2 --> D3[Peephole Optimization]
    end

    D3 --> E[Native/LLVM IR]

    style A fill:#e1f5ff
    style E fill:#e1ffe1
```

---

## 4. Stack Caching Architecture

```mermaid
graph TB
    subgraph "Traditional Forth"
        A1[Stack in Memory] --> A2[Push/Pop Operations]
        A2 --> A3[Memory Bandwidth Limited]
    end

    subgraph "Fast Forth with Stack Caching"
        B1[Virtual Stack] --> B2{Stack Depth Analysis}
        B2 -->|Top 8| B3[Register Cache]
        B2 -->|Overflow| B4[Memory Spill]

        B3 --> B5[Register Operations]
        B4 --> B6[Memory Operations]

        B5 --> B7[70-90% Faster]
        B6 --> B7
    end

    style A3 fill:#ffe1e1
    style B7 fill:#e1ffe1
```

---

## 5. JIT Tiering Strategy

```mermaid
stateDiagram-v2
    [*] --> Threaded: First Call

    Threaded --> LLVM_O1: 1000+ calls
    LLVM_O1 --> LLVM_O3: 10000+ calls

    Threaded: Threaded Code\n60% C perf\n10ms compile
    LLVM_O1: LLVM -O1\n85% C perf\n50ms compile
    LLVM_O3: LLVM -O3 + PGO\n95-100% C perf\n200ms compile

    note right of Threaded
        Fast startup
        Interactive development
    end note

    note right of LLVM_O3
        Maximum performance
        Production workloads
    end note
```

---

## 6. Plugin Architecture

```mermaid
graph TB
    A[Plugin Manager] --> B[Plugin Discovery]
    B --> C[Load Plugins]

    C --> D[Plugin 1]
    C --> E[Plugin 2]
    C --> F[Plugin N]

    D --> G[Register Optimizations]
    E --> G
    F --> G

    G --> H[Optimization Registry]

    subgraph "Compilation Hooks"
        I[on_hir_created]
        J[on_mir_created]
        K[on_lir_created]
    end

    H --> I
    H --> J
    H --> K

    I --> L[Modified HIR]
    J --> M[Modified MIR]
    K --> N[Modified LIR]

    style A fill:#e1f5ff
    style L fill:#e1ffe1
    style M fill:#e1ffe1
    style N fill:#e1ffe1
```

---

## 7. Type System Components

```mermaid
classDiagram
    class Type {
        +Int32
        +Int64
        +Float64
        +Array(Type)
        +Function(StackEffect)
        +Var(TypeVar)
    }

    class StackEffect {
        +inputs: Vec~Type~
        +outputs: Vec~Type~
        +compose(StackEffect)
        +notation() String
    }

    class TypeScheme {
        +quantified: Vec~TypeVar~
        +constraints: Vec~Constraint~
        +ty: Type
        +instantiate()
        +generalize()
    }

    class Constraint {
        +Equal(Type, Type)
        +Numeric(TypeVar)
        +Ordered(TypeVar)
    }

    Type --> StackEffect
    TypeScheme --> Type
    TypeScheme --> Constraint
```

---

## 8. Compilation Pipeline Data Flow

```mermaid
sequenceDiagram
    participant S as Source
    participant F as Frontend
    participant T as Type System
    participant I as IR Builder
    participant O as Optimizer
    participant B as Backend

    S->>F: Forth Code
    F->>F: Lex & Parse
    F->>T: AST
    T->>T: Infer Types
    T->>I: Typed AST
    I->>I: Build HIR
    I->>I: Lower to MIR
    I->>O: MIR
    O->>O: Optimize
    O->>B: Optimized MIR
    B->>B: Generate LIR
    B->>B: LLVM IR / Threaded Code
    B->>B: Native Code
    B-->>S: Executable
```

---

## 9. Register Allocation

```mermaid
graph TB
    A[MIR with Virtual Registers] --> B[Liveness Analysis]
    B --> C[Interference Graph]

    C --> D[Graph Coloring]
    D --> E{Colorable?}

    E -->|Yes| F[Physical Register Assignment]
    E -->|No| G[Spill to Memory]

    G --> H[Insert Load/Store]
    H --> D

    F --> I[LIR with Physical Registers]

    style A fill:#e1f5ff
    style I fill:#e1ffe1
```

---

## 10. Error Reporting Flow

```mermaid
graph LR
    A[Error Detected] --> B{Error Type?}

    B -->|Lexer| C[Lexical Error]
    B -->|Parser| D[Syntax Error]
    B -->|Type| E[Type Error]
    B -->|Semantic| F[Semantic Error]

    C --> G[Error Context Extraction]
    D --> G
    E --> G
    F --> G

    G --> H[Source Location]
    H --> I[Format Error Message]

    I --> J[Display with Context]

    J --> K[Source Line]
    J --> L[Error Message]
    J --> M[Suggestion]

    style A fill:#ffe1e1
    style J fill:#fff4e1
```

---

## 11. Memory Layout

```mermaid
graph TB
    subgraph "Runtime Memory"
        A[Data Stack]
        B[Return Stack]
        C[Dictionary]
        D[Heap]
        E[Code Cache]
    end

    subgraph "Data Stack (64KB)"
        A1[TOS - Register]
        A2[TOS-1 - Register]
        A3[TOS-7 - Register]
        A4[Deeper - Memory]
    end

    subgraph "Dictionary"
        C1[Word Headers]
        C2[Type Schemes]
        C3[Implementation Pointers]
    end

    A --> A1
    A1 --> A2
    A2 --> A3
    A3 --> A4

    C --> C1
    C1 --> C2
    C2 --> C3
```

---

## 12. Optimization Pass Order

```mermaid
graph TD
    A[Unoptimized MIR] --> B[Stack Caching]

    B --> C{Changed?}
    C -->|Yes| D[Constant Folding]
    C -->|No| E[Next Pass]

    D --> F{Changed?}
    F -->|Yes| G[CSE]
    F -->|No| E

    G --> H{Changed?}
    H -->|Yes| I[DCE]
    H -->|No| E

    I --> J{Changed?}
    J -->|Yes| K[Superinstructions]
    J -->|No| E

    K --> L{Changed?}
    L -->|Yes| M[Specialization]
    L -->|No| E

    M --> N{Converged?}
    N -->|No| B
    N -->|Yes| O[Optimized MIR]

    style A fill:#e1f5ff
    style O fill:#e1ffe1
```

---

## 13. Development Streams Timeline

```mermaid
gantt
    title Fast Forth Development Timeline
    dateFormat YYYY-MM-DD
    section Phase 1
    STREAM 1 Architecture    :done, s1, 2025-11-14, 1w
    STREAM 2 Frontend        :active, s2, 2025-11-21, 2w
    STREAM 3 Type System     :s3, 2025-11-21, 2w

    section Phase 2
    STREAM 4 IR Builder      :s4, 2025-12-05, 2w
    STREAM 5 Optimizer       :s5, 2025-12-12, 2w
    STREAM 6 LLVM Backend    :s6, 2025-12-12, 2w

    section Phase 3
    STREAM 7 Runtime/JIT     :s7, 2025-12-26, 2w
    STREAM 8 Testing         :s8, 2025-12-26, 2w

    section Phase 4
    Integration             :2026-01-09, 1w
    Optimization            :2026-01-16, 2w
```

---

## 14. Component Dependencies

```mermaid
graph TD
    A[Frontend] --> B[Type System]
    B --> C[IR Builder]
    C --> D[Optimizer]
    D --> E[Backend]

    F[Plugin System] -.-> D
    G[Runtime] -.-> E

    H[Testing] -.-> A
    H -.-> B
    H -.-> C
    H -.-> D
    H -.-> E

    style A fill:#e1f5ff
    style E fill:#e1ffe1
    style F fill:#fff4e1
```

---

These diagrams provide visual reference for the Fast Forth architecture. For implementation details, see:
- [ARCHITECTURE.md](ARCHITECTURE.md)
- [ARCHITECTURE_QUICKSTART.md](ARCHITECTURE_QUICKSTART.md)
- [IR_SPECIFICATION.md](../specs/IR_SPECIFICATION.md)
- [TYPE_SYSTEM_SPECIFICATION.md](../specs/TYPE_SYSTEM_SPECIFICATION.md)

---

**Generated by**: Architect Agent (STREAM 1)
**Date**: 2025-11-14
**Version**: 1.0
