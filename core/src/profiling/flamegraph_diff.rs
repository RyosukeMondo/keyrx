//! Differential flame graph generation for comparing performance profiles
//!
//! This module provides functionality to compare two sets of stack samples
//! and generate differential flame graphs that highlight performance
//! regressions and improvements.

use std::collections::HashMap;

use super::StackSample;
use crate::profiling::flamegraph::FlameGraphConfig;

/// Color scheme for differential visualization
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffColorScheme {
    /// Red for regressions (increased time), green for improvements
    RedGreen,
    /// Blue for regressions, orange for improvements
    BlueOrange,
    /// Grayscale with intensity showing magnitude of change
    Grayscale,
}

/// Configuration for differential flame graph generation
#[derive(Debug, Clone)]
pub struct DiffFlameGraphConfig {
    /// Width of the SVG output in pixels
    pub width: u32,
    /// Height of the SVG output in pixels
    pub height: u32,
    /// Color scheme to use for differential visualization
    pub color_scheme: DiffColorScheme,
    /// Title displayed at the top of the flame graph
    pub title: String,
    /// Height of each frame bar in pixels
    pub frame_height: u32,
    /// Minimum width in pixels for a frame to be displayed
    pub min_width: f64,
    /// Threshold for highlighting significant changes (as percentage)
    pub highlight_threshold: f64,
}

impl Default for DiffFlameGraphConfig {
    fn default() -> Self {
        Self {
            width: 1200,
            height: 800,
            color_scheme: DiffColorScheme::RedGreen,
            title: "Differential Flame Graph".to_string(),
            frame_height: 16,
            min_width: 0.1,
            highlight_threshold: 5.0, // 5% change
        }
    }
}

impl From<FlameGraphConfig> for DiffFlameGraphConfig {
    fn from(config: FlameGraphConfig) -> Self {
        Self {
            width: config.width,
            height: config.height,
            color_scheme: DiffColorScheme::RedGreen,
            title: format!("Diff: {}", config.title),
            frame_height: config.frame_height,
            min_width: config.min_width,
            highlight_threshold: 5.0,
        }
    }
}

/// Node in the differential flame graph tree structure
#[derive(Debug, Clone)]
struct DiffFlameNode {
    name: String,
    baseline_value: u64,
    current_value: u64,
    children: HashMap<String, DiffFlameNode>,
}

impl DiffFlameNode {
    fn new(name: String) -> Self {
        Self {
            name,
            baseline_value: 0,
            current_value: 0,
            children: HashMap::new(),
        }
    }

    fn add_baseline_stack(&mut self, frames: &[String]) {
        self.baseline_value += 1;

        if let Some((first, rest)) = frames.split_first() {
            let child = self
                .children
                .entry(first.clone())
                .or_insert_with(|| DiffFlameNode::new(first.clone()));
            child.add_baseline_stack(rest);
        }
    }

    fn add_current_stack(&mut self, frames: &[String]) {
        self.current_value += 1;

        if let Some((first, rest)) = frames.split_first() {
            let child = self
                .children
                .entry(first.clone())
                .or_insert_with(|| DiffFlameNode::new(first.clone()));
            child.add_current_stack(rest);
        }
    }

    /// Calculate the percentage change from baseline to current
    fn percent_change(&self) -> f64 {
        if self.baseline_value == 0 {
            if self.current_value > 0 {
                100.0 // New path
            } else {
                0.0
            }
        } else {
            let baseline = self.baseline_value as f64;
            let current = self.current_value as f64;
            ((current - baseline) / baseline) * 100.0
        }
    }

    /// Get the total value for percentage calculations
    fn total_value(&self) -> u64 {
        self.baseline_value.max(self.current_value)
    }
}

/// Generates differential flame graphs from baseline and current samples
pub struct DiffFlameGraphGenerator {
    config: DiffFlameGraphConfig,
}

impl Default for DiffFlameGraphGenerator {
    fn default() -> Self {
        Self::new(DiffFlameGraphConfig::default())
    }
}

impl DiffFlameGraphGenerator {
    /// Create a new differential flame graph generator
    pub fn new(config: DiffFlameGraphConfig) -> Self {
        Self { config }
    }

