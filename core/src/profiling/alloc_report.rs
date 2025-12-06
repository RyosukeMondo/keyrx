//! Allocation reporting and analysis
//!
//! This module provides comprehensive memory allocation reporting with
//! hot spot detection, threshold warnings, and JSON serialization.

use serde::{Deserialize, Serialize};

use crate::error::KeyRxError;
use crate::profiling::allocations::{AllocationSite, AllocationStats};

/// Configuration for allocation report generation
#[derive(Debug, Clone)]
pub struct AllocationReportConfig {
    /// Minimum bytes for a site to be considered a hot spot
    pub hot_spot_threshold: usize,
    /// Maximum number of hot spots to include in report
    pub max_hot_spots: usize,
    /// Whether to include full stack traces
    pub include_stack_traces: bool,
    /// Threshold for generating warnings (bytes)
    pub warning_threshold: Option<usize>,
}

impl Default for AllocationReportConfig {
    fn default() -> Self {
        Self {
            hot_spot_threshold: 1024 * 1024, // 1MB
            max_hot_spots: 20,
            include_stack_traces: true,
            warning_threshold: Some(100 * 1024 * 1024), // 100MB
        }
    }
}

/// Complete allocation report with statistics and hot spots
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationReport {
    /// Overall allocation statistics
    pub stats: SerializableAllocationStats,
    /// Hot spot allocation sites
    pub hot_spots: Vec<SerializableAllocationSite>,
    /// Warnings generated during analysis
    pub warnings: Vec<String>,
    /// Summary metrics
    pub summary: ReportSummary,
}

/// Serializable version of AllocationStats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableAllocationStats {
    pub total_allocated: usize,
    pub total_freed: usize,
    pub peak_usage: usize,
    pub current_usage: usize,
    pub allocation_count: u64,
    pub free_count: u64,
}

impl From<AllocationStats> for SerializableAllocationStats {
    fn from(stats: AllocationStats) -> Self {
        Self {
            total_allocated: stats.total_allocated,
            total_freed: stats.total_freed,
            peak_usage: stats.peak_usage,
            current_usage: stats.current_usage,
            allocation_count: stats.allocation_count,
            free_count: stats.free_count,
        }
    }
}

/// Serializable version of AllocationSite
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableAllocationSite {
    pub location: String,
    pub count: u64,
    pub total_bytes: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stack_trace: Option<Vec<String>>,
    pub percentage: f64,
}

/// Summary metrics for the report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportSummary {
    /// Average allocation size
    pub avg_allocation_size: usize,
    /// Memory efficiency (freed / allocated)
    pub memory_efficiency: f64,
    /// Number of hot spots detected
    pub hot_spot_count: usize,
    /// Largest single allocation site
    pub largest_site_bytes: usize,
}

/// Generator for allocation reports
pub struct AllocationReportGenerator {
    config: AllocationReportConfig,
}

impl AllocationReportGenerator {
    /// Create a new report generator with the given configuration
    pub fn new(config: AllocationReportConfig) -> Self {
        Self { config }
    }

    /// Generate a comprehensive allocation report
    pub fn generate(
        &self,
        stats: AllocationStats,
        sites: Vec<AllocationSite>,
    ) -> Result<AllocationReport, KeyRxError> {
        // Convert stats
        let serializable_stats = SerializableAllocationStats::from(stats);

        // Identify hot spots
        let hot_spots = self.identify_hot_spots(&sites, serializable_stats.total_allocated);

        // Generate warnings
        let warnings = self.generate_warnings(&serializable_stats, &hot_spots);

        // Calculate summary
        let summary = self.calculate_summary(&serializable_stats, &hot_spots);

        Ok(AllocationReport {
            stats: serializable_stats,
            hot_spots,
            warnings,
            summary,
        })
    }

    /// Generate JSON representation of the report
    pub fn to_json(&self, report: &AllocationReport) -> Result<String, KeyRxError> {
        serde_json::to_string_pretty(report)
            .map_err(|e| KeyRxError::platform(format!("Failed to serialize report: {}", e)))
    }

    /// Identify hot spot allocation sites
    fn identify_hot_spots(
        &self,
        sites: &[AllocationSite],
        total_allocated: usize,
    ) -> Vec<SerializableAllocationSite> {
        let mut hot_spots: Vec<_> = sites
            .iter()
            .filter(|site| site.total_bytes >= self.config.hot_spot_threshold)
            .map(|site| SerializableAllocationSite {
                location: site.location.clone(),
                count: site.count,
                total_bytes: site.total_bytes,
                stack_trace: if self.config.include_stack_traces {
                    Some(site.stack_trace.clone())
                } else {
                    None
                },
                percentage: if total_allocated > 0 {
                    (site.total_bytes as f64 / total_allocated as f64) * 100.0
                } else {
                    0.0
                },
            })
            .collect();

        // Sort by total bytes (descending)
        hot_spots.sort_by(|a, b| b.total_bytes.cmp(&a.total_bytes));

        // Limit to max hot spots
        hot_spots.truncate(self.config.max_hot_spots);

        hot_spots
    }

