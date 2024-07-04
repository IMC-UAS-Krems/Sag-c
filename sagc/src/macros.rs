#[macro_export]
/// Macro that returns a field while checking if this field is present and its type is correct
/// parse!(`HashMap`, `type`, `section_name`, `field_name`)
macro_rules! parse {
    ($map:expr, Option<$fty:ty>, $block_name:expr, $field_name:expr) => {
        match &mut $map.value {
            Value::Block(block) => match block.get_mut($field_name) {
                Some(field) => Some(
                    TryInto::<$fty>::try_into(&mut field.value)
                        .map_err(|_| {
                            SagError::language_error(
                                LanguageErrorKind::InvalidType(),
                                field.position,
                            )
                        })
                        .map(|v| (v, field.position))
                        .unwrap(),
                ),
                None => None,
            },
            _ => unreachable!(),
        }
    };
    ($map:expr, $fty:ty, $block_name:expr, $field_name:expr) => {
        match &mut $map.value {
            Value::Block(block) => match block.get_mut($field_name) {
                Some(field) => {
                    let result: Result<$fty, _> = TryInto::<$fty>::try_into(&mut field.value);
                    result
                        .map_err(|_| {
                            SagError::language_error(
                                LanguageErrorKind::InvalidType(),
                                field.position,
                            )
                        })
                        .map(|v| (v, field.position))
                }
                None => Err(SagError::language_error(
                    LanguageErrorKind::MissingField($field_name.to_string()),
                    $map.position,
                )),
            },
            _ => unreachable!(),
        }
    };
}