    /// Generate a differential SVG flame graph comparing baseline and current samples
    ///
    /// # Arguments
    ///
    /// * `baseline` - The baseline stack samples (e.g., from before optimization)
    /// * `current` - The current stack samples (e.g., from after optimization)
    ///
    /// # Returns
    ///
    /// An SVG string representing the differential flame graph where:
    /// - Red/warm colors indicate performance regressions (more time spent)
    /// - Green/cool colors indicate improvements (less time spent)
    /// - Gray indicates no significant change
    pub fn generate(&self, baseline: &[StackSample], current: &[StackSample]) -> String {
        if baseline.is_empty() && current.is_empty() {
            return self.generate_empty_svg("No samples collected");
        }

        if baseline.is_empty() {
            return self.generate_empty_svg("No baseline samples");
        }

        if current.is_empty() {
            return self.generate_empty_svg("No current samples");
        }

        // Build the differential flame tree
        let mut root = DiffFlameNode::new("all".to_string());

        for sample in baseline {
            let mut reversed_frames = sample.frames.clone();
            reversed_frames.reverse();
            root.add_baseline_stack(&reversed_frames);
        }

        for sample in current {
            let mut reversed_frames = sample.frames.clone();
            reversed_frames.reverse();
            root.add_current_stack(&reversed_frames);
        }

        // Generate SVG
        self.render_svg(&root)
    }

    /// Generate an empty SVG with a message
    fn generate_empty_svg(&self, message: &str) -> String {
        format!(
            r##"<?xml version="1.0" standalone="no"?>
<!DOCTYPE svg PUBLIC "-//W3C//DTD SVG 1.1//EN" "http://www.w3.org/Graphics/SVG/1.1/DTD/svg11.dtd">
<svg version="1.1" width="{}" height="{}" xmlns="http://www.w3.org/2000/svg">
  <text x="{}" y="{}" font-size="24" text-anchor="middle" fill="#999">{}</text>
</svg>"##,
            self.config.width,
            self.config.height,
            self.config.width / 2,
            self.config.height / 2,
            message
        )
    }