    /// Generate warnings based on thresholds
    fn generate_warnings(
        &self,
        stats: &SerializableAllocationStats,
        hot_spots: &[SerializableAllocationSite],
    ) -> Vec<String> {
        let mut warnings = Vec::new();

        // Check total allocation threshold
        if let Some(threshold) = self.config.warning_threshold {
            if stats.total_allocated > threshold {
                warnings.push(format!(
                    "Total allocations ({} bytes) exceed threshold ({} bytes)",
                    format_bytes(stats.total_allocated),
                    format_bytes(threshold)
                ));
            }

            if stats.peak_usage > threshold {
                warnings.push(format!(
                    "Peak memory usage ({} bytes) exceed threshold ({} bytes)",
                    format_bytes(stats.peak_usage),
                    format_bytes(threshold)
                ));
            }
        }

        // Check for memory leaks
        let leaked = stats.total_allocated.saturating_sub(stats.total_freed);
        if leaked > 0 && stats.current_usage > 0 {
            let leak_percentage = (leaked as f64 / stats.total_allocated as f64) * 100.0;
            if leak_percentage > 10.0 {
                warnings.push(format!(
                    "Potential memory leak detected: {} bytes ({:.1}% of allocations) not freed",
                    format_bytes(leaked),
                    leak_percentage
                ));
            }
        }

        // Check for excessive hot spots
        if hot_spots.len() > 10 {
            warnings.push(format!(
                "Large number of hot spots ({}) detected - consider optimizing allocation patterns",
                hot_spots.len()
            ));
        }

        // Check for dominant hot spots
        for hot_spot in hot_spots.iter().take(3) {
            if hot_spot.percentage > 25.0 {
                warnings.push(format!(
                    "Hot spot at {} accounts for {:.1}% of total allocations",
                    hot_spot.location, hot_spot.percentage
                ));
            }
        }

        warnings
    }

    /// Calculate summary metrics
    fn calculate_summary(
        &self,
        stats: &SerializableAllocationStats,
        hot_spots: &[SerializableAllocationSite],
    ) -> ReportSummary {
        let avg_allocation_size = if stats.allocation_count > 0 {
            stats.total_allocated / stats.allocation_count as usize
        } else {
            0
        };

        let memory_efficiency = if stats.total_allocated > 0 {
            stats.total_freed as f64 / stats.total_allocated as f64
        } else {
            0.0
        };

        let largest_site_bytes = hot_spots.first().map(|site| site.total_bytes).unwrap_or(0);

        ReportSummary {
            avg_allocation_size,
            memory_efficiency,
            hot_spot_count: hot_spots.len(),
            largest_site_bytes,
        }
    }
}

impl Default for AllocationReportGenerator {
    fn default() -> Self {
        Self::new(AllocationReportConfig::default())
    }
}

/// Format bytes as human-readable string
fn format_bytes(bytes: usize) -> String {
    const KB: usize = 1024;
    const MB: usize = KB * 1024;
    const GB: usize = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} bytes", bytes)
    }
}

/// Analyze allocation patterns and provide recommendations
pub struct AllocationAnalyzer;

