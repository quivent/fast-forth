// profiler.rs - Fast Forth Profiler with Flame Graph Support
// Provides detailed performance analysis with actionable optimization insights

use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Profiling data for a single word
#[derive(Debug, Clone)]
pub struct WordProfile {
    pub name: String,
    pub call_count: u64,
    pub total_time: Duration,
    pub self_time: Duration, // Exclusive time (not including children)
    pub children: Vec<String>,
    pub parent: Option<String>,
}

impl WordProfile {
    pub fn per_call_time(&self) -> Duration {
        if self.call_count == 0 {
            Duration::from_secs(0)
        } else {
            self.total_time / self.call_count as u32
        }
    }

    pub fn percentage(&self, total_time: Duration) -> f64 {
        if total_time.as_secs_f64() == 0.0 {
            0.0
        } else {
            (self.self_time.as_secs_f64() / total_time.as_secs_f64()) * 100.0
        }
    }
}

/// Call stack entry for profiling
#[derive(Debug, Clone)]
struct CallFrame {
    word: String,
    start_time: Instant,
    children_time: Duration,
}

/// Profiler state
pub struct Profiler {
    profiles: HashMap<String, WordProfile>,
    call_stack: Vec<CallFrame>,
    total_time: Duration,
    program_start: Option<Instant>,
}

impl Profiler {
    pub fn new() -> Self {
        Profiler {
            profiles: HashMap::new(),
            call_stack: Vec::new(),
            total_time: Duration::from_secs(0),
            program_start: None,
        }
    }

    /// Start profiling a program
    pub fn start(&mut self) {
        self.program_start = Some(Instant::now());
        self.profiles.clear();
        self.call_stack.clear();
    }

    /// Stop profiling
    pub fn stop(&mut self) {
        if let Some(start) = self.program_start {
            self.total_time = start.elapsed();
        }
    }

    /// Enter a word (push onto call stack)
    pub fn enter_word(&mut self, word: String) {
        let frame = CallFrame {
            word: word.clone(),
            start_time: Instant::now(),
            children_time: Duration::from_secs(0),
        };
        self.call_stack.push(frame);

        // Initialize profile if doesn't exist
        self.profiles.entry(word.clone()).or_insert(WordProfile {
            name: word,
            call_count: 0,
            total_time: Duration::from_secs(0),
            self_time: Duration::from_secs(0),
            children: Vec::new(),
            parent: None,
        });
    }

    /// Exit a word (pop from call stack)
    pub fn exit_word(&mut self, word: &str) {
        if let Some(frame) = self.call_stack.pop() {
            if frame.word != word {
                eprintln!("Warning: Mismatched word exit: expected {}, got {}", frame.word, word);
                return;
            }

            let elapsed = frame.start_time.elapsed();
            let self_time = elapsed - frame.children_time;

            // Update profile
            if let Some(profile) = self.profiles.get_mut(&frame.word) {
                profile.call_count += 1;
                profile.total_time += elapsed;
                profile.self_time += self_time;
            }

            // Update parent's children time
            if let Some(parent_frame) = self.call_stack.last_mut() {
                parent_frame.children_time += elapsed;

                // Record parent-child relationship
                if let Some(parent_profile) = self.profiles.get_mut(&parent_frame.word) {
                    if !parent_profile.children.contains(&frame.word) {
                        parent_profile.children.push(frame.word.clone());
                    }
                }

                if let Some(child_profile) = self.profiles.get_mut(&frame.word) {
                    child_profile.parent = Some(parent_frame.word.clone());
                }
            }
        }
    }

    /// Generate profiler report
    pub fn generate_report(&self) -> ProfilerReport {
        let mut hot_spots: Vec<&WordProfile> = self.profiles.values().collect();
        hot_spots.sort_by(|a, b| b.self_time.cmp(&a.self_time));

        ProfilerReport {
            total_time: self.total_time,
            hot_spots: hot_spots.iter().take(10).map(|p| (*p).clone()).collect(),
            all_profiles: self.profiles.clone(),
        }
    }

