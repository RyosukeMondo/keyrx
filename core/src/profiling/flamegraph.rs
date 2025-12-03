//! Flame graph generation for visualizing profiling data
//!
//! This module converts stack samples into interactive SVG flame graphs
//! that help identify performance bottlenecks.

use std::collections::HashMap;

use super::StackSample;

/// Color scheme for flame graph visualization
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorScheme {
    /// Hot colors (red/orange/yellow) for CPU-intensive operations
    Hot,
    /// Cool colors (blue/green) for I/O operations
    Cool,
    /// Rainbow spectrum for general visualization
    Rainbow,
    /// Grayscale for printing or accessibility
    Grayscale,
}

/// Configuration for flame graph generation
#[derive(Debug, Clone)]
pub struct FlameGraphConfig {
    /// Width of the SVG output in pixels
    pub width: u32,
    /// Height of the SVG output in pixels
    pub height: u32,
    /// Color scheme to use
    pub color_scheme: ColorScheme,
    /// Title displayed at the top of the flame graph
    pub title: String,
    /// Height of each frame bar in pixels
    pub frame_height: u32,
    /// Minimum width in pixels for a frame to be displayed
    pub min_width: f64,
}

impl Default for FlameGraphConfig {
    fn default() -> Self {
        Self {
            width: 1200,
            height: 800,
            color_scheme: ColorScheme::Hot,
            title: "Flame Graph".to_string(),
            frame_height: 16,
            min_width: 0.1,
        }
    }
}

/// Node in the flame graph tree structure
#[derive(Debug, Clone)]
struct FlameNode {
    name: String,
    value: u64,
    children: HashMap<String, FlameNode>,
}

impl FlameNode {
    fn new(name: String) -> Self {
        Self {
            name,
            value: 0,
            children: HashMap::new(),
        }
    }

    fn add_stack(&mut self, frames: &[String]) {
        self.value += 1;

        if let Some((first, rest)) = frames.split_first() {
            let child = self
                .children
                .entry(first.clone())
                .or_insert_with(|| FlameNode::new(first.clone()));
            child.add_stack(rest);
        }
    }
}

/// Generates flame graph visualizations from stack samples
pub struct FlameGraphGenerator {
    config: FlameGraphConfig,
}

impl Default for FlameGraphGenerator {
    fn default() -> Self {
        Self::new(FlameGraphConfig::default())
    }
}

impl FlameGraphGenerator {
    /// Create a new flame graph generator with the given configuration
    pub fn new(config: FlameGraphConfig) -> Self {
        Self { config }
    }

    /// Generate an SVG flame graph from stack samples
    ///
    /// # Arguments
    ///
    /// * `stacks` - The stack samples to visualize
    ///
    /// # Returns
    ///
    /// An SVG string representing the flame graph
    pub fn generate(&self, stacks: &[StackSample]) -> String {
        if stacks.is_empty() {
            return self.generate_empty_svg("No samples collected");
        }

        // Build the flame tree
        let mut root = FlameNode::new("all".to_string());
        for sample in stacks {
            // Reverse frames to build from root to leaf
            let mut reversed_frames = sample.frames.clone();
            reversed_frames.reverse();
            root.add_stack(&reversed_frames);
        }

        // Generate SVG
        self.render_svg(&root)
    }

    /// Generate a differential flame graph comparing baseline and current samples
    ///
    /// This is a convenience method that creates a DiffFlameGraphGenerator
    /// with settings derived from this generator's configuration.
    ///
    /// # Arguments
    ///
    /// * `baseline` - The baseline stack samples (e.g., before optimization)
    /// * `current` - The current stack samples (e.g., after optimization)
    ///
    /// # Returns
    ///
    /// An SVG string representing the differential flame graph
    ///
    /// # Example
    ///
    /// ```no_run
    /// use keyrx_core::profiling::{FlameGraphGenerator, FlameGraphConfig};
    /// # use keyrx_core::profiling::StackSample;
    /// # use std::time::Duration;
    ///
    /// let generator = FlameGraphGenerator::default();
    /// # let baseline: Vec<StackSample> = vec![];
    /// # let current: Vec<StackSample> = vec![];
    /// let svg = generator.generate_diff(&baseline, &current);
    /// ```
    pub fn generate_diff(&self, baseline: &[StackSample], current: &[StackSample]) -> String {
        use super::flamegraph_diff::{DiffFlameGraphConfig, DiffFlameGraphGenerator};

        let diff_config = DiffFlameGraphConfig::from(self.config.clone());
        let diff_generator = DiffFlameGraphGenerator::new(diff_config);
        diff_generator.generate(baseline, current)
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

    /// Render the flame tree as SVG
    fn render_svg(&self, root: &FlameNode) -> String {
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
</style>
<rect x="0" y="0" width="100%" height="100%" fill="url(#background)" />
<text x="{}" y="24" class="title">{}</text>
"##,
            self.config.width,
            self.config.height,
            self.config.width / 2,
            self.config.title
        ));

