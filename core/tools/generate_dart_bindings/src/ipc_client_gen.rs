//! Generates Dart IPC client code from FFI contracts.

use crate::templates::{
    context, render, to_camel_case, to_pascal_case, IPC_CLIENT_CLASS_TEMPLATE,
    IPC_CLIENT_FILE_HEADER, IPC_EVENT_TEMPLATE, IPC_METHOD_TEMPLATE,
};
use crate::type_mapper::map_to_dart_native_type;
use anyhow::Result;
use chrono;
use keyrx_core::ffi::contract::FfiContract;
use std::collections::HashSet;

pub struct IpcClientGenerator {
    // Generation state, if any
}

impl Default for IpcClientGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl IpcClientGenerator {
    pub fn new() -> Self {
        Self {}
    }

    pub fn generate(&self, contracts: &[FfiContract]) -> Result<String> {
        let mut output = String::new();

        // Add file header
        let mut header_ctx = context();
        header_ctx.insert("timestamp".to_string(), chrono::Utc::now().to_rfc2822());
        output.push_str(&render(IPC_CLIENT_FILE_HEADER, &header_ctx));

        for contract in contracts {
            output.push_str(&generate_client_class(contract)?);
            output.push_str("\n\n");
        }
        Ok(output)
    }
}

pub fn generate_client_class(contract: &FfiContract) -> Result<String> {
    let class_name = format!("{}Client", to_pascal_case(&contract.domain));
    let domain = contract.domain.clone();

    let enum_names: HashSet<String> = contract
        .types
        .iter()
        .filter(|(_, t)| t.is_enum())
        .map(|(n, _)| to_pascal_case(n))
        .collect();

    let methods = generate_methods(contract, &enum_names)?;
    let events = generate_events(contract, &enum_names)?;

    let mut ctx = context();
    ctx.insert("class_name".to_string(), class_name);
    ctx.insert("domain".to_string(), domain);
    ctx.insert("methods".to_string(), methods);
    ctx.insert("events".to_string(), events);

    Ok(render(IPC_CLIENT_CLASS_TEMPLATE, &ctx))
}

fn generate_events(contract: &FfiContract, enum_names: &HashSet<String>) -> Result<String> {
    let mut events = Vec::new();

    for event in &contract.events {
        let event_name = to_pascal_case(&event.name);
        let event_type = map_to_dart_native_type(event.payload.type_name())?;

        let event_mapper = if event_type.starts_with("Map<") {
            "event.params".to_string()
        } else if enum_names.contains(&event_type) {
            format!("{}.fromJson(event.params as String)", event_type)
        } else {
            format!(
                "{}.fromJson(event.params as Map<String, dynamic>)",
                event_type
            )
        };

        let mut ctx = context();
        ctx.insert("doc_comment".to_string(), event.description.clone());
        ctx.insert("event_name".to_string(), event_name);
        ctx.insert("event_type".to_string(), event_type);
        ctx.insert("event_mapper".to_string(), event_mapper);
        ctx.insert(
            "event_full_name".to_string(),
            format!("{}.{}", contract.domain, event.name),
        );

        events.push(render(IPC_EVENT_TEMPLATE, &ctx));
    }

    Ok(events.join("\n"))
}

fn generate_methods(contract: &FfiContract, enum_names: &HashSet<String>) -> Result<String> {
    let mut methods = Vec::new();

    for func in &contract.functions {
        let method_name = to_camel_case(&func.name);
        let return_type = map_to_dart_native_type(func.returns.type_name())?;

        // Check for incompatible types (Pointers/Functions) that can't be serialized over IPC
        if return_type.contains("Pointer") || return_type.contains("Function") {
            eprintln!(
                "Warning: Skipping IPC generation for {}.{} due to incompatible return type: {}",
                contract.domain, func.name, return_type
            );
            continue;
        }

        let mut incompatible_param = None;
        for p in &func.parameters {
            let dart_type = map_to_dart_native_type(&p.param_type)?;
            if dart_type.contains("Pointer") || dart_type.contains("Function") {
                incompatible_param = Some((p.name.clone(), dart_type));
                break;
            }
        }

        if let Some((name, type_)) = incompatible_param {
            eprintln!(
                "Warning: Skipping IPC generation for {}.{} due to incompatible parameter '{}' of type '{}'",
                contract.domain, func.name, name, type_
            );
            continue;
        }

        let param_signature = func
            .parameters
            .iter()
            .map(|p| {
                let dart_type = map_to_dart_native_type(&p.param_type)?;
                Ok(format!("{} {}", dart_type, p.name))
            })
            .collect::<Result<Vec<_>>>()?
            .join(", ");

        let param_map = func
            .parameters
            .iter()
            .map(|p| format!("        '{}': {},", p.name, p.name))
            .collect::<Vec<_>>()
            .join("\n");

        let result_deserialization = if return_type == "void" {
            "    return;".to_string()
        } else if return_type.starts_with("List<") {
            let inner_type = &return_type[5..return_type.len() - 1];
            if enum_names.contains(inner_type) {
                format!(
                    "    return (result as List).map((e) => {}.fromJson(e as String)).toList();",
                    inner_type
                )
            } else {
                format!(
                    "    return (result as List).map((e) => {}.fromJson(e as Map<String, dynamic>)).toList();",
                    inner_type
                )
            }
        } else if return_type == "String"
            || return_type == "int"
            || return_type == "double"
            || return_type == "bool"
            || return_type.starts_with("Map<")
        {
            format!("    return result as {};", return_type)
        } else {
            // Check if it's an enum or class
            if enum_names.contains(&return_type) {
                format!("    return {}.fromJson(result as String);", return_type)
            } else {
                format!(
                    "    return {}.fromJson(result as Map<String, dynamic>);",
                    return_type
                )
            }
        };

        let maybe_result_assignment = if return_type == "void" {
            "".to_string()
        } else {
            "final result = ".to_string()
        };

        let mut ctx = context();
        ctx.insert("doc_comment".to_string(), func.description.clone());
        ctx.insert("return_type".to_string(), return_type);
        ctx.insert("method_name".to_string(), method_name);
        ctx.insert("param_signature".to_string(), param_signature);
        ctx.insert(
            "method_full_name".to_string(),
            format!("{}.{}", contract.domain, func.name),
        );
        ctx.insert(
            "maybe_result_assignment".to_string(),
            maybe_result_assignment,
        );
        ctx.insert("param_map".to_string(), param_map);
        ctx.insert("result_deserialization".to_string(), result_deserialization);

        methods.push(render(IPC_METHOD_TEMPLATE, &ctx));
    }

    Ok(methods.join("\n"))
}