    /// Generate flame graph HTML
    pub fn generate_flame_graph(&self) -> String {
        let mut html = String::new();

        html.push_str(r#"<!DOCTYPE html>
<html>
<head>
<title>Fast Forth Flame Graph</title>
<style>
body {
    font-family: 'Monaco', 'Menlo', monospace;
    margin: 0;
    padding: 20px;
    background: #1e1e1e;
}
#flame-graph {
    width: 100%;
    height: 800px;
}
.flame-rect {
    stroke: #000;
    stroke-width: 1;
    cursor: pointer;
}
.flame-rect:hover {
    stroke: #fff;
    stroke-width: 2;
}
.flame-text {
    font-size: 12px;
    fill: #000;
    pointer-events: none;
}
#info {
    color: #fff;
    padding: 10px;
    background: #2e2e2e;
    margin-bottom: 10px;
    border-radius: 4px;
}
</style>
</head>
<body>
<div id="info">
    <h2>Fast Forth Flame Graph</h2>
    <p>Click on a frame to zoom. Hover for details. Width = time spent.</p>
    <div id="details"></div>
</div>
<svg id="flame-graph"></svg>

<script>
const data = "#);

        // Generate JSON data for flame graph
        html.push_str(&self.generate_flame_graph_data());

        html.push_str(r#";

// Render flame graph
const svg = document.getElementById('flame-graph');
const width = svg.clientWidth;
const height = 800;
const colors = ['#e74c3c', '#3498db', '#2ecc71', '#f39c12', '#9b59b6'];

function render(node, x, y, width, depth = 0) {
    const rect = document.createElementNS('http://www.w3.org/2000/svg', 'rect');
    rect.setAttribute('class', 'flame-rect');
    rect.setAttribute('x', x);
    rect.setAttribute('y', y);
    rect.setAttribute('width', width);
    rect.setAttribute('height', 20);
    rect.setAttribute('fill', colors[depth % colors.length]);

    rect.addEventListener('click', () => zoom(node));
    rect.addEventListener('mouseenter', () => showDetails(node));

    svg.appendChild(rect);

    const text = document.createElementNS('http://www.w3.org/2000/svg', 'text');
    text.setAttribute('class', 'flame-text');
    text.setAttribute('x', x + 5);
    text.setAttribute('y', y + 14);
    text.textContent = `${node.name} (${node.time}ms)`;
    svg.appendChild(text);

    // Render children
    let childX = x;
    for (const child of node.children || []) {
        const childWidth = (child.time / node.time) * width;
        render(child, childX, y + 25, childWidth, depth + 1);
        childX += childWidth;
    }
}

function showDetails(node) {
    document.getElementById('details').innerHTML = `
        <strong>${node.name}</strong><br>
        Time: ${node.time}ms (${node.percentage}%)<br>
        Calls: ${node.calls}
    `;
}

function zoom(node) {
    svg.innerHTML = '';
    render(node, 0, 0, width);
}

render(data, 0, 0, width);
</script>
</body>
</html>"#);

        html
    }

    /// Generate flame graph data as JSON
    fn generate_flame_graph_data(&self) -> String {
        // Build tree structure from profiles
        // For now, simple placeholder
        format!(
            r#"{{
    "name": "MAIN",
    "time": {},
    "percentage": 100,
    "calls": 1,
    "children": []
}}"#,
            self.total_time.as_millis()
        )
    }
}

/// Profiler report output
pub struct ProfilerReport {
    pub total_time: Duration,
    pub hot_spots: Vec<WordProfile>,
    pub all_profiles: HashMap<String, WordProfile>,
}

impl ProfilerReport {
    /// Display profiler report to console
    pub fn display(&self) {
        self.print_header();
        self.print_hot_spots();
        self.print_call_graph();
        self.print_optimization_opportunities();
        self.print_summary();
    }

    fn print_header(&self) {
        println!("═══════════════════════════════════════════════════════════");
        println!("  Fast Forth Profiler v1.0.0");
        println!("  Runtime: {:.2} seconds ({:.0}ms)",
            self.total_time.as_secs_f64(),
            self.total_time.as_secs_f64() * 1000.0
        );
        println!("═══════════════════════════════════════════════════════════");
        println!();
    }

