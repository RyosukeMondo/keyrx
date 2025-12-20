//! Model class generator for Dart
//!
//! Generates Dart model classes from FFI contract type definitions,
//! including fromJson and toJson serialization methods.

use crate::templates::{
    context, render, to_camel_case, to_pascal_case, to_snake_case, CONSTRUCTOR_PARAM_OPTIONAL,
    CONSTRUCTOR_PARAM_REQUIRED, ENUM_CLASS_TEMPLATE, FROM_JSON_OPTIONAL, FROM_JSON_REQUIRED,
    MODEL_CLASS_TEMPLATE, MODEL_FIELD_TEMPLATE, TO_JSON_FIELD,
};
use crate::type_mapper::{base_type, is_nullable, map_to_dart_native_type, TypeMappingError};
use keyrx_core::ffi::contract::{FfiContract, TypeDefinition};
use std::collections::{HashMap, HashSet};

/// Error type for model generation
#[derive(Debug, Clone)]
pub struct ModelGenError {
    pub type_name: String,
    pub message: String,
}

impl std::fmt::Display for ModelGenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Failed to generate model for '{}': {}",
            self.type_name, self.message
        )
    }
}

impl std::error::Error for ModelGenError {}

impl From<TypeMappingError> for ModelGenError {
    fn from(err: TypeMappingError) -> Self {
        ModelGenError {
            type_name: String::new(),
            message: err.to_string(),
        }
    }
}

/// Generated Dart model class
#[derive(Debug, Clone)]
pub struct GeneratedModel {
    /// The Dart class name (PascalCase)
    pub class_name: String,
    /// The full generated class code
    pub code: String,
}

/// Generate model classes from contract's custom types
pub fn generate_models_from_types(
    contract: &FfiContract,
    enum_names: &HashSet<String>,
) -> Result<Vec<GeneratedModel>, ModelGenError> {
    contract
        .types
        .iter()
        .filter_map(|(name, type_def)| {
            if type_def.is_object() {
                Some(generate_model_class(name, type_def, enum_names))
            } else if type_def.is_enum() {
                Some(generate_enum_class(name, type_def))
            } else {
                None // Skip primitives and arrays
            }
        })
        .collect()
}

/// Generate model classes from function return types that are objects
pub fn generate_models_from_returns(
    contract: &FfiContract,
    enum_names: &HashSet<String>,
) -> Result<Vec<GeneratedModel>, ModelGenError> {
    let mut models = Vec::new();

    for func in &contract.functions {
        if let TypeDefinition::Object { properties, .. } = &func.returns {
            if !properties.is_empty() {
                let class_name = format!("{}Result", to_pascal_case(&func.name));
                let model =
                    generate_model_from_properties(&class_name, properties, None, enum_names)?;
                models.push(model);
            }
        }
    }

    Ok(models)
}

/// Generate all models for a contract (both custom types and return types)
pub fn generate_all_models(contract: &FfiContract) -> Result<Vec<GeneratedModel>, ModelGenError> {
    // Collect enum names to distinguish between Enums and Models during deserialization
    let enum_names: HashSet<String> = contract
        .types
        .iter()
        .filter(|(_, t)| t.is_enum())
        .map(|(n, _)| to_pascal_case(n))
        .collect();

    let mut models = generate_models_from_types(contract, &enum_names)?;
    let return_models = generate_models_from_returns(contract, &enum_names)?;
    models.extend(return_models);
    Ok(models)
}

/// Generate a Dart model class from a named type definition
fn generate_model_class(
    name: &str,
    type_def: &TypeDefinition,
    enum_names: &HashSet<String>,
) -> Result<GeneratedModel, ModelGenError> {
    let class_name = to_pascal_case(name);

    match type_def {
        TypeDefinition::Object {
            description,
            properties,
            ..
        } => generate_model_from_properties(
            &class_name,
            properties,
            description.as_deref(),
            enum_names,
        ),
        _ => Err(ModelGenError {
            type_name: name.to_string(),
            message: "Only object types can be converted to model classes".to_string(),
        }),
    }
}

