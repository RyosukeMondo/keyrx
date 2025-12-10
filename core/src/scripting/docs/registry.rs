//! Central registry for storing and retrieving Rhai API documentation.
//!
//! This module provides a thread-safe static registry for documentation that
//! can be populated at startup and queried efficiently at runtime.

use super::types::{FunctionDoc, ModuleDoc, TypeDoc};
use std::collections::HashMap;
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

/// Static global documentation registry.
static DOC_REGISTRY: RwLock<Option<DocRegistry>> = RwLock::new(None);

/// Central registry for all documentation.
///
/// The registry stores documentation for functions, types, and modules
/// in a structured way that allows efficient lookup and retrieval.
#[derive(Debug, Clone, Default)]
pub struct DocRegistry {
    /// All registered function documentation, indexed by fully qualified name
    /// (module::function_name)
    functions: HashMap<String, FunctionDoc>,

    /// All registered type documentation, indexed by type name
    types: HashMap<String, TypeDoc>,

    /// Module documentation, indexed by module name
    modules: HashMap<String, ModuleDoc>,
}

impl DocRegistry {
    /// Creates a new empty documentation registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a function in the documentation registry.
    ///
    /// # Arguments
    /// * `function` - The function documentation to register
    ///
    /// # Example
    /// ```no_run
    /// # use keyrx_core::scripting::docs::{FunctionDoc, FunctionSignature};
    /// # use keyrx_core::scripting::docs::registry::DocRegistry;
    /// let mut registry = DocRegistry::new();
    /// let func_doc = FunctionDoc {
    ///     name: "emit_key".to_string(),
    ///     module: "keys".to_string(),
    ///     signature: FunctionSignature {
    ///         params: vec![("key".to_string(), "KeyCode".to_string())],
    ///         return_type: Some("()".to_string()),
    ///     },
    ///     description: "Emits a key press".to_string(),
    ///     parameters: vec![],
    ///     returns: None,
    ///     examples: vec![],
    ///     since: None,
    ///     deprecated: None,
    ///     notes: None,
    /// };
    /// registry.register_function(func_doc);
    /// ```
    pub fn register_function(&mut self, function: FunctionDoc) {
        let key = format!("{}::{}", function.module, function.name);
        self.functions.insert(key, function);
    }

    /// Registers a type in the documentation registry.
    ///
    /// # Arguments
    /// * `type_doc` - The type documentation to register
    pub fn register_type(&mut self, type_doc: TypeDoc) {
        self.types.insert(type_doc.name.clone(), type_doc);
    }

    /// Registers a module in the documentation registry.
    ///
    /// # Arguments
    /// * `module` - The module documentation to register
    pub fn register_module(&mut self, module: ModuleDoc) {
        self.modules.insert(module.name.clone(), module);
    }

    /// Retrieves documentation for a specific function.
    ///
    /// # Arguments
    /// * `module` - The module name
    /// * `name` - The function name
    ///
    /// # Returns
    /// The function documentation if found, None otherwise
    pub fn get_function(&self, module: &str, name: &str) -> Option<&FunctionDoc> {
        let key = format!("{}::{}", module, name);
        self.functions.get(&key)
    }

    /// Retrieves documentation for a specific type.
    ///
    /// # Arguments
    /// * `name` - The type name
    ///
    /// # Returns
    /// The type documentation if found, None otherwise
    pub fn get_type(&self, name: &str) -> Option<&TypeDoc> {
        self.types.get(name)
    }

    /// Retrieves documentation for a specific module.
    ///
    /// # Arguments
    /// * `name` - The module name
    ///
    /// # Returns
    /// The module documentation if found, None otherwise
    pub fn get_module(&self, name: &str) -> Option<&ModuleDoc> {
        self.modules.get(name)
    }

    /// Returns all registered function documentation.
    pub fn all_functions(&self) -> impl Iterator<Item = &FunctionDoc> {
        self.functions.values()
    }

    /// Returns all registered type documentation.
    pub fn all_types(&self) -> impl Iterator<Item = &TypeDoc> {
        self.types.values()
    }