    /// Render the differential flame tree as SVG
    fn render_svg(&self, root: &DiffFlameNode) -> String {
        let mut svg = String::new();

        // SVG header
        svg.push_str(&format!(
            r##"<?xml version="1.0" standalone="no"?>
<!DOCTYPE svg PUBLIC "-//W3C//DTD SVG 1.1//EN" "http://www.w3.org/Graphics/SVG/1.1/DTD/svg11.dtd">
<svg version="1.1" width="{}" height="{}" xmlns="http://www.w3.org/2000/svg">
<defs>
  <linearGradient id="background" y1="0" y2="1" x1="0" x2="0">
    <stop stop-color="#eeeeee" offset="5%" />
    <stop stop-color="#eeeeb0" offset="95%" />
  </linearGradient>
</defs>
<style type="text/css">
  text {{ font-family: Verdana, sans-serif; font-size: 12px; fill: #000; }}
  .frame {{ cursor: pointer; }}
  .frame:hover {{ stroke: #000; stroke-width: 0.5; }}
  .title {{ font-size: 17px; font-weight: bold; }}
  .legend {{ font-size: 12px; }}
</style>
<rect x="0" y="0" width="100%" height="100%" fill="url(#background)" />
<text x="{}" y="24" class="title" text-anchor="middle">{}</text>
"##,
            self.config.width,
            self.config.height,
            self.config.width / 2,
            self.config.title
        ));

        // Add legend
        self.render_legend(&mut svg);

        // Render frames starting below the title and legend
        let y_offset = 70;
        let total_samples = root.total_value();
        self.render_frame(
            &mut svg,
            root,
            0.0,
            y_offset as f64,
            self.config.width as f64,
            total_samples,
            0,
        );

        // SVG footer
        svg.push_str("</svg>\n");

        svg
    }

    /// Render the color legend
    fn render_legend(&self, svg: &mut String) {
        let legend_y = 40;
        let legend_height = 15;
        let legend_width = 100;
        let x_center = self.config.width / 2;

        match self.config.color_scheme {
            DiffColorScheme::RedGreen => {
                // Regression (red)
                svg.push_str(&format!(
                    r##"<rect x="{}" y="{}" width="{}" height="{}" fill="rgb(220,50,50)" />
<text x="{}" y="{}" class="legend">Regression</text>
"##,
                    x_center - 250,
                    legend_y,
                    legend_width,
                    legend_height,
                    x_center - 140,
                    legend_y + 12
                ));

                // No change (gray)
                svg.push_str(&format!(
                    r##"<rect x="{}" y="{}" width="{}" height="{}" fill="rgb(180,180,180)" />
<text x="{}" y="{}" class="legend">No Change</text>
"##,
                    x_center - 50,
                    legend_y,
                    legend_width,
                    legend_height,
                    x_center + 60,
                    legend_y + 12
                ));

                // Improvement (green)
                svg.push_str(&format!(
                    r##"<rect x="{}" y="{}" width="{}" height="{}" fill="rgb(50,200,50)" />
<text x="{}" y="{}" class="legend">Improvement</text>
"##,
                    x_center + 150,
                    legend_y,
                    legend_width,
                    legend_height,
                    x_center + 260,
                    legend_y + 12
                ));
            }
            DiffColorScheme::BlueOrange => {
                svg.push_str(&format!(
                    r##"<rect x="{}" y="{}" width="{}" height="{}" fill="rgb(50,50,220)" />
<text x="{}" y="{}" class="legend">Regression</text>
<rect x="{}" y="{}" width="{}" height="{}" fill="rgb(180,180,180)" />
<text x="{}" y="{}" class="legend">No Change</text>
<rect x="{}" y="{}" width="{}" height="{}" fill="rgb(220,150,50)" />
<text x="{}" y="{}" class="legend">Improvement</text>
"##,
                    x_center - 250,
                    legend_y,
                    legend_width,
                    legend_height,
                    x_center - 140,
                    legend_y + 12,
                    x_center - 50,
                    legend_y,
                    legend_width,
                    legend_height,
                    x_center + 60,
                    legend_y + 12,
                    x_center + 150,
                    legend_y,
                    legend_width,
                    legend_height,
                    x_center + 260,
                    legend_y + 12
                ));
            }
            DiffColorScheme::Grayscale => {
                svg.push_str(&format!(
                    r##"<rect x="{}" y="{}" width="{}" height="{}" fill="rgb(50,50,50)" />
<text x="{}" y="{}" class="legend">High Change</text>
<rect x="{}" y="{}" width="{}" height="{}" fill="rgb(180,180,180)" />
<text x="{}" y="{}" class="legend">Low Change</text>
"##,
                    x_center - 150,
                    legend_y,
                    legend_width,
                    legend_height,
                    x_center - 40,
                    legend_y + 12,
                    x_center + 50,
                    legend_y,
                    legend_width,
                    legend_height,
                    x_center + 160,
                    legend_y + 12
                ));
            }
        }
    }

    /// Recursively render a frame and its children
    #[allow(clippy::too_many_arguments)]
    fn render_frame(
        &self,
        svg: &mut String,
        node: &DiffFlameNode,
        x: f64,
        y: f64,
        width: f64,
        total: u64,
        _depth: usize,
    ) {
        if width < self.config.min_width {
            return;
        }

        let height = self.config.frame_height as f64;
        let percent_change = node.percent_change();

        // Get color based on performance change
        let color = self.get_color(percent_change);

        // Escape text for XML
        let escaped_name = self.escape_xml(&node.name);

        // Build detailed title with statistics
        let title = if node.baseline_value > 0 && node.current_value > 0 {
            format!(
                "{} (baseline: {}, current: {}, change: {:.1}%)",
                escaped_name, node.baseline_value, node.current_value, percent_change
            )
        } else if node.baseline_value > 0 {
            format!(
                "{} (baseline: {}, removed)",
                escaped_name, node.baseline_value
            )
        } else {
            format!("{} (new, {} samples)", escaped_name, node.current_value)
        };

        // Truncate label if too long
        let label = if escaped_name.len() as f64 * 7.0 > width {
            let max_chars = (width / 7.0) as usize;
            if max_chars > 3 {
                format!("{}...", &escaped_name[..max_chars.saturating_sub(3)])
            } else {
                "".to_string()
            }
        } else {
            escaped_name
        };

        // Render the frame rectangle
        svg.push_str(&format!(
            r##"<g class="frame">
  <title>{}</title>
  <rect x="{:.1}" y="{:.1}" width="{:.1}" height="{:.1}" fill="{}" rx="2" ry="2" />
  <text x="{:.1}" y="{:.1}" text-anchor="middle">{}</text>
</g>
"##,
            title,
            x,
            y,
            width,
            height,
            color,
            x + width / 2.0,
            y + height / 2.0 + 4.0,
            label
        ));

        // Render children
        let mut child_x = x;
        let child_y = y + height;

        // Sort children by total value for consistent rendering
        let mut children: Vec<_> = node.children.values().collect();
        children.sort_by_key(|b| std::cmp::Reverse(b.total_value()));

        for child in children {
            let child_width = (child.total_value() as f64 / total as f64) * width;
            self.render_frame(svg, child, child_x, child_y, child_width, total, _depth + 1);
            child_x += child_width;
        }
    }

    /// Get a color based on the performance change percentage
    fn get_color(&self, percent_change: f64) -> String {
        match self.config.color_scheme {
            DiffColorScheme::RedGreen => self.red_green_color(percent_change),
            DiffColorScheme::BlueOrange => self.blue_orange_color(percent_change),
            DiffColorScheme::Grayscale => self.grayscale_diff_color(percent_change),
        }
    }

    /// Generate red-green differential color
    fn red_green_color(&self, percent_change: f64) -> String {
        let abs_change = percent_change.abs();

        if abs_change < self.config.highlight_threshold {
            // No significant change - gray
            return "rgb(180,180,180)".to_string();
        }

        if percent_change > 0.0 {
            // Regression - red (more time spent)
            let intensity = (abs_change / 100.0).min(1.0);
            let r = 150 + (intensity * 70.0) as u8;
            let g = (150.0 * (1.0 - intensity)) as u8;
            let b = (150.0 * (1.0 - intensity)) as u8;
            format!("rgb({},{},{})", r, g, b)
        } else {
            // Improvement - green (less time spent)
            let intensity = (abs_change / 100.0).min(1.0);
            let r = (150.0 * (1.0 - intensity)) as u8;
            let g = 150 + (intensity * 70.0) as u8;
            let b = (150.0 * (1.0 - intensity)) as u8;
            format!("rgb({},{},{})", r, g, b)
        }
    }

    /// Generate blue-orange differential color
    fn blue_orange_color(&self, percent_change: f64) -> String {
        let abs_change = percent_change.abs();

        if abs_change < self.config.highlight_threshold {
            return "rgb(180,180,180)".to_string();
        }

        if percent_change > 0.0 {
            // Regression - blue
            let intensity = (abs_change / 100.0).min(1.0);
            let r = (150.0 * (1.0 - intensity)) as u8;
            let g = (150.0 * (1.0 - intensity)) as u8;
            let b = 150 + (intensity * 70.0) as u8;
            format!("rgb({},{},{})", r, g, b)
        } else {
            // Improvement - orange
            let intensity = (abs_change / 100.0).min(1.0);
            let r = 150 + (intensity * 70.0) as u8;
            let g = 100 + (intensity * 50.0) as u8;
            let b = (150.0 * (1.0 - intensity)) as u8;
            format!("rgb({},{},{})", r, g, b)
        }
    }

    /// Generate grayscale differential color based on magnitude
    fn grayscale_diff_color(&self, percent_change: f64) -> String {
        let abs_change = percent_change.abs();

        if abs_change < self.config.highlight_threshold {
            return "rgb(200,200,200)".to_string();
        }

        let intensity = (abs_change / 100.0).min(1.0);
        let gray = (200.0 * (1.0 - intensity * 0.7)) as u8;
        format!("rgb({},{},{})", gray, gray, gray)
    }

    /// Escape XML special characters
    fn escape_xml(&self, s: &str) -> String {
        s.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&apos;")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn create_baseline_samples() -> Vec<StackSample> {
        vec![
            StackSample {
                timestamp: Duration::from_millis(0),
                frames: vec![
                    "main".to_string(),
                    "process".to_string(),
                    "slow_function".to_string(),
                ],
            },
            StackSample {
                timestamp: Duration::from_millis(10),
                frames: vec![
                    "main".to_string(),
                    "process".to_string(),
                    "slow_function".to_string(),
                ],
            },
            StackSample {
                timestamp: Duration::from_millis(20),
                frames: vec![
                    "main".to_string(),
                    "process".to_string(),
                    "fast_function".to_string(),
                ],
            },
        ]
    }

    fn create_improved_samples() -> Vec<StackSample> {
        vec![
            StackSample {
                timestamp: Duration::from_millis(0),
                frames: vec![
                    "main".to_string(),
                    "process".to_string(),
                    "slow_function".to_string(),
                ],
            },
            StackSample {
                timestamp: Duration::from_millis(10),
                frames: vec![
                    "main".to_string(),
                    "process".to_string(),
                    "fast_function".to_string(),
                ],
            },
            StackSample {
                timestamp: Duration::from_millis(20),
                frames: vec![
                    "main".to_string(),
                    "process".to_string(),
                    "fast_function".to_string(),
                ],
            },
        ]
    }

    fn create_regressed_samples() -> Vec<StackSample> {
        vec![
            StackSample {
                timestamp: Duration::from_millis(0),
                frames: vec![
                    "main".to_string(),
                    "process".to_string(),
                    "slow_function".to_string(),
                ],
            },
            StackSample {
                timestamp: Duration::from_millis(10),
                frames: vec![
                    "main".to_string(),
                    "process".to_string(),
                    "slow_function".to_string(),
                ],
            },
            StackSample {
                timestamp: Duration::from_millis(20),
                frames: vec![
                    "main".to_string(),
                    "process".to_string(),
                    "slow_function".to_string(),
                ],
            },
        ]
    }

    #[test]
    fn test_diff_flame_graph_generation() {
        let generator = DiffFlameGraphGenerator::default();
        let baseline = create_baseline_samples();
        let current = create_improved_samples();
        let svg = generator.generate(&baseline, &current);

        assert!(svg.contains("<?xml"));
        assert!(svg.contains("<svg"));
        assert!(svg.contains("</svg>"));
        assert!(svg.contains("Differential Flame Graph"));
    }

    #[test]
    fn test_empty_baseline() {
        let generator = DiffFlameGraphGenerator::default();
        let svg = generator.generate(&[], &create_improved_samples());

        assert!(svg.contains("No baseline samples"));
    }

    #[test]
    fn test_empty_current() {
        let generator = DiffFlameGraphGenerator::default();
        let svg = generator.generate(&create_baseline_samples(), &[]);

        assert!(svg.contains("No current samples"));
    }

    #[test]
    fn test_both_empty() {
        let generator = DiffFlameGraphGenerator::default();
        let svg = generator.generate(&[], &[]);

        assert!(svg.contains("No samples collected"));
    }

    #[test]
    fn test_improvement_detection() {
        let generator = DiffFlameGraphGenerator::default();
        let baseline = create_baseline_samples();
        let improved = create_improved_samples();
        let svg = generator.generate(&baseline, &improved);

        // Should contain frames from both baseline and current
        assert!(svg.contains("main"));
        assert!(svg.contains("process"));
        assert!(svg.contains("slow_function"));
        assert!(svg.contains("fast_function"));

        // Should have colors indicating change
        assert!(svg.contains("rgb("));
    }

    #[test]
    fn test_regression_detection() {
        let generator = DiffFlameGraphGenerator::default();
        let baseline = create_baseline_samples();
        let regressed = create_regressed_samples();
        let svg = generator.generate(&baseline, &regressed);

        assert!(svg.contains("main"));
        assert!(svg.contains("process"));
        assert!(svg.contains("slow_function"));
    }

    #[test]
    fn test_diff_color_schemes() {
        let baseline = create_baseline_samples();
        let current = create_improved_samples();

        for scheme in [
            DiffColorScheme::RedGreen,
            DiffColorScheme::BlueOrange,
            DiffColorScheme::Grayscale,
        ] {
            let config = DiffFlameGraphConfig {
                color_scheme: scheme,
                ..Default::default()
            };
            let generator = DiffFlameGraphGenerator::new(config);
            let svg = generator.generate(&baseline, &current);

            assert!(svg.contains("rgb("));
            assert!(svg.contains("legend"));
        }
    }

    #[test]
    fn test_custom_config() {
        let config = DiffFlameGraphConfig {
            width: 800,
            height: 600,
            color_scheme: DiffColorScheme::BlueOrange,
            title: "Test Diff".to_string(),
            highlight_threshold: 10.0,
            ..Default::default()
        };

        let generator = DiffFlameGraphGenerator::new(config);
        let baseline = create_baseline_samples();
        let current = create_improved_samples();
        let svg = generator.generate(&baseline, &current);

        assert!(svg.contains("width=\"800\""));
        assert!(svg.contains("height=\"600\""));
        assert!(svg.contains("Test Diff"));
    }

    #[test]
    fn test_percent_change_calculation() {
        let mut node = DiffFlameNode::new("test".to_string());

        node.baseline_value = 100;
        node.current_value = 150;
        assert_eq!(node.percent_change(), 50.0);

        node.baseline_value = 100;
        node.current_value = 50;
        assert_eq!(node.percent_change(), -50.0);

        node.baseline_value = 0;
        node.current_value = 100;
        assert_eq!(node.percent_change(), 100.0);

        node.baseline_value = 0;
        node.current_value = 0;
        assert_eq!(node.percent_change(), 0.0);
    }

    #[test]
    fn test_xml_escaping() {
        let generator = DiffFlameGraphGenerator::default();
        let baseline = vec![StackSample {
            timestamp: Duration::from_millis(0),
            frames: vec!["<script>".to_string()],
        }];
        let current = vec![StackSample {
            timestamp: Duration::from_millis(0),
            frames: vec!["<script>".to_string()],
        }];

        let svg = generator.generate(&baseline, &current);

        assert!(!svg.contains("<script>"));
        assert!(svg.contains("&lt;script&gt;"));
    }

    #[test]
    fn test_diff_node_building() {
        let mut root = DiffFlameNode::new("root".to_string());

        let frames = vec!["main".to_string(), "foo".to_string()];
        root.add_baseline_stack(&frames);
        root.add_baseline_stack(&frames);
        root.add_current_stack(&frames);

        assert_eq!(root.baseline_value, 2);
        assert_eq!(root.current_value, 1);
        assert_eq!(root.children.len(), 1);

        let main_node = &root.children["main"];
        assert_eq!(main_node.baseline_value, 2);
        assert_eq!(main_node.current_value, 1);
    }

    #[test]
    fn test_new_path_detection() {
        let mut root = DiffFlameNode::new("root".to_string());

        let baseline_frames = vec!["main".to_string(), "old_function".to_string()];
        root.add_baseline_stack(&baseline_frames);

        let current_frames = vec!["main".to_string(), "new_function".to_string()];
        root.add_current_stack(&current_frames);

        assert_eq!(root.children.len(), 1);
        let main_node = &root.children["main"];
        assert_eq!(main_node.children.len(), 2);
        assert!(main_node.children.contains_key("old_function"));
        assert!(main_node.children.contains_key("new_function"));
    }

    #[test]
    fn test_config_from_flamegraph_config() {
        use crate::profiling::flamegraph::{ColorScheme, FlameGraphConfig};

        let flame_config = FlameGraphConfig {
            width: 1000,
            height: 700,
            color_scheme: ColorScheme::Hot,
            title: "Original".to_string(),
            frame_height: 20,
            min_width: 0.5,
        };

        let diff_config = DiffFlameGraphConfig::from(flame_config);

        assert_eq!(diff_config.width, 1000);
        assert_eq!(diff_config.height, 700);
        assert_eq!(diff_config.frame_height, 20);
        assert_eq!(diff_config.min_width, 0.5);
        assert_eq!(diff_config.title, "Diff: Original");
    }
}
