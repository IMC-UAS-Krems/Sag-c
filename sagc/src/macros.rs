#[macro_export]
/// Macro that returns a field while checking if this field is present and its type is correct
/// parse!(`HashMap`, `type`, `section_name`, `field_name`)
macro_rules! parse {
    ($map:expr, Option<$fty:ty>, $block_name:expr, $field_name:expr) => {
        match $map {
            Value::Block(map) => match map.get($field_name) {
                Some(field) => Some(
                    TryInto::<$fty>::try_into(field)
                        .map_err(|e| SagError::parsing_error($block_name, Some($field_name), e))?,
                ),
                None => None,
            },
            _ => unreachable!(),
        }
    };
    ($map:expr, $fty:ty, $block_name:expr, $field_name:expr) => {
        match $map {
            Value::Block(map) => match map.get($field_name) {
                Some(field) => {
                    let result: Result<$fty, _> = TryInto::<$fty>::try_into(field);
                    result
                        .map_err(|e| SagError::parsing_error($block_name, Some($field_name), e))?
                }
                None => return Err(SagError::missing_field($block_name, $field_name)),
            },
            _ => unreachable!(),
        }
    };
}
