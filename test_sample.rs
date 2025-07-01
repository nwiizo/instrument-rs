// Test sample file for AST analysis

fn main() {
    println!("Hello, world!");
    let result = process_data(vec![1, 2, 3]);
    println!("Result: {:?}", result);
}

fn process_data(data: Vec<i32>) -> Result<i32, String> {
    if data.is_empty() {
        return Err("Empty data".to_string());
    }
    
    let sum = data.iter().sum::<i32>();
    
    if sum > 100 {
        Ok(sum)
    } else {
        match validate_sum(sum) {
            Some(validated) => Ok(validated),
            None => Err("Invalid sum".to_string()),
        }
    }
}

fn validate_sum(sum: i32) -> Option<i32> {
    if sum > 0 {
        Some(sum)
    } else {
        None
    }
}

#[test]
fn test_process_data() {
    assert_eq!(process_data(vec![1, 2, 3]).unwrap(), 6);
    assert!(process_data(vec![]).is_err());
}