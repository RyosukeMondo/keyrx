//! HTML rendering functions for documentation generation.
//!
//! Contains functions that render documentation elements (modules, types, functions,
//! parameters, etc.) to HTML strings.

use crate::scripting::docs::{FunctionDoc, FunctionSignature, ParamDoc, PropertyDoc, TypeDoc};

use super::ModuleData;

/// Generates HTML for a single module.
pub fn generate_module_html(output: &mut String, module_name: &str, module: &ModuleData) {
    let anchor = module_name.to_lowercase().replace(' ', "-");

    output.push_str(&format!("<section class=\"module\" id=\"{}\">\n", anchor));
    output.push_str(&format!("<h2>{}</h2>\n", escape_html(module_name)));

    // Module description
    if let Some(description) = &module.description {
        output.push_str(&format!(
            "<p class=\"module-desc\">{}</p>\n",
            escape_html(description)
        ));
    }

    // Types section
    if !module.types.is_empty() {
        output.push_str("<div class=\"types-section\">\n");
        output.push_str("<h3>Types</h3>\n");
        for type_doc in &module.types {
            generate_type_html(output, type_doc);
        }
        output.push_str("</div>\n");
    }

    // Functions section
    if !module.functions.is_empty() {
        output.push_str("<div class=\"functions-section\">\n");
        output.push_str("<h3>Functions</h3>\n");
        for func in &module.functions {
            generate_function_html(output, func);
        }
        output.push_str("</div>\n");
    }

    output.push_str("</section>\n");
}

/// Generates HTML for a single type.
pub fn generate_type_html(output: &mut String, type_doc: &TypeDoc) {
    output.push_str("<div class=\"type-doc\">\n");
    output.push_str(&format!(
        "<h4><code>{}</code></h4>\n",
        escape_html(&type_doc.name)
    ));
    output.push_str(&format!("<p>{}</p>\n", escape_html(&type_doc.description)));

    // Version info
    if let Some(since) = &type_doc.since {
        output.push_str(&format!(
            "<p class=\"version\"><em>Since: {}</em></p>\n",
            escape_html(since)
        ));
    }

    // Properties
    if !type_doc.properties.is_empty() {
        output.push_str("<div class=\"properties\">\n");
        output.push_str("<strong>Properties:</strong>\n");
        output.push_str("<ul>\n");
        for prop in &type_doc.properties {
            generate_property_html(output, prop);
        }
        output.push_str("</ul>\n");
        output.push_str("</div>\n");
    }

    // Methods
    if !type_doc.methods.is_empty() {
        output.push_str("<div class=\"methods\">\n");
        output.push_str("<strong>Methods:</strong>\n");
        output.push_str("<ul>\n");
        for method in &type_doc.methods {
            generate_method_html(output, method);
        }
        output.push_str("</ul>\n");
        output.push_str("</div>\n");
    }

    // Constructors
    if !type_doc.constructors.is_empty() {
        output.push_str("<div class=\"constructors\">\n");
        output.push_str("<strong>Constructors:</strong>\n");
        output.push_str("<ul>\n");
        for constructor in &type_doc.constructors {
            generate_method_html(output, constructor);
        }
        output.push_str("</ul>\n");
        output.push_str("</div>\n");
    }

    // Examples
    if !type_doc.examples.is_empty() {
        output.push_str("<div class=\"examples\">\n");
        output.push_str("<strong>Examples:</strong>\n");
        for example in &type_doc.examples {
            output.push_str("<pre><code class=\"language-rhai\">");
            output.push_str(&escape_html(example));
            output.push_str("</code></pre>\n");
        }
        output.push_str("</div>\n");
    }

    output.push_str("</div>\n");
}

/// Generates HTML for a property.
pub fn generate_property_html(output: &mut String, prop: &PropertyDoc) {
    let readonly = if prop.readonly {
        " <span class=\"badge readonly\">read-only</span>"
    } else {
        ""
    };
    output.push_str(&format!(
        "<li><code>{}</code>: <code>{}</code>{} - {}</li>\n",
        escape_html(&prop.name),
        escape_html(&prop.type_name),
        readonly,
        escape_html(&prop.description)
    ));
}

/// Generates HTML for a method (compact format for type methods).
pub fn generate_method_html(output: &mut String, method: &FunctionDoc) {
    let signature = format_signature(&method.signature);
    output.push_str(&format!(
        "<li><code>{}</code> - {}</li>\n",
        escape_html(&signature),
        escape_html(&method.description)
    ));
}

