//! 기본 기능 테스트 - 최소한의 동작 검증

#[cfg(test)]
mod tests {
    #[test]
    fn test_basic_math() {
        // 가장 기본적인 테스트
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn test_string_operations() {
        let hello = "Hello";
        let world = "World";
        let combined = format!("{} {}", hello, world);
        assert_eq!(combined, "Hello World");
    }

    #[test]
    fn test_vector_operations() {
        let mut vec = vec![1, 2, 3];
        vec.push(4);
        assert_eq!(vec.len(), 4);
        assert_eq!(vec[3], 4);
    }

    #[test]
    fn test_option_handling() {
        let some_value: Option<i32> = Some(42);
        let none_value: Option<i32> = None;
        
        assert!(some_value.is_some());
        assert!(none_value.is_none());
        assert_eq!(some_value.unwrap(), 42);
    }

    #[test]
    fn test_result_handling() {
        fn divide(a: i32, b: i32) -> Result<i32, &'static str> {
            if b == 0 {
                Err("Division by zero")
            } else {
                Ok(a / b)
            }
        }
        
        assert!(divide(10, 2).is_ok());
        assert!(divide(10, 0).is_err());
        assert_eq!(divide(10, 2).unwrap(), 5);
    }
}