    /// Returns all registered module documentation.
    pub fn all_modules(&self) -> impl Iterator<Item = &ModuleDoc> {
        self.modules.values()
    }

    /// Returns all functions in a specific module.
    ///
    /// # Arguments
    /// * `module` - The module name
    ///
    /// # Returns
    /// An iterator over function documentation for the specified module
    pub fn functions_in_module<'a>(
        &'a self,
        module: &'a str,
    ) -> impl Iterator<Item = &'a FunctionDoc> + 'a {
        self.functions.values().filter(move |f| f.module == module)
    }

    /// Returns all types in a specific module.
    ///
    /// # Arguments
    /// * `module` - The module name
    ///
    /// # Returns
    /// An iterator over type documentation for the specified module
    pub fn types_in_module<'a>(
        &'a self,
        module: &'a str,
    ) -> impl Iterator<Item = &'a TypeDoc> + 'a {
        self.types.values().filter(move |t| t.module == module)
    }

    /// Returns the total number of registered functions.
    pub fn function_count(&self) -> usize {
        self.functions.len()
    }

    /// Returns the total number of registered types.
    pub fn type_count(&self) -> usize {
        self.types.len()
    }

    /// Returns the total number of registered modules.
    pub fn module_count(&self) -> usize {
        self.modules.len()
    }

    /// Clears all documentation from the registry.
    pub fn clear(&mut self) {
        self.functions.clear();
        self.types.clear();
        self.modules.clear();
    }
}

/// Initialize the global documentation registry.
///
/// This should be called once at application startup before any
/// documentation queries are made.
pub fn initialize() {
    let mut registry = write_registry();
    *registry = Some(DocRegistry::new());
}

/// Register a function in the global documentation registry.
///
/// # Returns
/// `true` if registration succeeded, `false` if registry is not initialized or lock failed.
pub fn register_function(function: FunctionDoc) -> bool {
    let mut registry = write_registry();
    if let Some(ref mut reg) = *registry {
        reg.register_function(function);
        true
    } else {
        false
    }
}

/// Register a type in the global documentation registry.
///
/// # Returns
/// `true` if registration succeeded, `false` if registry is not initialized or lock failed.
pub fn register_type(type_doc: TypeDoc) -> bool {
    let mut registry = write_registry();
    if let Some(ref mut reg) = *registry {
        reg.register_type(type_doc);
        true
    } else {
        false
    }
}

/// Register a module in the global documentation registry.
///
/// # Returns
/// `true` if registration succeeded, `false` if registry is not initialized or lock failed.
pub fn register_module(module: ModuleDoc) -> bool {
    let mut registry = write_registry();
    if let Some(ref mut reg) = *registry {
        reg.register_module(module);
        true
    } else {
        false
    }
}

/// Get a function from the global documentation registry.
///
/// Returns None if the registry is not initialized, lock failed, or the function is not found.
pub fn get_function(module: &str, name: &str) -> Option<FunctionDoc> {
    let registry = read_registry();
    registry.as_ref()?.get_function(module, name).cloned()
}

/// Get a type from the global documentation registry.
///
/// Returns None if the registry is not initialized, lock failed, or the type is not found.
pub fn get_type(name: &str) -> Option<TypeDoc> {
    let registry = read_registry();
    registry.as_ref()?.get_type(name).cloned()
}

/// Get a module from the global documentation registry.
///
/// Returns None if the registry is not initialized, lock failed, or the module is not found.
pub fn get_module(name: &str) -> Option<ModuleDoc> {
    let registry = read_registry();
    registry.as_ref()?.get_module(name).cloned()
}

/// Get all functions from the global documentation registry.
///
/// Returns an empty vector if the registry is not initialized or lock failed.
pub fn all_functions() -> Vec<FunctionDoc> {
    let registry = read_registry();
    if let Some(reg) = registry.as_ref() {
        return reg.all_functions().cloned().collect();
    }
    vec![]
}

