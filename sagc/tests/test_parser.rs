#[cfg(test)]
mod tests {
    use sagc::parser::{Blocks, Value, Position, ParseResult, Span, Token, TokenValue, handle_error, parse_block_name};
    use std::collections::HashMap;

// =====BLOCK=====

    #[test]
    fn test_blocks_new(){
        let blocks = Blocks::new();
        assert!(blocks.blocks.is_empty());

    }

    #[test]
    fn test_block_insertion() {
        let mut blocks = Blocks::new();
        let key = "test_block";
        let value = ParseResult::new(Position { row_start: 1, row_end: 1, col_start: 1, col_end: 5 }, Value::String("forced_inser_block"));
        
        blocks.insert(key, value);
        
        assert_eq!(blocks.blocks.len(), 1);
        assert!(blocks.blocks.contains_key("test_block"));

    }

    #[test]
    fn test_block_get() {
        let mut blocks = Blocks::new();
        let val1 = ParseResult::new(Position { row_start: 1, row_end: 1, col_start: 1, col_end: 5 }, Value::String("V1"));
        let val2 = ParseResult::new(Position { row_start: 1, row_end: 1, col_start: 79, col_end: 5 }, Value::String("V2"));
        blocks.insert("b1",val1);
        blocks.insert("b2",val2);
        
        let val1 = blocks.get_mut("b1");
        let val2 = blocks.get_mut("b2");
        let val_none = blocks.get_mut("notakey");
    }

    #[test]
    fn test_is_block() {
        let string_value = Value::String("test");
        let vec_value = Value::Vec(vec!["a", "b", "c"]);
        let block_value = Value::Block(Blocks::new());

        assert!(!string_value.is_block(), "Expected String variant to return false");
        assert!(!vec_value.is_block(), "Expected Vec variant to return false");
        assert!(block_value.is_block(), "Expected Block variant to return true");
    }

    // =====VALUE CONVERSION=====

    #[test]
    fn test_try_into_str() {
        let val = Value::String("test_string");
        let mut val_mut = val;

        let result: Result<&str, _> = TryInto::try_into(&mut val_mut);
        assert_eq!(result.unwrap(), "test_string");
    }

    #[test]
    fn test_try_into_num() {
        let val = Value::String("42");
        let mut val_mut = val;

        let result: Result<usize, _> = TryInto::try_into(&mut val_mut);

        assert_eq!(result.unwrap(), 42);

        let val1 = Value::String("not_a_number");
        let mut val_mut1 = val1;
        let result: Result<usize, _> = TryInto::try_into(&mut val_mut1);

        assert!(result.is_err(), "Expected error when parsing non-numeric string as usize");
    }

    #[test]
    fn test_try_into_multival() {
        let val = Value::Vec(vec!["test1", "X", "test3"]);
        let mut val_mut = val;

        let result: Result<Vec<&str>, _> = TryInto::try_into(&mut val_mut);

        assert_eq!(result.unwrap(), vec!["test1", "X", "test3"]);
    }

    #[test]
    fn test_try_into_multival_str() {
        let val = Value::Vec(vec!["a", "b", "c"]);
        let mut val_mut = val;

        let result: Result<&str, _> = TryInto::try_into(&mut val_mut);
        assert!(result.is_err(), "Vector should not convert to str");
    }
}