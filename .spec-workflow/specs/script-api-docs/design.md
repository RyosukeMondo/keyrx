# Design Document

## Overview

This design creates a documentation extraction system for Rhai functions registered in KeyRx. The core innovation is a `#[rhai_doc]` attribute macro that captures function signatures and doc comments at compile time, building a documentation registry that can be exported to multiple formats.

## Steering Document Alignment

### Technical Standards (tech.md)
- **Documentation**: Auto-generated from source
- **Testing**: Examples are tests
- **Type Safety**: Type info from Rust types

### Project Structure (structure.md)
- Rhai registration in `core/src/scripting/`
- Doc generation in `core/src/scripting/docs/`
- Output to `docs/scripting/`

## Code Reuse Analysis

### Existing Components to Leverage
- **Rhai Engine**: Already registers functions
- **Doc comments**: Already exist on some functions
- **Build script**: Can generate docs

### Integration Points
- **Rhai registration**: Extract doc metadata
- **Build process**: Generate docs
- **Flutter UI**: Display docs in-app

## Architecture

```mermaid
graph TD
    subgraph "Source Code"
        SRC[Rhai Functions] --> |#[rhai_doc]| META[Doc Metadata]
        SRC --> |register| ENG[Rhai Engine]
    end

    subgraph "Doc Registry"
        META --> REG[DocRegistry]
        REG --> FN[FunctionDoc]
        REG --> TY[TypeDoc]
        REG --> EX[ExampleDoc]
    end

    subgraph "Output"
        REG --> |generate| HTML[HTML Docs]
        REG --> |generate| MD[Markdown]
        REG --> |generate| JSON[JSON Schema]
    end
```

### Modular Design Principles
- **Single Source**: Code is documentation source
- **Compile-Time Extraction**: Metadata captured at build
- **Multiple Outputs**: Same data, different formats
- **Testable Examples**: Examples run as tests

## Components and Interfaces

### Component 1: DocRegistry

- **Purpose:** Central registry of API documentation
- **Interfaces:**
  ```rust
  pub struct DocRegistry {
      modules: HashMap<String, ModuleDoc>,
      types: HashMap<String, TypeDoc>,
      examples: Vec<Example>,
  }

  impl DocRegistry {
      pub fn global() -> &'static Self;
      pub fn register_function(&mut self, doc: FunctionDoc);
      pub fn register_type(&mut self, doc: TypeDoc);
      pub fn add_example(&mut self, example: Example);
      pub fn get_module(&self, name: &str) -> Option<&ModuleDoc>;
      pub fn all_functions(&self) -> impl Iterator<Item = &FunctionDoc>;
      pub fn search(&self, query: &str) -> Vec<SearchResult>;
  }
  ```
- **Dependencies:** None
- **Reuses:** Registry pattern

### Component 2: FunctionDoc

- **Purpose:** Documentation for a single Rhai function
- **Interfaces:**
  ```rust
  #[derive(Debug, Clone, Serialize)]
  pub struct FunctionDoc {
      pub name: String,
      pub module: String,
      pub signature: FunctionSignature,
      pub description: String,
      pub parameters: Vec<ParamDoc>,
      pub returns: Option<ReturnDoc>,
      pub examples: Vec<String>,
      pub since: Option<String>,
      pub deprecated: Option<String>,
  }

  #[derive(Debug, Clone, Serialize)]
  pub struct FunctionSignature {
      pub params: Vec<(String, String)>,  // (name, type)
      pub return_type: Option<String>,
  }

  #[derive(Debug, Clone, Serialize)]
  pub struct ParamDoc {
      pub name: String,
      pub type_name: String,
      pub description: String,
      pub optional: bool,
      pub default: Option<String>,
  }

  #[derive(Debug, Clone, Serialize)]
  pub struct ReturnDoc {
      pub type_name: String,
      pub description: String,
  }
  ```
- **Dependencies:** serde
- **Reuses:** Documentation patterns

### Component 3: TypeDoc

