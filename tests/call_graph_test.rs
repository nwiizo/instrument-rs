//! Integration tests for call graph construction

use instrument_rs::call_graph::{GraphBuilder, NodeKind, CallKind};
use std::fs;
use std::path::Path;
use tempfile::TempDir;

#[test]
fn test_simple_call_graph() {
    let temp_dir = TempDir::new().unwrap();
    let src_dir = temp_dir.path().join("src");
    fs::create_dir(&src_dir).unwrap();
    
    // Create a test file with various function calls
    let test_code = r#"
        fn main() {
            println!("Starting");
            process_data();
            helper::utility();
        }
        
        fn process_data() {
            let data = vec![1, 2, 3];
            data.iter().for_each(|x| println!("{}", x));
            validate(&data);
        }
        
        fn validate(data: &[i32]) -> bool {
            !data.is_empty()
        }
        
        mod helper {
            pub fn utility() {
                super::validate(&[1, 2, 3]);
            }
        }
        
        #[test]
        fn test_validate() {
            assert!(validate(&[1]));
            assert!(!validate(&[]));
        }
    "#;
    
    fs::write(src_dir.join("main.rs"), test_code).unwrap();
    
    // Build the call graph
    let mut builder = GraphBuilder::new();
    let graph = builder.build_from_directory(&src_dir).unwrap();
    
    // Verify nodes exist
    assert!(graph.get_node("main").is_some());
    assert!(graph.get_node("process_data").is_some());
    assert!(graph.get_node("validate").is_some());
    assert!(graph.get_node("helper::utility").is_some());
    assert!(graph.get_node("test_validate").is_some());
    
    // Verify node kinds
    let main_node = graph.get_node("main").unwrap();
    assert_eq!(main_node.kind, NodeKind::Endpoint);
    
    let test_node = graph.get_node("test_validate").unwrap();
    assert_eq!(test_node.kind, NodeKind::Test);
    
    let process_node = graph.get_node("process_data").unwrap();
    assert_eq!(process_node.kind, NodeKind::Internal);
    
    // Verify edges exist
    let edges = graph.edges();
    assert!(edges.iter().any(|e| e.from == "main" && e.to == "process_data"));
    assert!(edges.iter().any(|e| e.from == "process_data" && e.to == "validate"));
    assert!(edges.iter().any(|e| e.from == "helper::utility" && e.to == "validate"));
    
    // Verify reachability
    let reachable_from_main = graph.find_reachable("main");
    assert!(reachable_from_main.contains("process_data"));
    assert!(reachable_from_main.contains("validate"));
    
    // Verify graph statistics
    let stats = graph.stats();
    assert_eq!(stats.endpoint_count, 1); // main
    assert_eq!(stats.test_count, 1); // test_validate
    assert_eq!(stats.internal_count, 3); // process_data, validate, helper::utility
}

#[test]
fn test_recursive_functions() {
    let temp_dir = TempDir::new().unwrap();
    let src_dir = temp_dir.path().join("src");
    fs::create_dir(&src_dir).unwrap();
    
    let recursive_code = r#"
        fn factorial(n: u32) -> u32 {
            if n <= 1 {
                1
            } else {
                n * factorial(n - 1)
            }
        }
        
        fn fibonacci(n: u32) -> u32 {
            match n {
                0 => 0,
                1 => 1,
                _ => fibonacci(n - 1) + fibonacci(n - 2),
            }
        }
        
        fn main() {
            println!("5! = {}", factorial(5));
            println!("fib(10) = {}", fibonacci(10));
        }
    "#;
    
    fs::write(src_dir.join("lib.rs"), recursive_code).unwrap();
    
    let mut builder = GraphBuilder::new();
    let graph = builder.build_from_directory(&src_dir).unwrap();
    
    // Check for recursive edges
    let edges = graph.edges();
    assert!(edges.iter().any(|e| e.from == "factorial" && e.to == "factorial" && e.kind == CallKind::Recursive));
    assert!(edges.iter().any(|e| e.from == "fibonacci" && e.to == "fibonacci" && e.kind == CallKind::Recursive));
}

#[test]
fn test_cycle_detection() {
    let temp_dir = TempDir::new().unwrap();
    let src_dir = temp_dir.path().join("src");
    fs::create_dir(&src_dir).unwrap();
    
    let cyclic_code = r#"
        fn a() {
            b();
        }
        
        fn b() {
            c();
        }
        
        fn c() {
            a();
        }
        
        fn main() {
            a();
        }
    "#;
    
    fs::write(src_dir.join("main.rs"), cyclic_code).unwrap();
    
    let mut builder = GraphBuilder::new();
    let graph = builder.build_from_directory(&src_dir).unwrap();
    
    // Detect cycles
    let cycles = graph.find_cycles();
    assert!(!cycles.is_empty());
    
    // Should find the a -> b -> c -> a cycle
    let has_expected_cycle = cycles.iter().any(|cycle| {
        cycle.len() == 3 && 
        cycle.contains(&"a".to_string()) &&
        cycle.contains(&"b".to_string()) &&
        cycle.contains(&"c".to_string())
    });
    assert!(has_expected_cycle);
}

#[test]
fn test_external_dependencies() {
    let temp_dir = TempDir::new().unwrap();
    let src_dir = temp_dir.path().join("src");
    fs::create_dir(&src_dir).unwrap();
    
    let external_code = r#"
        use std::collections::HashMap;
        use std::fs::File;
        
        fn create_map() -> HashMap<String, i32> {
            let mut map = HashMap::new();
            map.insert("key".to_string(), 42);
            map
        }
        
        fn read_file() -> std::io::Result<String> {
            std::fs::read_to_string("data.txt")
        }
        
        fn main() {
            let map = create_map();
            if let Ok(content) = read_file() {
                println!("Read: {}", content);
            }
        }
    "#;
    
    fs::write(src_dir.join("main.rs"), external_code).unwrap();
    
    let mut builder = GraphBuilder::new();
    let graph = builder.build_from_directory(&src_dir).unwrap();
    
    // Check for external nodes
    let external_nodes = graph.nodes_by_kind(NodeKind::External);
    assert!(!external_nodes.is_empty());
    
    // Should detect standard library calls
    let has_hashmap = external_nodes.iter().any(|node| 
        node.name == "new" || node.module_path.contains(&"HashMap".to_string())
    );
    assert!(has_hashmap);
}

#[test]
fn test_path_finding() {
    let temp_dir = TempDir::new().unwrap();
    let src_dir = temp_dir.path().join("src");
    fs::create_dir(&src_dir).unwrap();
    
    let path_code = r#"
        fn main() {
            a();
        }
        
        fn a() {
            b();
        }
        
        fn b() {
            c();
        }
        
        fn c() {
            d();
        }
        
        fn d() {
            println!("End of chain");
        }
    "#;
    
    fs::write(src_dir.join("main.rs"), path_code).unwrap();
    
    let mut builder = GraphBuilder::new();
    let graph = builder.build_from_directory(&src_dir).unwrap();
    
    // Find path from main to d
    let path = graph.find_path("main", "d");
    assert!(path.is_some());
    
    let path = path.unwrap();
    assert_eq!(path, vec!["main", "a", "b", "c", "d"]);
    
    // No path from d to main (not cyclic)
    let reverse_path = graph.find_path("d", "main");
    assert!(reverse_path.is_none());
}