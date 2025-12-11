//! Model class generator for Dart
//!
//! Generates Dart model classes from FFI contract type definitions,
//! including fromJson and toJson serialization methods.

use crate::templates::{
    context, render, to_camel_case, to_pascal_case, to_snake_case, CONSTRUCTOR_PARAM_OPTIONAL,
    CONSTRUCTOR_PARAM_REQUIRED, FROM_JSON_OPTIONAL, FROM_JSON_REQUIRED, MODEL_CLASS_TEMPLATE,
    MODEL_FIELD_TEMPLATE, TO_JSON_FIELD,
};
use crate::type_mapper::{base_type, is_nullable, map_to_dart_native_type, TypeMappingError};
use keyrx_core::ffi::contract::{FfiContract, TypeDefinition};
use std::collections::HashMap;

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
) -> Result<Vec<GeneratedModel>, ModelGenError> {
    contract
        .types
        .iter()
        .filter_map(|(name, type_def)| {
            if type_def.is_object() {
                Some(generate_model_class(name, type_def))
            } else {
                None // Skip non-object types (primitives, enums, etc.)
            }
        })
        .collect()
}

/// Generate model classes from function return types that are objects
pub fn generate_models_from_returns(
    contract: &FfiContract,
) -> Result<Vec<GeneratedModel>, ModelGenError> {
    let mut models = Vec::new();

    for func in &contract.functions {
        if let TypeDefinition::Object { properties, .. } = &func.returns {
            if !properties.is_empty() {
                let class_name = format!("{}Result", to_pascal_case(&func.name));
                let model = generate_model_from_properties(&class_name, properties, None)?;
                models.push(model);
            }
        }
    }

    Ok(models)
}

/// Generate all models for a contract (both custom types and return types)
pub fn generate_all_models(contract: &FfiContract) -> Result<Vec<GeneratedModel>, ModelGenError> {
    let mut models = generate_models_from_types(contract)?;
    let return_models = generate_models_from_returns(contract)?;
    models.extend(return_models);
    Ok(models)
}

/// Generate a Dart model class from a named type definition
fn generate_model_class(
    name: &str,
    type_def: &TypeDefinition,
) -> Result<GeneratedModel, ModelGenError> {
    let class_name = to_pascal_case(name);

    match type_def {
        TypeDefinition::Object {
            description,
            properties,
            ..
        } => generate_model_from_properties(&class_name, properties, description.as_deref()),
        _ => Err(ModelGenError {
            type_name: name.to_string(),
            message: "Only object types can be converted to model classes".to_string(),
        }),
    }
}