        // Render frames starting below the title
        let y_offset = 40;
        let total_samples = root.value;
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

    /// Recursively render a frame and its children
    #[allow(clippy::too_many_arguments)]
    fn render_frame(
        &self,
        svg: &mut String,
        node: &FlameNode,
        x: f64,
        y: f64,
        width: f64,
        total: u64,
        depth: usize,
    ) {
        if width < self.config.min_width {
            return;
        }

        // Calculate frame dimensions
        let height = self.config.frame_height as f64;

        // Generate color based on frame name and depth
        let color = self.get_color(&node.name, depth);

        // Escape text for XML
        let escaped_name = self.escape_xml(&node.name);
        let title = format!("{} ({} samples)", escaped_name, node.value);

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

        // Sort children by value for consistent rendering
        let mut children: Vec<_> = node.children.values().collect();
        children.sort_by(|a, b| b.value.cmp(&a.value));

        for child in children {
            let child_width = (child.value as f64 / total as f64) * width;
            self.render_frame(svg, child, child_x, child_y, child_width, total, depth + 1);
            child_x += child_width;
        }
    }

    /// Get a color for a frame based on the color scheme
    fn get_color(&self, name: &str, depth: usize) -> String {
        match self.config.color_scheme {
            ColorScheme::Hot => self.hot_color(name),
            ColorScheme::Cool => self.cool_color(name),
            ColorScheme::Rainbow => self.rainbow_color(depth),
            ColorScheme::Grayscale => self.grayscale_color(name),
        }
    }

    /// Generate a hot color (red/orange/yellow)
    fn hot_color(&self, name: &str) -> String {
        let hash = self.hash_string(name);
        let r = 200 + (hash % 56) as u8;
        let g = 50 + (hash / 2 % 150) as u8;
        let b = 0;
        format!("rgb({},{},{})", r, g, b)
    }

    /// Generate a cool color (blue/green)
    fn cool_color(&self, name: &str) -> String {
        let hash = self.hash_string(name);
        let r = 0;
        let g = 50 + (hash / 2 % 150) as u8;
        let b = 200 + (hash % 56) as u8;
        format!("rgb({},{},{})", r, g, b)
    }

    /// Generate a rainbow color based on depth
    fn rainbow_color(&self, depth: usize) -> String {
        let hue = (depth * 40) % 360;
        self.hsl_to_rgb(hue as f64, 0.7, 0.6)
    }

    /// Generate a grayscale color
    fn grayscale_color(&self, name: &str) -> String {
        let hash = self.hash_string(name);
        let gray = 150 + (hash % 80) as u8;
        format!("rgb({},{},{})", gray, gray, gray)
    }

