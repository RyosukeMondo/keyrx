//! Generates Dart IPC client code from FFI contracts.

use crate::templates::{
    context, render, to_pascal_case, IPC_CLIENT_CLASS_TEMPLATE, IPC_CLIENT_FILE_HEADER,
};
use anyhow::Result;
use chrono;
use keyrx_core::ffi::contract::FfiContract;

pub struct IpcClientGenerator {
    // Generation state, if any
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
    let methods = String::new(); // Placeholder
    let events = String::new(); // Placeholder

    let mut ctx = context();
    ctx.insert("class_name".to_string(), class_name);
    ctx.insert("domain".to_string(), domain);
    ctx.insert("methods".to_string(), methods);
    ctx.insert("events".to_string(), events);

    Ok(render(IPC_CLIENT_CLASS_TEMPLATE, &ctx))
}