/// Get all types from the global documentation registry.
///
/// Returns an empty vector if the registry is not initialized or lock failed.
pub fn all_types() -> Vec<TypeDoc> {
    let registry = read_registry();
    if let Some(reg) = registry.as_ref() {
        return reg.all_types().cloned().collect();
    }
    vec![]
}

/// Get all modules from the global documentation registry.
///
/// Returns an empty vector if the registry is not initialized or lock failed.
pub fn all_modules() -> Vec<ModuleDoc> {
    let registry = read_registry();
    if let Some(reg) = registry.as_ref() {
        return reg.all_modules().cloned().collect();
    }
    vec![]
}

/// Get all functions in a specific module from the global registry.
pub fn functions_in_module(module: &str) -> Vec<FunctionDoc> {
    let registry = read_registry();
    if let Some(reg) = registry.as_ref() {
        return reg.functions_in_module(module).cloned().collect();
    }
    vec![]
}

/// Get all types in a specific module from the global registry.
pub fn types_in_module(module: &str) -> Vec<TypeDoc> {
    let registry = read_registry();
    if let Some(reg) = registry.as_ref() {
        return reg.types_in_module(module).cloned().collect();
    }
    vec![]
}

/// Clear all documentation from the global registry.
///
/// Primarily useful for testing.
pub fn clear() {
    let mut registry = write_registry();
    if let Some(ref mut reg) = *registry {
        reg.clear();
    }
}

fn write_registry() -> RwLockWriteGuard<'static, Option<DocRegistry>> {
    match DOC_REGISTRY.write() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

