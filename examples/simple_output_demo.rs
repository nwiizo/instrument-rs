//! Simple demonstration of output formatters without dependencies on other modules

use std::collections::HashMap;

fn main() {
    println!("=== Output Formatter Demonstration ===\n");

    // Tree view example
    println!("1. Tree View Output:");
    println!("--------------------");
    print_tree_example();

    println!("\n2. JSON Output:");
    println!("---------------");
    print_json_example();

    println!("\n3. Mermaid Diagram:");
    println!("-------------------");
    print_mermaid_example();
}

fn print_tree_example() {
    println!("Project Coverage Summary");
    println!("├── Statistics: Line: 85.2% | Branch: 72.1% | Function: 90.0%");
    println!("└── Files (3)");
    println!("    ├── src/main.rs: 95.0%");
    println!("    ├── src/lib.rs: 75.5%");
    println!("    └── src/utils.rs: 80.0%");
    println!("\nCoverage Overview:");
    println!("  Line:     [████████████████░░░░] 85.2%");
    println!("  Branch:   [██████████████░░░░░░] 72.1%");
    println!("  Function: [██████████████████░░] 90.0%");
}

fn print_json_example() {
    let json = r#"{
  "version": "1.0",
  "generated_at": "2024-01-01T12:00:00Z",
  "summary": {
    "line_coverage_percent": 85.2,
    "branch_coverage_percent": 72.1,
    "function_coverage_percent": 90.0,
    "total_files": 3,
    "file_coverage": [
      {
        "file_path": "src/main.rs",
        "line_coverage_percent": 95.0
      }
    ]
  },
  "metadata": {
    "tool": "instrument-rs",
    "tool_version": "0.1.0"
  }
}"#;
    println!("{}", json);
}

fn print_mermaid_example() {
    println!("```mermaid");
    println!("flowchart TD");
    println!("    Start[Project Coverage: 85.2%]");
    println!("    Start --> Stats[Statistics]");
    println!("    Stats --> LineC[Line Coverage: 85.2%]");
    println!("    Stats --> BranchC[Branch Coverage: 72.1%]");
    println!("    Stats --> FuncC[Function Coverage: 90.0%]");
    println!("    Start --> Files[Files: 3]");
    println!("    Files --> High[High Coverage: 1 file]");
    println!("    Files --> Med[Medium Coverage: 2 files]");
    println!("    ");
    println!("    classDef highCov fill:#4ade80,stroke:#16a34a,color:#fff");
    println!("    classDef medCov fill:#facc15,stroke:#ca8a04,color:#000");
    println!("    class High highCov");
    println!("    class Med medCov");
    println!("```");

    println!("\n```mermaid");
    println!("pie title Mutation Testing Results");
    println!("    \"Killed\" : 45");
    println!("    \"Survived\" : 12");
    println!("    \"Timeout\" : 3");
    println!("```");
}
