//! Test framework adapters

use crate::framework::{RunnerConfig, TestFramework, TestResults, TestRunner};
use crate::Result;

/// Built-in test framework adapter
pub struct BuiltinTestAdapter;

impl TestRunner for BuiltinTestAdapter {
    fn framework(&self) -> TestFramework {
        TestFramework::BuiltinTest
    }

    fn build_test_command(&self, config: &RunnerConfig) -> Vec<String> {
        vec!["cargo".to_string(), "test".to_string()]
    }

    fn parse_results(&self, output: &str) -> Result<TestResults> {
        // TODO: Implement test output parsing
        Ok(TestResults {
            passed: 0,
            failed: 0,
            ignored: 0,
            duration_ms: 0,
            tests: vec![],
        })
    }

    fn is_success(&self, exit_code: i32, _output: &str) -> bool {
        exit_code == 0
    }
}