/// Generate a Dart enum from an enum type definition
fn generate_enum_class(
    name: &str,
    type_def: &TypeDefinition,
) -> Result<GeneratedModel, ModelGenError> {
    let enum_name = to_pascal_case(name);

    match type_def {
        TypeDefinition::Enum { values, .. } => {
            // Convert enum values to camelCase for Dart enum variants
            let dart_values: Vec<String> = values.iter().map(|v| to_camel_case(v)).collect();

            // Generate enum values (e.g., "inProgress, completed, cancelled")
            let enum_values = dart_values.join(", ");

            // Generate fromJson cases
            let from_json_cases: Vec<String> = values
                .iter()
                .zip(dart_values.iter())
                .map(|(json_val, dart_val)| {
                    format!(
                        "      case '{}': return {}.{};",
                        json_val, enum_name, dart_val
                    )
                })
                .collect();
            let from_json_cases_str = from_json_cases.join("\n");

            // Generate toJson cases
            let to_json_cases: Vec<String> = values
                .iter()
                .zip(dart_values.iter())
                .map(|(json_val, dart_val)| {
                    format!(
                        "      case {}.{}: return '{}';",
                        enum_name, dart_val, json_val
                    )
                })
                .collect();
            let to_json_cases_str = to_json_cases.join("\n");

            // Use type_def description if available, otherwise generate a default
            let doc_comment = format!("Enum for {}", name);

            let mut ctx = context();
            ctx.insert("doc_comment".to_string(), doc_comment);
            ctx.insert("enum_name".to_string(), enum_name.clone());
            ctx.insert("enum_values".to_string(), enum_values);
            ctx.insert("from_json_cases".to_string(), from_json_cases_str);
            ctx.insert("to_json_cases".to_string(), to_json_cases_str);

            let code = render(ENUM_CLASS_TEMPLATE, &ctx);

            Ok(GeneratedModel {
                class_name: enum_name,
                code,
            })
        }
        _ => Err(ModelGenError {
            type_name: name.to_string(),
            message: "Only enum types can be converted to enum classes".to_string(),
        }),
    }
}

/// Generate a Dart model class from a properties map
fn generate_model_from_properties(
    class_name: &str,
    properties: &HashMap<String, Box<TypeDefinition>>,
    description: Option<&str>,
    enum_names: &HashSet<String>,
) -> Result<GeneratedModel, ModelGenError> {
    let mut fields = Vec::new();
    let mut constructor_params = Vec::new();
    let mut from_json_body = Vec::new();
    let mut to_json_body = Vec::new();

    // Sort properties for deterministic output
    let mut sorted_props: Vec<_> = properties.iter().collect();
    sorted_props.sort_by_key(|(k, _)| *k);

    for (prop_name, prop_type) in sorted_props {
        let field = generate_field(prop_name, prop_type, enum_names)?;
        fields.push(field.declaration);
        constructor_params.push(field.constructor_param);
        from_json_body.push(field.from_json);
        to_json_body.push(field.to_json);
    }

    let default_doc = format!("Model class for {}", class_name);
    let doc_comment = description.unwrap_or(&default_doc);

    let mut ctx = context();
    ctx.insert("class_name".to_string(), class_name.to_string());
    ctx.insert("doc_comment".to_string(), doc_comment.to_string());
    ctx.insert("fields".to_string(), fields.join("\n"));
    ctx.insert(
        "constructor_params".to_string(),
        constructor_params.join("\n"),
    );
    ctx.insert("from_json_body".to_string(), from_json_body.join("\n"));
    ctx.insert("to_json_body".to_string(), to_json_body.join("\n"));

    let code = render(MODEL_CLASS_TEMPLATE, &ctx);

    Ok(GeneratedModel {
        class_name: class_name.to_string(),
        code,
    })
}

/// Generated field information
struct FieldInfo {
    declaration: String,
    constructor_param: String,
    from_json: String,
    to_json: String,
}