/// Generate a Dart model class from a properties map
fn generate_model_from_properties(
    class_name: &str,
    properties: &HashMap<String, Box<TypeDefinition>>,
    description: Option<&str>,
) -> Result<GeneratedModel, ModelGenError> {
    let mut fields = Vec::new();
    let mut constructor_params = Vec::new();
    let mut from_json_body = Vec::new();
    let mut to_json_body = Vec::new();

    // Sort properties for deterministic output
    let mut sorted_props: Vec<_> = properties.iter().collect();
    sorted_props.sort_by_key(|(k, _)| *k);

    for (prop_name, prop_type) in sorted_props {
        let field = generate_field(prop_name, prop_type)?;
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
fn generate_field(prop_name: &str, prop_type: &TypeDefinition) -> Result<FieldInfo, ModelGenError> {
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

    // Generate fromJson assignment
    let dart_cast = get_dart_cast(&dart_type);
    let from_json = if nullable {
        let mut json_ctx = context();
        json_ctx.insert("field_name".to_string(), field_name.clone());
        json_ctx.insert("json_key".to_string(), json_key.clone());
        json_ctx.insert(
            "dart_cast_nullable".to_string(),
            format!("as {}?", dart_type),
        );
        render(FROM_JSON_OPTIONAL, &json_ctx)
    } else {
        let mut json_ctx = context();
        json_ctx.insert("field_name".to_string(), field_name.clone());
        json_ctx.insert("json_key".to_string(), json_key.clone());
        json_ctx.insert("dart_cast".to_string(), dart_cast);
        render(FROM_JSON_REQUIRED, &json_ctx)
    };

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

/// Get the Dart type cast for fromJson
fn get_dart_cast(dart_type: &str) -> String {
    format!("as {}", dart_type)
}

/// Generate all models as a single string block
pub fn generate_models_block(models: &[GeneratedModel]) -> String {
    models
        .iter()
        .map(|m| m.code.clone())
        .collect::<Vec<_>>()
        .join("\n\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use keyrx_core::ffi::contract::FunctionContract;

    fn create_object_type() -> TypeDefinition {
        let mut properties = HashMap::new();
        properties.insert(
            "device_count".to_string(),
            Box::new(TypeDefinition::Primitive {
                type_name: "uint32".to_string(),
                description: Some("Number of devices".to_string()),
                constraints: None,
            }),
        );
        properties.insert(
            "is_active".to_string(),
            Box::new(TypeDefinition::Primitive {
                type_name: "bool".to_string(),
                description: Some("Whether active".to_string()),
                constraints: None,
            }),
        );
        properties.insert(
            "name".to_string(),
            Box::new(TypeDefinition::Primitive {
                type_name: "string".to_string(),
                description: Some("Device name".to_string()),
                constraints: None,
            }),
        );

        TypeDefinition::Object {
            type_name: "object".to_string(),
            description: Some("A device profile".to_string()),
            properties,
        }
    }

    #[test]
    fn test_generate_model_class() {
        let type_def = create_object_type();
        let model = generate_model_class("DeviceProfile", &type_def).unwrap();

        assert_eq!(model.class_name, "DeviceProfile");
        assert!(model.code.contains("class DeviceProfile"));
        assert!(model.code.contains("final int deviceCount"));
        assert!(model.code.contains("final bool isActive"));
        assert!(model.code.contains("final String name"));
    }

    #[test]
    fn test_model_has_constructor() {
        let type_def = create_object_type();
        let model = generate_model_class("TestModel", &type_def).unwrap();

        assert!(model.code.contains("TestModel({"));
        assert!(model.code.contains("required this.deviceCount"));
        assert!(model.code.contains("required this.isActive"));
        assert!(model.code.contains("required this.name"));
    }

    #[test]
    fn test_model_has_from_json() {
        let type_def = create_object_type();
        let model = generate_model_class("TestModel", &type_def).unwrap();

        assert!(model.code.contains("factory TestModel.fromJson"));
        assert!(model.code.contains("json['device_count']"));
        assert!(model.code.contains("json['is_active']"));
        assert!(model.code.contains("json['name']"));
    }

    #[test]
    fn test_model_has_to_json() {
        let type_def = create_object_type();
        let model = generate_model_class("TestModel", &type_def).unwrap();

        assert!(model.code.contains("Map<String, dynamic> toJson()"));
        assert!(model.code.contains("'device_count': deviceCount"));
        assert!(model.code.contains("'is_active': isActive"));
        assert!(model.code.contains("'name': name"));
    }

    #[test]
    fn test_field_type_mapping() {
        let mut properties = HashMap::new();
        properties.insert(
            "count".to_string(),
            Box::new(TypeDefinition::Primitive {
                type_name: "int32".to_string(),
                description: None,
                constraints: None,
            }),
        );
        properties.insert(
            "progress".to_string(),
            Box::new(TypeDefinition::Primitive {
                type_name: "float64".to_string(),
                description: None,
                constraints: None,
            }),
        );

        let type_def = TypeDefinition::Object {
            type_name: "object".to_string(),
            description: None,
            properties,
        };

        let model = generate_model_class("Stats", &type_def).unwrap();

        assert!(model.code.contains("final int count"));
        assert!(model.code.contains("final double progress"));
    }

    #[test]
    fn test_generate_models_from_returns() {
        let contract = FfiContract {
            schema: "https://keyrx.dev/schemas/ffi-contract-v1.json".to_string(),
            version: "1.0.0".to_string(),
            domain: "test".to_string(),
            description: "Test contract".to_string(),
            protocol_version: 1,
            functions: vec![FunctionContract {
                name: "get_status".to_string(),
                description: "Get status".to_string(),
                rust_name: None,
                parameters: vec![],
                returns: create_object_type(),
                errors: vec![],
                events_emitted: vec![],
                example: None,
                deprecated: false,
                since_version: None,
            }],
            types: HashMap::new(),
            events: vec![],
        };

        let models = generate_models_from_returns(&contract).unwrap();

        assert_eq!(models.len(), 1);
        assert_eq!(models[0].class_name, "GetStatusResult");
    }

    #[test]
    fn test_generate_all_models() {
        let mut custom_types = HashMap::new();
        custom_types.insert("DeviceInfo".to_string(), create_object_type());

        let contract = FfiContract {
            schema: "https://keyrx.dev/schemas/ffi-contract-v1.json".to_string(),
            version: "1.0.0".to_string(),
            domain: "test".to_string(),
            description: "Test contract".to_string(),
            protocol_version: 1,
            functions: vec![FunctionContract {
                name: "get_device".to_string(),
                description: "Get device".to_string(),
                rust_name: None,
                parameters: vec![],
                returns: create_object_type(),
                errors: vec![],
                events_emitted: vec![],
                example: None,
                deprecated: false,
                since_version: None,
            }],
            types: custom_types,
            events: vec![],
        };

        let models = generate_all_models(&contract).unwrap();

        // Should have both the custom type and the return type model
        assert_eq!(models.len(), 2);
        let names: Vec<_> = models.iter().map(|m| m.class_name.as_str()).collect();
        assert!(names.contains(&"DeviceInfo"));
        assert!(names.contains(&"GetDeviceResult"));
    }

    #[test]
    fn test_generate_models_block() {
        let type_def = create_object_type();
        let model1 = generate_model_class("ModelA", &type_def).unwrap();
        let model2 = generate_model_class("ModelB", &type_def).unwrap();

        let block = generate_models_block(&[model1, model2]);

        assert!(block.contains("class ModelA"));
        assert!(block.contains("class ModelB"));
    }

    #[test]
    fn test_non_object_type_returns_error() {
        let type_def = TypeDefinition::Primitive {
            type_name: "string".to_string(),
            description: None,
            constraints: None,
        };

        let result = generate_model_class("InvalidModel", &type_def);
        assert!(result.is_err());
    }

    #[test]
    fn test_camel_case_field_names() {
        let mut properties = HashMap::new();
        properties.insert(
            "device_count".to_string(),
            Box::new(TypeDefinition::Primitive {
                type_name: "int32".to_string(),
                description: None,
                constraints: None,
            }),
        );

        let type_def = TypeDefinition::Object {
            type_name: "object".to_string(),
            description: None,
            properties,
        };

        let model = generate_model_class("Test", &type_def).unwrap();

        // Field should be camelCase
        assert!(model.code.contains("deviceCount"));
        // JSON key should be snake_case
        assert!(model.code.contains("'device_count'"));
    }

    #[test]
    fn test_doc_comment_in_class() {
        let type_def = create_object_type();
        let model = generate_model_class("Device", &type_def).unwrap();

        assert!(model.code.contains("/// A device profile"));
    }
}
