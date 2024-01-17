use crate::{errors::SagError, sections::Config};
use nom::error::context;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alpha1, alphanumeric1, space0},
    combinator::{map_res, recognize, verify},
    multi::{many1_count, separated_list0},
    sequence::{pair, terminated},
    Err,
};
use std::borrow::BorrowMut;
use std::collections::HashMap;

// pub type Block<'a> = HashMap<&'a str, Line<'a>>;
pub type Blocks<'a> = HashMap<&'a str, Value<'a>>;

pub type IResult<'a, O> = nom::IResult<&'a str, O>;

const INDENT: usize = 4;

#[derive(Debug)]
pub enum Value<'a> {
    String(&'a str),
    Vec(Vec<&'a str>),
    Block(HashMap<&'a str, Value<'a>>),
}

impl<'a> TryInto<&'a str> for &Value<'a> {
    type Error = &'a str;
    fn try_into(self) -> Result<&'a str, Self::Error> {
        match self {
            Value::String(value) => Ok(value),
            _ => Err("value is specified in a wrong format"),
        }
    }
}

impl<'a> TryInto<Vec<&'a str>> for &Value<'a> {
    type Error = &'a str;
    fn try_into(self) -> Result<Vec<&'a str>, Self::Error> {
        match self {
            Value::Vec(value) => Ok(value.to_vec()),
            _ => Err("value is specified in a wrong format"),
        }
    }
}

impl<'a> TryInto<i32> for &Value<'a> {
    type Error = &'a str;
    fn try_into(self) -> Result<i32, Self::Error> {
        match self {
            Value::String(value) => {
                let value = value.parse::<i32>();
                match value {
                    Ok(value) => Ok(value),
                    Err(_) => Err("value is specified in a wrong format"),
                }
            }
            _ => Err("value is specified in a wrong format"),
        }
    }
}

fn parse_name(input: &str) -> IResult<&str> {
    let (input, result) = recognize(many1_count(alt((alpha1, tag("_")))))(input)?;
    Ok((input, result))
}

fn parse_section_name(input: &str) -> IResult<&str> {
    let (input, result) = terminated(alpha1, tag(":"))(input)?;
    Ok((input, result))
}

fn parse_vec(input: &str) -> IResult<Value> {
    let (input, items) = separated_list0(alt((tag(", "), tag(","))), alphanumeric1)(input)?;

    Ok((input, Value::Vec(items)))
}

fn parse_line(input: &str) -> IResult<(&str, Value)> {
    let (input, (name, separator)) = pair(parse_name, alt((tag(" is "), tag(" -> "))))(input)?;
    match separator {
        " is " => Ok(("", (name, Value::String(input)))),
        " -> " => {
            let (input, value) = parse_vec(input)?;
            Ok((input, (name, value)))
        }
        _ => unreachable!(),
    }
}

fn parse_indent(line: &str) -> IResult<usize> {
    // let (line, spaces) = take_till(|c| c != ' ')(line)?;

    context(
        "Invalid indentation",
        map_res(
            verify(space0, |s: &str| s.len() % INDENT == 0),
            |s: &str| Ok::<_, nom::error::Error<&str>>(s.len() / INDENT),
        ),
    )(line)
}

pub fn parse_lines(input: &str) -> Result<Blocks, SagError> {
    let lines = input.lines();
    let mut blocks: Blocks = HashMap::new();
    let mut last_blocks: Vec<&str> = Vec::new();

    for (line_n, line) in lines.enumerate() {
        let line_len = line.len();

        let (line, indent_level) = parse_indent(line).map_err(|e| match e {
            Err::Error(e) | Err::Failure(e) => {
                SagError::invalid_char(line_n, line_len - e.input.len())
            }
            _ => unreachable!(),
        })?;

        if line.is_empty() {
            continue;
        }

        for _ in 0..last_blocks.len() - indent_level {
            last_blocks.pop();
        }

        // pasre section name
        if let Ok((_, name)) = parse_section_name(line) {
            let current_block = last_blocks.iter().fold(blocks.borrow_mut(), |b, k| {
                if let Value::Block(b) = b.get_mut(k).expect("Key must be present") {
                    b
                } else {
                    unreachable!()
                }
            });
            current_block.insert(name, Value::Block(HashMap::new()));
            last_blocks.push(name);
            continue;
        }
        // parse indent
        // let line = match multispace0::<_, Error<&str>>(line) {
        //     Ok((input, _)) => input,
        //     Err(e) => {
        //         match e {
        //             Err::Error(e) | Err::Failure(e) => {
        //                 return Err(SagError::invalid_char(line_n, line_len - e.input.len()));
        //             }
        //             _ => unreachable!(),
        //         };
        //     }
        // };

        //parse line in block
        match parse_line(line) {
            Ok((input, (name, line))) => {
                if !input.is_empty() {
                    return Err(SagError::invalid_char(line_n, line_len - input.len()));
                }
                if !last_blocks.is_empty() {
                    let current_block = last_blocks.iter().fold(blocks.borrow_mut(), |b, k| {
                        if let Value::Block(b) = b.get_mut(k).expect("Key must be present") {
                            b
                        } else {
                            unreachable!()
                        }
                    });
                    current_block.borrow_mut().insert(name, line);
                }
            }
            Err(e) => {
                match e {
                    Err::Error(e) | Err::Failure(e) => {
                        return Err(SagError::invalid_char(line_n, line_len - e.input.len()));
                    }
                    _ => unreachable!(),
                };
            }
        };
    }
    dbg!(&blocks);
    Ok(blocks)
}

// pub fn parse_lines(input: &str) -> Result<Blocks, SagError> {
//     let lines = input.lines();
//     let mut blocks: Blocks = HashMap::new();
//     let mut last_block: Option<&str> = None;
//
//     for (line_n, line) in lines.enumerate() {
//         let line_len = line.len();
//
//         // pasre section name
//         if let Ok((_, name)) = parse_section_name(line) {
//             blocks.insert(name, HashMap::new());
//             last_block = Some(name);
//             continue;
//         }
//         // parse indent
//         let line = match multispace0::<_, Error<&str>>(line) {
//             Ok((input, _)) => input,
//             Err(e) => {
//                 match e {
//                     Err::Error(e) | Err::Failure(e) => {
//                         return Err(SagError::invalid_char(line_n, line_len - e.input.len()));
//                     }
//                     _ => unreachable!(),
//                 };
//             }
//         };
//         if line.is_empty() {
//             continue;
//         }
//
//         //parse line in block
//         match parse_line(line) {
//             Ok((input, (name, line))) => {
//                 if !input.is_empty() {
//                     return Err(SagError::invalid_char(line_n, line_len - input.len()));
//                 }
//                 if let Some(block) = last_block {
//                     blocks.get_mut(block).unwrap().insert(name, line);
//                 }
//             }
//             Err(e) => {
//                 match e {
//                     Err::Error(e) | Err::Failure(e) => {
//                         return Err(SagError::invalid_char(line_n, line_len - e.input.len()));
//                     }
//                     _ => unreachable!(),
//                 };
//             }
//         };
//     }
//     Ok(blocks)
// }

pub fn parse_input(input: &str) -> Result<Config<'_>, SagError> {
    let blocks = match parse_lines(input) {
        Ok(blocks) => blocks,
        Err(e) => {
            return Err(e);
        }
    };
    let config = match Config::new(&blocks) {
        Ok(config) => config,
        Err(e) => {
            return Err(e);
        }
    };
    Ok(config)
}