fn read_registry() -> RwLockReadGuard<'static, Option<DocRegistry>> {
    match DOC_REGISTRY.read() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scripting::docs::types::{FunctionSignature, ParamDoc, PropertyDoc, ReturnDoc};

    fn create_test_function(name: &str, module: &str) -> FunctionDoc {
        FunctionDoc {
            name: name.to_string(),
            module: module.to_string(),
            signature: FunctionSignature {
                params: vec![("key".to_string(), "KeyCode".to_string())],
                return_type: Some("()".to_string()),
            },
            description: format!("{} function", name),
            parameters: vec![ParamDoc {
                name: "key".to_string(),
                type_name: "KeyCode".to_string(),
                description: "Key parameter".to_string(),
                optional: false,
                default: None,
            }],
            returns: Some(ReturnDoc {
                type_name: "()".to_string(),
                description: "Nothing".to_string(),
            }),
            examples: vec![format!("{}();", name)],
            since: Some("0.1.0".to_string()),
            deprecated: None,
            notes: None,
        }
    }

    fn create_test_type(name: &str, module: &str) -> TypeDoc {
        TypeDoc {
            name: name.to_string(),
            description: format!("{} type", name),
            methods: vec![],
            properties: vec![PropertyDoc {
                name: "value".to_string(),
                type_name: "int".to_string(),
                description: "Test property".to_string(),
                readonly: true,
            }],
            constructors: vec![],
            module: module.to_string(),
            since: Some("0.1.0".to_string()),
            examples: vec![],
        }
    }

    fn create_test_module(name: &str) -> ModuleDoc {
        ModuleDoc {
            name: name.to_string(),
            description: format!("{} module", name),
            functions: vec![],
            types: vec![],
        }
    }

    #[test]
    fn test_registry_creation() {
        let registry = DocRegistry::new();
        assert_eq!(registry.function_count(), 0);
        assert_eq!(registry.type_count(), 0);
        assert_eq!(registry.module_count(), 0);
    }

    #[test]
    fn test_register_and_get_function() {
        let mut registry = DocRegistry::new();
        let func = create_test_function("emit_key", "keys");

        registry.register_function(func.clone());
        assert_eq!(registry.function_count(), 1);

        let retrieved = registry.get_function("keys", "emit_key");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "emit_key");
    }

    #[test]
    fn test_register_and_get_type() {
        let mut registry = DocRegistry::new();
        let type_doc = create_test_type("KeyCode", "keys");

        registry.register_type(type_doc.clone());
        assert_eq!(registry.type_count(), 1);

        let retrieved = registry.get_type("KeyCode");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "KeyCode");
    }

    #[test]
    fn test_register_and_get_module() {
        let mut registry = DocRegistry::new();
        let module = create_test_module("keys");

        registry.register_module(module.clone());
        assert_eq!(registry.module_count(), 1);

        let retrieved = registry.get_module("keys");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "keys");
    }

    #[test]
    fn test_all_functions() {
        let mut registry = DocRegistry::new();
        registry.register_function(create_test_function("emit_key", "keys"));
        registry.register_function(create_test_function("release_key", "keys"));

        let all_funcs: Vec<_> = registry.all_functions().collect();
        assert_eq!(all_funcs.len(), 2);
    }

    #[test]
    fn test_functions_in_module() {
        let mut registry = DocRegistry::new();
        registry.register_function(create_test_function("emit_key", "keys"));
        registry.register_function(create_test_function("switch_layer", "layers"));
        registry.register_function(create_test_function("release_key", "keys"));

        let keys_funcs: Vec<_> = registry.functions_in_module("keys").collect();
        assert_eq!(keys_funcs.len(), 2);

        let layer_funcs: Vec<_> = registry.functions_in_module("layers").collect();
        assert_eq!(layer_funcs.len(), 1);
    }

    #[test]
    fn test_types_in_module() {
        let mut registry = DocRegistry::new();
        registry.register_type(create_test_type("KeyCode", "keys"));
        registry.register_type(create_test_type("Layer", "layers"));
        registry.register_type(create_test_type("Modifier", "keys"));

        let keys_types: Vec<_> = registry.types_in_module("keys").collect();
        assert_eq!(keys_types.len(), 2);

        let layer_types: Vec<_> = registry.types_in_module("layers").collect();
        assert_eq!(layer_types.len(), 1);
    }

    #[test]
    fn test_clear() {
        let mut registry = DocRegistry::new();
        registry.register_function(create_test_function("emit_key", "keys"));
        registry.register_type(create_test_type("KeyCode", "keys"));
        registry.register_module(create_test_module("keys"));

        assert_eq!(registry.function_count(), 1);
        assert_eq!(registry.type_count(), 1);
        assert_eq!(registry.module_count(), 1);

        registry.clear();

        assert_eq!(registry.function_count(), 0);
        assert_eq!(registry.type_count(), 0);
        assert_eq!(registry.module_count(), 0);
    }

    #[test]
    fn test_function_not_found() {
        let registry = DocRegistry::new();
        let result = registry.get_function("keys", "nonexistent");
        assert!(result.is_none());
    }

    #[test]
    fn test_type_not_found() {
        let registry = DocRegistry::new();
        let result = registry.get_type("NonExistent");
        assert!(result.is_none());
    }

    #[test]
    fn test_module_not_found() {
        let registry = DocRegistry::new();
        let result = registry.get_module("nonexistent");
        assert!(result.is_none());
    }

    #[test]
    #[serial_test::serial]
    fn test_global_registry_initialization() {
        // Note: This test may interfere with other tests if run in parallel
        // In a real scenario, you'd want to use a test-specific registry
        initialize();

        let func = create_test_function("test_func", "test");
        register_function(func.clone());

        let retrieved = get_function("test", "test_func");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "test_func");

        clear();
    }

    #[test]
    #[serial_test::serial]
    fn test_global_registry_all_functions() {
        initialize();
        clear(); // Clear any previous state

        register_function(create_test_function("func1", "mod1"));
        register_function(create_test_function("func2", "mod2"));

        let funcs = all_functions();
        assert_eq!(funcs.len(), 2);

        clear();
    }

    #[test]
    #[serial_test::serial]
    fn test_global_registry_functions_in_module() {
        initialize();
        clear();

        register_function(create_test_function("func1", "mod1"));
        register_function(create_test_function("func2", "mod1"));
        register_function(create_test_function("func3", "mod2"));

        let mod1_funcs = functions_in_module("mod1");
        assert_eq!(mod1_funcs.len(), 2);

        clear();
    }
}