- **Purpose:** Documentation for Rhai types
- **Interfaces:**
  ```rust
  #[derive(Debug, Clone, Serialize)]
  pub struct TypeDoc {
      pub name: String,
      pub description: String,
      pub methods: Vec<FunctionDoc>,
      pub properties: Vec<PropertyDoc>,
      pub constructors: Vec<FunctionDoc>,
  }

  #[derive(Debug, Clone, Serialize)]
  pub struct PropertyDoc {
      pub name: String,
      pub type_name: String,
      pub description: String,
      pub readonly: bool,
  }
  ```
- **Dependencies:** FunctionDoc
- **Reuses:** Type documentation patterns

### Component 4: rhai_doc Attribute Macro

- **Purpose:** Extract documentation at compile time
- **Interfaces:**
  ```rust
  /// Document a Rhai function.
  ///
  /// # Example
  /// ```rust
  /// #[rhai_doc(module = "keys", since = "0.1.0")]
  /// /// Emits a key press event.
  /// ///
  /// /// # Parameters
  /// /// - `key`: The key code to press
  /// ///
  /// /// # Example
  /// /// ```rhai
  /// /// emit_key(Key::A);
  /// /// ```
  /// fn emit_key(key: KeyCode) -> Result<(), RhaiError> {
  ///     // implementation
  /// }
  /// ```
  #[proc_macro_attribute]
  pub fn rhai_doc(attr: TokenStream, item: TokenStream) -> TokenStream;
  ```
- **Dependencies:** proc-macro2, syn, quote
- **Reuses:** Attribute macro patterns

### Component 5: DocGenerator

- **Purpose:** Generate documentation in various formats
- **Interfaces:**
  ```rust
  pub struct DocGenerator {
      registry: &'static DocRegistry,
  }

  impl DocGenerator {
      pub fn new() -> Self;

      // Output formats
      pub fn generate_html(&self, output_dir: &Path) -> io::Result<()>;
      pub fn generate_markdown(&self, output_dir: &Path) -> io::Result<()>;
      pub fn generate_json_schema(&self, output: &Path) -> io::Result<()>;

      // Index for search
      pub fn generate_search_index(&self) -> SearchIndex;
  }

  pub struct SearchIndex {
      entries: Vec<SearchEntry>,
  }
  ```
- **Dependencies:** DocRegistry, templating
- **Reuses:** Documentation generation patterns

### Component 6: Example Runner

- **Purpose:** Test documentation examples
- **Interfaces:**
  ```rust
  pub struct ExampleRunner {
      engine: Engine,
  }

  impl ExampleRunner {
      pub fn new() -> Self;
      pub fn run_example(&self, code: &str) -> Result<(), ExampleError>;
      pub fn run_all_examples(&self) -> Vec<ExampleResult>;
  }

  pub struct ExampleResult {
      pub function: String,
      pub code: String,
      pub passed: bool,
      pub error: Option<String>,
  }
  ```
- **Dependencies:** Rhai engine
- **Reuses:** Test runner patterns

## Data Models

### SearchResult
```rust
#[derive(Debug, Clone, Serialize)]
pub struct SearchResult {
    pub kind: SearchResultKind,
    pub name: String,
    pub module: String,
    pub description: String,
    pub score: f64,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub enum SearchResultKind {
    Function,
    Type,
    Property,
    Example,
}
```

### ModuleDoc
```rust
#[derive(Debug, Clone, Serialize)]
pub struct ModuleDoc {
    pub name: String,
    pub description: String,
    pub functions: Vec<FunctionDoc>,
    pub types: Vec<TypeDoc>,
}
```

## Error Handling

### Error Scenarios

1. **Missing documentation**
   - **Handling:** Build warning, generate placeholder
   - **User Impact:** Incomplete but usable docs

2. **Example failure**
   - **Handling:** Build warning, mark example as broken
   - **User Impact:** Clear indication of issue

3. **Type extraction failure**
   - **Handling:** Use "any" type, log warning
   - **User Impact:** Less precise type info

## Testing Strategy

### Unit Testing
- Test doc extraction
- Test generation outputs
- Test search functionality

### Example Testing
- Run all examples as tests
- Verify example output
- Test error examples

### Integration Testing
- Full doc generation pipeline
- HTML rendering verification
- Search index accuracy