impl AllocationAnalyzer {
    /// Analyze a report and generate recommendations
    pub fn analyze(report: &AllocationReport) -> Vec<String> {
        let mut recommendations = Vec::new();

        // Check allocation/deallocation ratio
        if report.stats.allocation_count > 0 {
            let alloc_free_ratio =
                report.stats.free_count as f64 / report.stats.allocation_count as f64;
            if alloc_free_ratio < 0.9 {
                recommendations.push(
                    "Consider reviewing object lifetimes - low deallocation ratio detected"
                        .to_string(),
                );
            }
        }

        // Check for fragmentation indicators
        if report.summary.avg_allocation_size < 256 {
            recommendations.push(
                "Small average allocation size may indicate fragmentation - consider object pooling"
                    .to_string(),
            );
        }

        // Check hot spot concentration
        let total_hot_spot_bytes: usize = report.hot_spots.iter().map(|s| s.total_bytes).sum();
        if total_hot_spot_bytes > 0 {
            let concentration = total_hot_spot_bytes as f64 / report.stats.total_allocated as f64;
            if concentration > 0.8 {
                recommendations.push(format!(
                    "Hot spots account for {:.1}% of allocations - focus optimization efforts here",
                    concentration * 100.0
                ));
            }
        }

        // Check for excessive allocations
        if report.stats.allocation_count > 1_000_000 {
            recommendations.push(
                "Very high allocation count - consider reusing objects or using arena allocation"
                    .to_string(),
            );
        }

        recommendations
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_stats() -> AllocationStats {
        AllocationStats {
            total_allocated: 10_000_000,
            total_freed: 8_000_000,
            peak_usage: 5_000_000,
            current_usage: 2_000_000,
            allocation_count: 1000,
            free_count: 800,
        }
    }

    fn create_test_sites() -> Vec<AllocationSite> {
        vec![
            AllocationSite {
                location: "test.rs:10".to_string(),
                count: 100,
                total_bytes: 5_000_000,
                stack_trace: vec!["test.rs:10".to_string()],
            },
            AllocationSite {
                location: "test.rs:20".to_string(),
                count: 200,
                total_bytes: 3_000_000,
                stack_trace: vec!["test.rs:20".to_string()],
            },
            AllocationSite {
                location: "test.rs:30".to_string(),
                count: 300,
                total_bytes: 500_000,
                stack_trace: vec!["test.rs:30".to_string()],
            },
        ]
    }

    #[test]
    fn test_generate_report() {
        let generator = AllocationReportGenerator::default();
        let stats = create_test_stats();
        let sites = create_test_sites();

        let report = generator
            .generate(stats, sites)
            .expect("Failed to generate report");

        assert_eq!(report.stats.total_allocated, 10_000_000);
        assert_eq!(report.stats.total_freed, 8_000_000);
        assert!(!report.hot_spots.is_empty());
    }

    #[test]
    fn test_hot_spot_detection() {
        let config = AllocationReportConfig {
            hot_spot_threshold: 1_000_000,
            max_hot_spots: 10,
            include_stack_traces: true,
            warning_threshold: None,
        };
        let generator = AllocationReportGenerator::new(config);
        let stats = create_test_stats();
        let sites = create_test_sites();

        let report = generator
            .generate(stats, sites)
            .expect("Failed to generate report");

        // Should detect 2 hot spots (5MB and 3MB, but not 500KB)
        assert_eq!(report.hot_spots.len(), 2);
        assert_eq!(report.hot_spots[0].total_bytes, 5_000_000);
        assert_eq!(report.hot_spots[1].total_bytes, 3_000_000);
    }

    #[test]
    fn test_hot_spot_sorting() {
        let generator = AllocationReportGenerator::default();
        let stats = create_test_stats();
        let sites = create_test_sites();

        let report = generator
            .generate(stats, sites)
            .expect("Failed to generate report");

        // Hot spots should be sorted by size (descending)
        for i in 0..report.hot_spots.len().saturating_sub(1) {
            assert!(report.hot_spots[i].total_bytes >= report.hot_spots[i + 1].total_bytes);
        }
    }

    #[test]
    fn test_percentage_calculation() {
        let generator = AllocationReportGenerator::default();
        let stats = create_test_stats();
        let sites = create_test_sites();
        let total_allocated = stats.total_allocated;

        let report = generator
            .generate(stats, sites)
            .expect("Failed to generate report");

        // Check percentage calculation for largest hot spot
        if let Some(hot_spot) = report.hot_spots.first() {
            let expected_percentage =
                (hot_spot.total_bytes as f64 / total_allocated as f64) * 100.0;
            assert!((hot_spot.percentage - expected_percentage).abs() < 0.01);
        }
    }

    #[test]
    fn test_warning_generation_threshold() {
        let config = AllocationReportConfig {
            hot_spot_threshold: 1_000_000,
            max_hot_spots: 10,
            include_stack_traces: true,
            warning_threshold: Some(5_000_000),
        };
        let generator = AllocationReportGenerator::new(config);
        let stats = create_test_stats();
        let sites = create_test_sites();

        let report = generator
            .generate(stats, sites)
            .expect("Failed to generate report");

        // Should have warning about exceeding threshold
        assert!(!report.warnings.is_empty());
        assert!(report
            .warnings
            .iter()
            .any(|w| w.contains("exceed threshold")));
    }

    #[test]
    fn test_summary_calculation() {
        let generator = AllocationReportGenerator::default();
        let stats = create_test_stats();
        let sites = create_test_sites();

        let report = generator
            .generate(stats, sites)
            .expect("Failed to generate report");

        // Check summary metrics
        assert_eq!(report.summary.avg_allocation_size, 10_000);
        assert!((report.summary.memory_efficiency - 0.8).abs() < 0.01);
        assert_eq!(report.summary.hot_spot_count, report.hot_spots.len());
    }

    #[test]
    fn test_json_serialization() {
        let generator = AllocationReportGenerator::default();
        let stats = create_test_stats();
        let sites = create_test_sites();

        let report = generator
            .generate(stats, sites)
            .expect("Failed to generate report");
        let json = generator
            .to_json(&report)
            .expect("Failed to serialize to JSON");

        // Should be valid JSON
        assert!(json.contains("\"stats\""));
        assert!(json.contains("\"hot_spots\""));
        assert!(json.contains("\"warnings\""));
        assert!(json.contains("\"summary\""));
    }

    #[test]
    fn test_stack_trace_inclusion() {
        let config = AllocationReportConfig {
            hot_spot_threshold: 1_000_000,
            max_hot_spots: 10,
            include_stack_traces: false,
            warning_threshold: None,
        };
        let generator = AllocationReportGenerator::new(config);
        let stats = create_test_stats();
        let sites = create_test_sites();

        let report = generator
            .generate(stats, sites)
            .expect("Failed to generate report");

        // Stack traces should be omitted
        for hot_spot in &report.hot_spots {
            assert!(hot_spot.stack_trace.is_none());
        }
    }

    #[test]
    fn test_max_hot_spots_limit() {
        let config = AllocationReportConfig {
            hot_spot_threshold: 0, // All sites are hot spots
            max_hot_spots: 2,
            include_stack_traces: true,
            warning_threshold: None,
        };
        let generator = AllocationReportGenerator::new(config);
        let stats = create_test_stats();
        let sites = create_test_sites();

        let report = generator
            .generate(stats, sites)
            .expect("Failed to generate report");

        // Should limit to 2 hot spots
        assert_eq!(report.hot_spots.len(), 2);
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(500), "500 bytes");
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1024 * 1024), "1.00 MB");
        assert_eq!(format_bytes(1024 * 1024 * 1024), "1.00 GB");
        assert_eq!(format_bytes(1536), "1.50 KB");
    }

    #[test]
    fn test_analyzer_recommendations() {
        let generator = AllocationReportGenerator::default();
        let stats = create_test_stats();
        let sites = create_test_sites();

        let report = generator
            .generate(stats, sites)
            .expect("Failed to generate report");
        let recommendations = AllocationAnalyzer::analyze(&report);

        assert!(!recommendations.is_empty());
    }

    #[test]
    fn test_analyzer_low_deallocation_ratio() {
        let stats = AllocationStats {
            total_allocated: 10_000_000,
            total_freed: 5_000_000, // Only 50% freed
            peak_usage: 5_000_000,
            current_usage: 5_000_000,
            allocation_count: 1000,
            free_count: 500,
        };

        let generator = AllocationReportGenerator::default();
        let report = generator
            .generate(stats, vec![])
            .expect("Failed to generate report");
        let recommendations = AllocationAnalyzer::analyze(&report);

        assert!(recommendations
            .iter()
            .any(|r| r.contains("object lifetimes")));
    }

    #[test]
    fn test_analyzer_small_allocations() {
        let stats = AllocationStats {
            total_allocated: 100_000,
            total_freed: 90_000,
            peak_usage: 10_000,
            current_usage: 10_000,
            allocation_count: 1000, // Average 100 bytes per allocation
            free_count: 900,
        };

        let generator = AllocationReportGenerator::default();
        let report = generator
            .generate(stats, vec![])
            .expect("Failed to generate report");
        let recommendations = AllocationAnalyzer::analyze(&report);

        assert!(recommendations.iter().any(|r| r.contains("fragmentation")));
    }

    #[test]
    fn test_default_config() {
        let config = AllocationReportConfig::default();
        assert_eq!(config.hot_spot_threshold, 1024 * 1024);
        assert_eq!(config.max_hot_spots, 20);
        assert!(config.include_stack_traces);
        assert_eq!(config.warning_threshold, Some(100 * 1024 * 1024));
    }

    #[test]
    fn test_zero_allocations() {
        let stats = AllocationStats {
            total_allocated: 0,
            total_freed: 0,
            peak_usage: 0,
            current_usage: 0,
            allocation_count: 0,
            free_count: 0,
        };

        let generator = AllocationReportGenerator::default();
        let report = generator
            .generate(stats, vec![])
            .expect("Failed to generate report");

        assert_eq!(report.summary.avg_allocation_size, 0);
        assert_eq!(report.summary.memory_efficiency, 0.0);
        assert_eq!(report.hot_spots.len(), 0);
    }
}