    /// Simple hash function for string
    fn hash_string(&self, s: &str) -> u32 {
        s.bytes()
            .fold(0u32, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u32))
    }

    /// Convert HSL to RGB color string
    fn hsl_to_rgb(&self, h: f64, s: f64, l: f64) -> String {
        let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
        let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
        let m = l - c / 2.0;

        let (r, g, b) = if h < 60.0 {
            (c, x, 0.0)
        } else if h < 120.0 {
            (x, c, 0.0)
        } else if h < 180.0 {
            (0.0, c, x)
        } else if h < 240.0 {
            (0.0, x, c)
        } else if h < 300.0 {
            (x, 0.0, c)
        } else {
            (c, 0.0, x)
        };

        format!(
            "rgb({},{},{})",
            ((r + m) * 255.0) as u8,
            ((g + m) * 255.0) as u8,
            ((b + m) * 255.0) as u8
        )
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

    fn create_sample_stacks() -> Vec<StackSample> {
        vec![
            StackSample {
                timestamp: Duration::from_millis(0),
                frames: vec![
                    "main".to_string(),
                    "process_input".to_string(),
                    "parse_command".to_string(),
                ],
            },
            StackSample {
                timestamp: Duration::from_millis(10),
                frames: vec![
                    "main".to_string(),
                    "process_input".to_string(),
                    "parse_command".to_string(),
                ],
            },
            StackSample {
                timestamp: Duration::from_millis(20),
                frames: vec![
                    "main".to_string(),
                    "process_input".to_string(),
                    "execute_command".to_string(),
                ],
            },
        ]
    }

    #[test]
    fn test_flame_graph_generation() {
        let generator = FlameGraphGenerator::default();
        let stacks = create_sample_stacks();
        let svg = generator.generate(&stacks);

        // Verify basic SVG structure
        assert!(svg.contains("<?xml"));
        assert!(svg.contains("<svg"));
        assert!(svg.contains("</svg>"));
        assert!(svg.contains("Flame Graph"));
    }

    #[test]
    fn test_empty_flame_graph() {
        let generator = FlameGraphGenerator::default();
        let svg = generator.generate(&[]);

        assert!(svg.contains("No samples collected"));
    }

    #[test]
    fn test_frame_names_in_output() {
        let generator = FlameGraphGenerator::default();
        let stacks = create_sample_stacks();
        let svg = generator.generate(&stacks);

        assert!(svg.contains("main"));
        assert!(svg.contains("process_input"));
        assert!(svg.contains("parse_command"));
        assert!(svg.contains("execute_command"));
    }

    #[test]
    fn test_custom_config() {
        let config = FlameGraphConfig {
            width: 800,
            height: 600,
            color_scheme: ColorScheme::Cool,
            title: "Test Graph".to_string(),
            ..Default::default()
        };

        let generator = FlameGraphGenerator::new(config);
        let stacks = create_sample_stacks();
        let svg = generator.generate(&stacks);

        assert!(svg.contains("width=\"800\""));
        assert!(svg.contains("height=\"600\""));
        assert!(svg.contains("Test Graph"));
    }

    #[test]
    fn test_color_schemes() {
        let stacks = create_sample_stacks();

        for scheme in [
            ColorScheme::Hot,
            ColorScheme::Cool,
            ColorScheme::Rainbow,
            ColorScheme::Grayscale,
        ] {
            let config = FlameGraphConfig {
                color_scheme: scheme,
                ..Default::default()
            };
            let generator = FlameGraphGenerator::new(config);
            let svg = generator.generate(&stacks);

            // Should contain color definitions
            assert!(svg.contains("rgb("));
        }
    }

    #[test]
    fn test_xml_escaping() {
        let generator = FlameGraphGenerator::default();
        let stacks = vec![StackSample {
            timestamp: Duration::from_millis(0),
            frames: vec!["<script>alert('xss')</script>".to_string()],
        }];

        let svg = generator.generate(&stacks);

        // Should not contain unescaped XML
        assert!(!svg.contains("<script>"));
        assert!(svg.contains("&lt;script&gt;"));
    }

    #[test]
    fn test_flame_node_building() {
        let mut root = FlameNode::new("root".to_string());

        let frames = vec!["main".to_string(), "foo".to_string(), "bar".to_string()];
        root.add_stack(&frames);
        root.add_stack(&frames);

        assert_eq!(root.value, 2);
        assert_eq!(root.children.len(), 1);
        assert!(root.children.contains_key("main"));

        let main_node = &root.children["main"];
        assert_eq!(main_node.value, 2);
        assert_eq!(main_node.children.len(), 1);
    }

    #[test]
    fn test_hash_consistency() {
        let generator = FlameGraphGenerator::default();

        let hash1 = generator.hash_string("test");
        let hash2 = generator.hash_string("test");
        assert_eq!(hash1, hash2);

        let hash3 = generator.hash_string("other");
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_generate_diff() {
        let generator = FlameGraphGenerator::default();

        let baseline = vec![
            StackSample {
                timestamp: Duration::from_millis(0),
                frames: vec!["main".to_string(), "slow".to_string()],
            },
            StackSample {
                timestamp: Duration::from_millis(10),
                frames: vec!["main".to_string(), "slow".to_string()],
            },
        ];

        let current = vec![
            StackSample {
                timestamp: Duration::from_millis(0),
                frames: vec!["main".to_string(), "fast".to_string()],
            },
            StackSample {
                timestamp: Duration::from_millis(10),
                frames: vec!["main".to_string(), "fast".to_string()],
            },
        ];

        let svg = generator.generate_diff(&baseline, &current);

        // Verify it's a valid SVG differential flame graph
        assert!(svg.contains("<?xml"));
        assert!(svg.contains("<svg"));
        assert!(svg.contains("</svg>"));
        // Check for "Diff" in the title (since it prefixes the original title)
        assert!(svg.contains("Diff:"));
        assert!(svg.contains("main"));
        // Verify differential coloring is present
        assert!(svg.contains("legend"));
    }
}
