#[cfg(test)]
mod tests {
    use super::*;
    use nom::IResult;
    use sagc::parser::{TokenValue, Span, lexer, tokens_to_blocks};

    //DO NOT CHANGE FORMATTING OF STRINGS IN THIS FILE
    #[test]
    fn test_correct_sections() {
        let input = "application:
    type is Web
    dashboard is Dash
    layout is SinglePage
    roles -> User, SuperUser, Admin
    panels -> Map, Pie, XY, TS, Bar

Map:
    label is map
    type is geomap
    source is first
    data -> location, stationName, O3, NO2, SO2, address
";

        let input_span = Span::new(input);
        let result = lexer(input_span);

        match result {
            Ok(tokens) => {
                assert_eq!(tokens.len(), 40);
            },
            Err(errors) =>{ 
                println!("{:?}", errors);
                panic!("Found errors.");
            }
        }
    }

    #[test]
    fn test_indentation_error() {
        let input = "application:
    type is Web
    dashboard is Dash
layout is SinglePage
    roles -> User, SuperUser, Admin
    panels -> Map, Pie, XY, TS, Bar

Map:
    label is map
    type is geomap
    source is first
    data -> location, stationName, O3, NO2, SO2, address
";

        let input_span = Span::new(input);
        let result = lexer(input_span);

        match result {
            Ok(tokens) => {
                println!("{:?}",tokens);
                panic!("This should not succeed.");
                },
            Err(errors) => {
                assert!(matches!(errors[0].value, TokenValue::IndentError));

                let error_position = &errors[0].position;
                assert_eq!(error_position.row_start, 4);
            }
        }
    }

    #[test]
    fn test_emptylines_eol() {
        let input = "
application:
    type is Web
    dashboard is Dash
    layout is SinglePage
    roles -> User, SuperUser, Admin
    panels -> Map, Pie, XY, TS, Bar

Map:
    label is map
    type is geomap
    source is first
    data -> location, stationName, O3, NO2, SO2, address
";

        let input_span = Span::new(input);
        let result = lexer(input_span);

        match result {
            Ok(tokens) => {
                assert_eq!(tokens.len(), 41);
            },
            Err(errors) =>{ 
                println!("{:?}", errors);
                panic!("Found errors.");
            }
        }
    }

    #[test]
    fn test_emptylines_multi_eol() {
        let input = "


application:
    type is Web
    dashboard is Dash
    layout is SinglePage
    roles -> User, SuperUser, Admin
    panels -> Map, Pie, XY, TS, Bar

Map:
    label is map
    type is geomap
    source is first
    data -> location, stationName, O3, NO2, SO2, address
";

        let input_span = Span::new(input);
        let result = lexer(input_span);

        match result {
            Ok(tokens) => {
                assert_eq!(tokens.len(), 43); //eol makes a new block count
            },
            Err(errors) =>{ 
                println!("{:?}", errors);
                panic!("Found errors.");
            }
        }
    }

    #[test]
    fn test_emptylines_space() {
        let input = " 
application:
    type is Web
    dashboard is Dash
    layout is SinglePage
    roles -> User, SuperUser, Admin
    panels -> Map, Pie, XY, TS, Bar

Map:
    label is map
    type is geomap
    source is first
    data -> location, stationName, O3, NO2, SO2, address
";

        let input_span = Span::new(input);
        let result = lexer(input_span);

        match result {
            Ok(tokens) => {
                assert_eq!(tokens.len(), 40);
            },
            Err(errors) =>{ 
                println!("{:?}", errors);
                panic!("Found errors.");
            }
        }
    }

    #[test]
    fn test_emptylines_multispace() {
        let input = "        
application:
    type is Web
    dashboard is Dash
    layout is SinglePage
    roles -> User, SuperUser, Admin
    panels -> Map, Pie, XY, TS, Bar

Map:
    label is map
    type is geomap
    source is first
    data -> location, stationName, O3, NO2, SO2, address
";

        let input_span = Span::new(input);
        let result = lexer(input_span);

        match result {
            Ok(tokens) => {
                assert_eq!(tokens.len(), 41);
            },
            Err(errors) =>{ 
                println!("{:?}", errors);
                panic!("Found errors.");
            }
        }
    }

    #[test]
    fn test_emptylines_tab() {
        let input = "\t
application:
    type is Web
    dashboard is Dash
    layout is SinglePage
    roles -> User, SuperUser, Admin
    panels -> Map, Pie, XY, TS, Bar

Map:
    label is map
    type is geomap
    source is first
    data -> location, stationName, O3, NO2, SO2, address
";

        let input_span = Span::new(input);
        let result = lexer(input_span);

        match result {
            Ok(tokens) => {
                assert_eq!(tokens.len(), 40);
            },
            Err(errors) =>{ 
                println!("{:?}", errors);
                panic!("Found errors.");
            }
        }
    }

    #[test]
    fn test_emptylines_multitab() {
        let input = "\t\t
application:
    type is Web
    dashboard is Dash
    layout is SinglePage
    roles -> User, SuperUser, Admin
    panels -> Map, Pie, XY, TS, Bar

Map:
    label is map
    type is geomap
    source is first
    data -> location, stationName, O3, NO2, SO2, address
";

        let input_span = Span::new(input);
        let result = lexer(input_span);

        match result {
            Ok(tokens) => {
                assert_eq!(tokens.len(), 40);
            },
            Err(errors) =>{ 
                println!("{:?}", errors);
                panic!("Found errors.");
            }
        }
    }

    #[test]
    fn test_emptylines_mixed() {
        let input = "\t   \t
application:
    type is Web
    dashboard is Dash
    layout is SinglePage
    roles -> User, SuperUser, Admin
    panels -> Map, Pie, XY, TS, Bar

Map:
    label is map
    type is geomap
    source is first
    data -> location, stationName, O3, NO2, SO2, address
";

        let input_span = Span::new(input);
        let result = lexer(input_span);

        match result {
            Ok(tokens) => {
                assert_eq!(tokens.len(), 40);
            },
            Err(errors) =>{ 
                println!("{:?}", errors);
                panic!("Found errors.");
            }
        }
    }

    #[test]
    fn test_emptylines_between_blocks() {
        //has one space in the betweenblock files (DO NOT REFORMAT)
        let input = "\t\t
application:
    type is Web
    dashboard is Dash
    layout is SinglePage
    roles -> User, SuperUser, Admin
    panels -> Map, Pie, XY, TS, Bar
 

Map:
    label is map

    type is geomap
    source is first
    data -> location, stationName, O3, NO2, SO2, address
";

        let input_span = Span::new(input);
        let result = lexer(input_span);

        match result {
            Ok(tokens) => {
                assert_eq!(tokens.len(), 41);
            },
            Err(errors) =>{ 
                println!("{:?}", errors);
                panic!("Found errors.");
            }
        }
    }
}