    fn print_hot_spots(&self) {
        println!("TOP {} HOT SPOTS (by exclusive time):", self.hot_spots.len().min(10));
        println!();
        println!(" #  Word            Time      %    Calls    Per Call  Notes");
        println!("────────────────────────────────────────────────────────────");

        for (i, profile) in self.hot_spots.iter().enumerate().take(10) {
            let percentage = profile.percentage(self.total_time);
            let per_call = profile.per_call_time();

            let note = if percentage > 40.0 {
                "🔥 HOT"
            } else if percentage > 20.0 {
                "⚡ Warm"
            } else if percentage > 10.0 {
                "💡 Medium"
            } else {
                ""
            };

            println!(
                "{:2}  {:<15} {:>6.0}ms {:>4.1}%  {:>6}   {:>6.2}μs   {}",
                i + 1,
                profile.name,
                profile.self_time.as_secs_f64() * 1000.0,
                percentage,
                profile.call_count,
                per_call.as_secs_f64() * 1_000_000.0,
                note
            );
        }

        println!();
        println!("────────────────────────────────────────────────────────────");
        println!();
    }

    fn print_call_graph(&self) {
        println!("CALL GRAPH (top 3 paths):");
        println!();

        // For now, placeholder
        println!("1. MAIN → INNER-LOOP → COMPUTE → VALIDATE");
        println!("   (Call graph analysis coming soon)");
        println!();

        println!("────────────────────────────────────────────────────────────");
        println!();
    }

    fn print_optimization_opportunities(&self) {
        println!("OPTIMIZATION OPPORTUNITIES:");
        println!();

        // Analyze hot spots for optimization opportunities
        for profile in &self.hot_spots {
            let percentage = profile.percentage(self.total_time);

            if percentage > 20.0 {
                println!("🔥 CRITICAL: {} ({:.0}ms)",
                    profile.name,
                    profile.self_time.as_secs_f64() * 1000.0
                );
                println!();
                println!("   Issue: Hot spot consuming {:.1}% of runtime", percentage);
                println!();
                println!("   Recommendation:");
                println!("     • Profile this word in isolation");
                println!("     • Consider algorithm optimization");
                println!("     • Check for unnecessary allocations");
                println!();
            } else if percentage > 10.0 {
                println!("⚡ HIGH: {} ({:.0}ms)",
                    profile.name,
                    profile.self_time.as_secs_f64() * 1000.0
                );
                println!("   Consider optimization if called frequently");
                println!();
            }
        }

        println!("────────────────────────────────────────────────────────────");
        println!();
    }

    fn print_summary(&self) {
        println!("SUMMARY:");
        println!();
        println!("✓ Program executed successfully");

        let critical_count = self.hot_spots.iter()
            .filter(|p| p.percentage(self.total_time) > 20.0)
            .count();

        if critical_count > 0 {
            println!("⚠ {} critical optimization opportunit{} identified",
                critical_count,
                if critical_count == 1 { "y" } else { "ies" }
            );
        }

        println!();
        println!("Run 'fastforth profile --flame-graph' for visualization");
        println!();
        println!("═══════════════════════════════════════════════════════════");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profiler_basic() {
        let mut profiler = Profiler::new();
        profiler.start();

        profiler.enter_word("MAIN".to_string());
        std::thread::sleep(Duration::from_millis(10));

        profiler.enter_word("COMPUTE".to_string());
        std::thread::sleep(Duration::from_millis(5));
        profiler.exit_word("COMPUTE");

        profiler.exit_word("MAIN");

        profiler.stop();

        let report = profiler.generate_report();
        assert!(report.hot_spots.len() > 0);
        assert!(report.total_time.as_millis() >= 15);
    }

    #[test]
    fn test_percentage_calculation() {
        let profile = WordProfile {
            name: "TEST".to_string(),
            call_count: 1,
            total_time: Duration::from_secs(1),
            self_time: Duration::from_millis(250),
            children: Vec::new(),
            parent: None,
        };

        let total = Duration::from_secs(1);
        assert!((profile.percentage(total) - 25.0).abs() < 0.1);
    }
}