/// Generates HTML for a function.
pub fn generate_function_html(output: &mut String, func: &FunctionDoc) {
    output.push_str("<div class=\"function-doc\">\n");

    // Function signature
    let params = func
        .signature
        .params
        .iter()
        .map(|(name, type_name)| format!("{}: {}", name, type_name))
        .collect::<Vec<_>>()
        .join(", ");

    let return_type = func
        .signature
        .return_type
        .as_ref()
        .map(|t| format!(" -&gt; {}", t))
        .unwrap_or_default();

    let signature = format!("{}({}){}", func.name, params, return_type);
    output.push_str(&format!(
        "<h4><code>{}</code></h4>\n",
        escape_html(&signature)
    ));

    // Deprecation warning
    if let Some(deprecated) = &func.deprecated {
        output.push_str(&format!(
            "<div class=\"deprecated\">⚠️ DEPRECATED: {}</div>\n",
            escape_html(deprecated)
        ));
    }

    // Description
    output.push_str(&format!("<p>{}</p>\n", escape_html(&func.description)));

    // Parameters
    if !func.parameters.is_empty() {
        output.push_str("<div class=\"parameters\">\n");
        output.push_str("<strong>Parameters:</strong>\n");
        output.push_str("<ul>\n");
        for param in &func.parameters {
            generate_param_html(output, param);
        }
        output.push_str("</ul>\n");
        output.push_str("</div>\n");
    }

    // Return value
    if let Some(returns) = &func.returns {
        output.push_str(&format!(
            "<div class=\"returns\"><strong>Returns:</strong> <code>{}</code> - {}</div>\n",
            escape_html(&returns.type_name),
            escape_html(&returns.description)
        ));
    }

    // Notes
    if let Some(notes) = &func.notes {
        output.push_str(&format!(
            "<div class=\"notes\"><strong>Notes:</strong> {}</div>\n",
            escape_html(notes)
        ));
    }

    // Examples
    if !func.examples.is_empty() {
        output.push_str("<div class=\"examples\">\n");
        output.push_str("<strong>Examples:</strong>\n");
        for example in &func.examples {
            output.push_str("<pre><code class=\"language-rhai\">");
            output.push_str(&escape_html(example));
            output.push_str("</code></pre>\n");
        }
        output.push_str("</div>\n");
    }

    // Version info
    if let Some(since) = &func.since {
        output.push_str(&format!(
            "<p class=\"version\"><em>Since: {}</em></p>\n",
            escape_html(since)
        ));
    }

    output.push_str("</div>\n");
}

/// Generates HTML for a parameter.
pub fn generate_param_html(output: &mut String, param: &ParamDoc) {
    let optional = if param.optional {
        " <span class=\"badge optional\">optional</span>"
    } else {
        ""
    };
    let default = if let Some(default) = &param.default {
        format!(" = <code>{}</code>", escape_html(default))
    } else {
        String::new()
    };

    output.push_str(&format!(
        "<li><code>{}</code>: <code>{}</code>{}{} - {}</li>\n",
        escape_html(&param.name),
        escape_html(&param.type_name),
        optional,
        default,
        escape_html(&param.description)
    ));
}

/// Formats a function signature for display.
pub fn format_signature(sig: &FunctionSignature) -> String {
    let params = sig
        .params
        .iter()
        .map(|(name, type_name)| format!("{}: {}", name, type_name))
        .collect::<Vec<_>>()
        .join(", ");

    let return_type = sig
        .return_type
        .as_ref()
        .map(|t| format!(" -> {}", t))
        .unwrap_or_default();

    format!("fn({}){}", params, return_type)
}

/// Escapes HTML special characters.
pub fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scripting::docs::FunctionSignature;

    #[test]
    fn test_escape_html() {
        assert_eq!(
            escape_html("<script>alert('xss')</script>"),
            "&lt;script&gt;alert(&#39;xss&#39;)&lt;/script&gt;"
        );
        assert_eq!(escape_html("a & b"), "a &amp; b");
        assert_eq!(escape_html("\"quoted\""), "&quot;quoted&quot;");
    }

    #[test]
    fn test_format_signature() {
        let sig = FunctionSignature {
            params: vec![
                ("key".to_string(), "KeyCode".to_string()),
                ("count".to_string(), "int".to_string()),
            ],
            return_type: Some("()".to_string()),
        };

        let formatted = format_signature(&sig);
        assert_eq!(formatted, "fn(key: KeyCode, count: int) -> ()");
    }

    #[test]
    fn test_badges_in_html() {
        let mut output = String::new();

        let param = ParamDoc {
            name: "timeout".to_string(),
            type_name: "int".to_string(),
            description: "Timeout in milliseconds".to_string(),
            optional: true,
            default: Some("1000".to_string()),
        };

        generate_param_html(&mut output, &param);

        assert!(output.contains("optional"));
        assert!(output.contains("badge"));
        assert!(output.contains("1000"));
    }
}