/// Generate field information for a single property
fn generate_field(
    prop_name: &str,
    prop_type: &TypeDefinition,
    enum_names: &HashSet<String>,
) -> Result<FieldInfo, ModelGenError> {
    let type_name = prop_type.type_name();
    let nullable = is_nullable(type_name);
    let base = base_type(type_name);

    let dart_type = map_dart_type_for_property(base, prop_type)?;
    let dart_type_with_nullable = if nullable {
        format!("{}?", dart_type)
    } else {
        dart_type.clone()
    };

    let field_name = to_camel_case(prop_name);
    let json_key = to_snake_case(prop_name);

    // Generate field declaration
    let mut field_ctx = context();
    field_ctx.insert("dart_type".to_string(), dart_type_with_nullable.clone());
    field_ctx.insert("field_name".to_string(), field_name.clone());
    let declaration = render(MODEL_FIELD_TEMPLATE, &field_ctx);

    // Generate constructor parameter
    let constructor_param = if nullable {
        let mut param_ctx = context();
        param_ctx.insert("field_name".to_string(), field_name.clone());
        render(CONSTRUCTOR_PARAM_OPTIONAL, &param_ctx)
    } else {
        let mut param_ctx = context();
        param_ctx.insert("field_name".to_string(), field_name.clone());
        render(CONSTRUCTOR_PARAM_REQUIRED, &param_ctx)
    };

    // Generate fromJson assignment using robust deserialization logic
    let json_access = format!("json['{}']", json_key);
    let dart_type_base = base_type(&dart_type);
    let is_enum = enum_names.contains(dart_type_base);
    let deserialization = get_deserialization(&json_access, &dart_type, nullable, is_enum);
    let from_json = format!("      {}: {},", field_name, deserialization);

    // Generate toJson field
    let mut to_json_ctx = context();
    to_json_ctx.insert("json_key".to_string(), json_key);
    to_json_ctx.insert("field_name".to_string(), field_name);
    let to_json = render(TO_JSON_FIELD, &to_json_ctx);

    Ok(FieldInfo {
        declaration,
        constructor_param,
        from_json,
        to_json,
    })
}

fn is_primitive_dart_type(ty: &str) -> bool {
    matches!(
        ty,
        "int" | "double" | "bool" | "String" | "dynamic" | "num" | "Object" | "void"
    ) || ty.starts_with("Map<")
}

fn get_deserialization(json_expr: &str, dart_type: &str, nullable: bool, is_enum: bool) -> String {
    if is_primitive_dart_type(dart_type) {
        if nullable {
            format!("{} as {}?", json_expr, dart_type)
        } else {
            format!("{} as {}", json_expr, dart_type)
        }
    } else if dart_type.starts_with("List<") {
        let inner = &dart_type[5..dart_type.len() - 1];
        if is_primitive_dart_type(inner) {
            // List of primitives: cast
            if nullable {
                format!("({} as List?)?.cast<{}>().toList()", json_expr, inner)
            } else {
                format!("({} as List).cast<{}>().toList()", json_expr, inner)
            }
        } else {
            // List of objects: map to fromJson
            if nullable {
                format!(
                    "({} as List?)?.map((e) => {}.fromJson(e as Map<String, dynamic>)).toList()",
                    json_expr, inner
                )
            } else {
                format!(
                    "({} as List).map((e) => {}.fromJson(e as Map<String, dynamic>)).toList()",
                    json_expr, inner
                )
            }
        }
    } else {
        // Custom type (Model or Enum)
        if is_enum {
            // Enums are strings in JSON, mapped via fromJson(String)
            if nullable {
                format!(
                    "{} == null ? null : {}.fromJson({} as String)",
                    json_expr, dart_type, json_expr
                )
            } else {
                format!("{}.fromJson({} as String)", dart_type, json_expr)
            }
        } else {
            // Classes are Objects in JSON, mapped via fromJson(Map<String, dynamic>)
            if nullable {
                format!(
                    "{} == null ? null : {}.fromJson({} as Map<String, dynamic>)",
                    json_expr, dart_type, json_expr
                )
            } else {
                format!(
                    "{}.fromJson({} as Map<String, dynamic>)",
                    dart_type, json_expr
                )
            }
        }
    }
}

/// Map a property type to its Dart type string
fn map_dart_type_for_property(
    base_type: &str,
    type_def: &TypeDefinition,
) -> Result<String, ModelGenError> {
    // Handle nested objects
    if type_def.is_object() {
        return Ok("Map<String, dynamic>".to_string());
    }

    // Handle arrays
    if type_def.is_array() {
        if let TypeDefinition::Array { items, .. } = type_def {
            let item_type = map_dart_type_for_property(items.type_name(), items)?;
            return Ok(format!("List<{}>", item_type));
        }
    }

    // Map primitive types
    map_to_dart_native_type(base_type).map_err(|e| ModelGenError {
        type_name: base_type.to_string(),
        message: e.to_string(),
    })
}

/// Generate all models as a single string block
pub fn generate_models_block(models: &[GeneratedModel]) -> String {
    models
        .iter()
        .map(|m| m.code.clone())
        .collect::<Vec<_>>()
        .join("\n\n")
}
