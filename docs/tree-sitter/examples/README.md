# Tree-sitter Examples

This directory contains examples demonstrating the usage of tree-sitter for parsing and analyzing source code across multiple programming languages.

## Definition Enumeration Example

The main example demonstrates how to use tree-sitter to enumerate definitions (functions, classes, methods, etc.) in source code files. It supports multiple programming languages including Python, JavaScript, TypeScript, and Java.

### Implementation Details

The implementation consists of several components:

1. **Core Parser (`enumerate_definitions.rs`)**
   - Uses tree-sitter's parsing capabilities to analyze source code
   - Extracts definitions based on language-specific patterns
   - Handles multiple programming languages
   - Avoids duplicate definitions through careful tracking

2. **Language Queries (`queries/`)**
   - `python.scm`: Captures Python functions and classes
   - `javascript.scm`: Captures JavaScript functions, classes, and variables
   - `typescript.scm`: Extends JavaScript with TypeScript-specific patterns
   - `java.scm`: Captures Java classes, methods, interfaces, and enums

3. **Test Files (`testfiles/`)**
   Example files demonstrating supported constructs in each language:
   - `test.py`: Python functions and classes
   - `test.js`: JavaScript functions, classes, and variables
   - `test.ts`: TypeScript interfaces, types, and classes
   - `test.java`: Java classes, methods, interfaces, and enums

### Usage Example

```rust
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Files to analyze
    let files = vec![
        PathBuf::from("path/to/file.py"),
        PathBuf::from("path/to/file.js"),
        PathBuf::from("path/to/file.ts"),
        PathBuf::from("path/to/file.java"),
    ];

    // Enumerate definitions
    let results = enumerate_definitions(files)?;

    // Process results
    for (file_path, definitions) in results {
        println!("Definitions in {}:", file_path.display());
        for def in definitions {
            println!("- {} ({}..{})", def.name, def.start_byte, def.end_byte);
        }
    }

    Ok(())
}
```

### Supported Definition Types

#### Python
- Functions (`def`)
- Classes (`class`)
- Methods (class functions)

#### JavaScript
- Functions (`function`)
- Classes (`class`)
- Methods (class functions)
- Variables (`const`, `let`)

#### TypeScript
- Interfaces (`interface`)
- Types (`type`)
- Functions (`function`)
- Classes (`class`)
- Methods (class functions)
- Variables (`const`, `let`)

#### Java
- Classes (`class`)
- Methods (excluding constructors)
- Interfaces (`interface`)
- Enums (`enum`)

### Implementation Notes

1. **Language Detection**
   - Uses file extensions to determine the appropriate parser
   - Supports `.py`, `.js`, `.ts`, and `.java` files

2. **Definition Extraction**
   - Uses tree-sitter queries to identify definitions
   - Handles nested definitions (e.g., methods in classes)
   - Avoids duplicate definitions through careful tracking

3. **Error Handling**
   - Gracefully handles unsupported file types
   - Reports parsing errors without crashing

### Dependencies

```toml
[dependencies]
tree-sitter = "0.20.10"
tree-sitter-python = "0.20.4"
tree-sitter-javascript = "0.20.1"
tree-sitter-typescript = "0.20.3"
tree-sitter-java = "0.20.0"
```

### Testing

The implementation includes comprehensive tests for each supported language, verifying:
- Basic definition extraction
- Nested definition handling
- Language-specific features
- Error cases

Run tests with:
```bash
cargo test
```
