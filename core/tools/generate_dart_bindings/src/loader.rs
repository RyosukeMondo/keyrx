//! Contract loader for Dart binding generation
//!
//! Loads FFI contracts from a directory, optionally filtering by domain.

use anyhow::{Context, Result};
use keyrx_core::ffi::contract::{ContractRegistry, FfiContract};
use std::path::Path;

/// Load all contracts from a directory
pub fn load_contracts(contracts_dir: &Path) -> Result<ContractRegistry> {
    ContractRegistry::load_from_dir(contracts_dir)
        .with_context(|| format!("Failed to load contracts from {:?}", contracts_dir))
}

/// Load contracts filtered by domain
pub fn load_contracts_for_domain(
    contracts_dir: &Path,
    domain: &str,
) -> Result<Vec<FfiContract>> {
    let registry = load_contracts(contracts_dir)?;

    let contracts: Vec<FfiContract> = registry
        .all_contracts()
        .values()
        .filter(|c| c.domain == domain)
        .cloned()
        .collect();

    if contracts.is_empty() {
        anyhow::bail!("No contract found for domain: {}", domain);
    }

    Ok(contracts)
}

/// Load all contracts as a vector
pub fn load_all_contracts(contracts_dir: &Path) -> Result<Vec<FfiContract>> {
    let registry = load_contracts(contracts_dir)?;

    let contracts: Vec<FfiContract> = registry.all_contracts().values().cloned().collect();

    if contracts.is_empty() {
        anyhow::bail!("No contracts found in {:?}", contracts_dir);
    }

    Ok(contracts)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn contracts_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("src/ffi/contracts")
    }

    #[test]
    fn test_load_contracts() {
        let registry = load_contracts(&contracts_dir()).expect("Failed to load contracts");
        assert!(!registry.domains().is_empty());
    }

    #[test]
    fn test_load_contracts_for_domain() {
        let contracts =
            load_contracts_for_domain(&contracts_dir(), "config").expect("Failed to load config");
        assert_eq!(contracts.len(), 1);
        assert_eq!(contracts[0].domain, "config");
    }

    #[test]
    fn test_load_contracts_for_missing_domain() {
        let result = load_contracts_for_domain(&contracts_dir(), "nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_load_all_contracts() {
        let contracts = load_all_contracts(&contracts_dir()).expect("Failed to load all contracts");
        assert!(contracts.len() >= 4); // We have at least config, engine, runtime, discovery
    }
}
