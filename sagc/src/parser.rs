use crate::{errors::SagError, sections::Config};
use nom::bytes::complete::{tag, take_until, take_while};
use nom::character::complete::space1;
use nom::error::context;
use nom::{
    branch::alt,
    character::complete::{alpha1, alphanumeric1, space0},
    combinator::{map_res, recognize, verify},
    multi::{many1_count, separated_list0},
    sequence::terminated,
};
use nom_locate::LocatedSpan;
use std::borrow::BorrowMut;
use std::collections::HashMap;

type IResult<'a> = nom::IResult<Span<'a>, Token<'a>>;
type IResultVec<'a> = nom::IResult<Span<'a>, Vec<Token<'a>>>;
type Span<'a> = LocatedSpan<&'a str>;
pub type Blocks<'a> = HashMap<&'a str, ParseResult<'a>>;

const INDENT: usize = 4;

#[derive(Debug, Copy, Clone)]
pub struct Position {
    pub row_start: usize,
    pub row_end: usize,
    pub col_start: usize,
    pub col_end: usize,
}

#[derive(Debug)]
pub enum TokenValue<'a> {
    Indent(usize),
    Block(&'a str),
    Section(&'a str),
    ValueStr(&'a str),
    ValueVec(Vec<&'a str>),
    Is,
    Arrow,
    IndentError,
    UnparsableError,
}

#[derive(Debug)]
struct Token<'a> {
    position: Position,
    value: TokenValue<'a>,
}

#[derive(Debug)]
pub struct ParseResult<'a> {
    pub position: Position,
    pub value: Value<'a>,
}

impl ParseResult<'_> {
    fn new(position: Position, value: Value<'_>) -> ParseResult<'_> {
        ParseResult { position, value }
    }
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

impl<'a> Value<'a> {
    pub fn is_block(&self) -> bool {
        matches!(self, Value::Block(_))
    }
}
#[derive(Debug)]
pub enum Value<'a> {
    String(&'a str),
    Vec(Vec<&'a str>),
    Block(HashMap<&'a str, ParseResult<'a>>),
}

fn parse_block_name(input: Span) -> IResult {
    let line = input.location_line() as usize;
    let col_start = input.get_column();

    terminated(alpha1, tag(":"))(input).map(|(input, result)| {
        let (input, _) =
            take_while::<_, nom_locate::LocatedSpan<&str>, ()>(|c| c == '\n')(input).unwrap();

        let position = Position {
            row_start: line,
            row_end: line,
            col_start,
            col_end: col_start + result.len(),
        };
        (
            input,
            Token {
                position,
                value: TokenValue::Block(result.fragment()),
            },
        )
    })
}

fn parse_section_name(input: Span) -> IResult {
    let line = input.location_line() as usize;
    let col_start = input.get_column();

    recognize(many1_count(alt((alpha1, tag("_")))))(input).map(|(input, result)| {
        let position = Position {
            row_start: line,
            row_end: line,
            col_start,
            col_end: col_start + result.len(),
        };
        (
            input,
            Token {
                position,
                value: TokenValue::Section(result.fragment()),
            },
        )
    })
}

fn parse_vec(input: Span) -> IResult {
    let line = input.location_line() as usize;
    let col_start = input.get_column();

    separated_list0(
        alt((tag(", "), tag(","))),
        recognize(many1_count(alt((alphanumeric1, space1, tag("."))))),
    )(input)
    .map(|(input, result)| {
        let position = Position {
            row_start: line,
            row_end: line,
            col_start,
            col_end: col_start + result.len(),
        };
        (
            input,
            Token {
                position,
                value: TokenValue::ValueVec(
                    result.iter().map(|s| *s.fragment()).collect::<Vec<&str>>(),
                ),
            },
        )
    })
}

fn parse_value(input: Span) -> IResult {
    let line = input.location_line() as usize;
    let col_start = input.get_column();

    take_until("\n")(input).map(|(input, result)| {
        let position = Position {
            row_start: line,
            row_end: line,
            col_start,
            col_end: col_start + result.len(),
        };
        (
            input,
            Token {
                position,
                value: TokenValue::ValueStr(result.fragment().trim_end()),
            },
        )
    })
}

fn parse_separator(input: Span) -> IResult {
    let line = input.location_line() as usize;
    let col_start = input.get_column();

    alt((tag(" -> "), tag(" is ")))(input).map(|(input, result)| {
        let position = Position {
            row_start: line,
            row_end: line,
            col_start,
            col_end: col_start + result.len(),
        };
        (
            input,
            Token {
                position,
                value: if *result.fragment() == " -> " {
                    TokenValue::Arrow
                } else {
                    TokenValue::Is
                },
            },
        )
    })
}

fn parse_section_line(input: Span) -> IResultVec {
    let mut to_return = Vec::new();
    let (input, result) = parse_section_name(input)?;
    to_return.push(result);

    let (input, result) = parse_separator(input)?;
    to_return.push(result);

    let sep = to_return.last().unwrap();
    let (input, result) = if let TokenValue::Arrow = sep.value {
        parse_vec(input)?
    } else {
        parse_value(input)?
    };
    to_return.push(result);

    let (input, _) =
        take_while::<_, nom_locate::LocatedSpan<&str>, ()>(|c| c == '\n')(input).unwrap();

    Ok((input, to_return))
}

fn handle_error<'a>(input: Span<'a>, error: TokenValue<'a>) -> IResult<'a> {
    match error {
        TokenValue::IndentError | TokenValue::UnparsableError => (),
        _ => unreachable!("No, no, no... Do not do this"),
    }

    let line = input.location_line() as usize;
    let col_start = input.get_column();

    take_until("\n")(input).map(|(input, result)| {
        let (input, _) =
            take_while::<_, nom_locate::LocatedSpan<&str>, ()>(|c| c == '\n')(input).unwrap();

        let position = Position {
            row_start: line,
            row_end: line,
            col_start,
            col_end: col_start + result.len(),
        };
        (
            input,
            Token {
                position,
                value: error,
            },
        )
    })
}

fn parse_indent(input: Span) -> IResult {
    // let (line, spaces) = take_till(|c| c != ' ')(line)?;
    let line = input.location_line();
    let column = input.get_column();

    context(
        "Invalid indentation",
        map_res(
            verify(space0, |s: &Span| s.len() % INDENT == 0),
            |s: Span| Ok::<_, nom::error::Error<&str>>(s.len() / INDENT),
        ),
    )(input)
    .map(|(input, result)| {
        (
            input,
            Token {
                position: Position {
                    row_start: line as usize,
                    row_end: line as usize,
                    col_start: column,
                    col_end: column + result * INDENT,
                },
                value: TokenValue::Indent(result),
            },
        )
    })
}

fn lexer(input: Span) -> Result<Vec<Token>, Vec<Token>> {
    let mut input = input;
    let mut tokens = Vec::new();
    let mut errors = Vec::new();
    loop {
        let result = parse_indent(input);
        match result {
            Ok((i, token)) => {
                input = i;
                tokens.push(token);
            }
            Err(_) => {
                let (i, token) = handle_error(input, TokenValue::IndentError).unwrap();
                input = i;
                errors.push(token);
                continue;
            }
        }
        if let Ok((i, result)) = parse_block_name(input) {
            input = i;
            tokens.push(result);
            if input.is_empty() {
                break;
            }
            continue;
        }
        let result = parse_section_line(input);
        match result {
            Ok((i, token)) => {
                input = i;
                tokens.extend(token);
            }
            Err(_) => {
                let (i, token) = handle_error(input, TokenValue::UnparsableError).unwrap();
                input = i;
                errors.push(token);
            }
        }
        if input.is_empty() {
            break;
        }
    }
    if errors.is_empty() {
        Ok(tokens)
    } else {
        Err(errors)
    }
}

fn tokens_to_blocks(tokens: Vec<Token>) -> Blocks {
    let mut blocks: HashMap<&str, ParseResult> = HashMap::new();
    let mut last_blocks = Vec::new();
    let mut tokens = tokens.iter();

    while tokens.len() > 0 {
        let token = tokens.next().unwrap();
        match token.value {
            TokenValue::Indent(indent) => {
                for _ in 0..last_blocks.len() - indent {
                    last_blocks.pop();
                }
            }
            TokenValue::Block(name) => {
                let current_block = last_blocks.iter().fold(blocks.borrow_mut(), |b, k| {
                    if let Value::Block(b) = b
                        .get_mut(k)
                        .expect("Key must be present")
                        .value
                        .borrow_mut()
                    {
                        b
                    } else {
                        unreachable!()
                    }
                });
                current_block.insert(
                    name,
                    ParseResult::new(token.position, Value::Block(HashMap::new())),
                );
                last_blocks.push(name);
                continue;
            }
            TokenValue::Section(name) => {
                let current_block = last_blocks.iter().fold(blocks.borrow_mut(), |b, k| {
                    if let Value::Block(b) = b
                        .get_mut(k)
                        .expect("Key must be present")
                        .value
                        .borrow_mut()
                    {
                        b
                    } else {
                        unreachable!()
                    }
                });
                let sep = tokens.next().unwrap();
                match sep.value {
                    TokenValue::Arrow => {
                        let token = tokens.next().unwrap();
                        let value = match &token.value {
                            TokenValue::ValueVec(value) => value,
                            _ => unreachable!(),
                        };
                        current_block.insert(
                            name,
                            ParseResult::new(token.position, Value::Vec(value.to_vec())),
                        );
                    }
                    TokenValue::Is => {
                        let token = tokens.next().unwrap();
                        let value = match token.value {
                            TokenValue::ValueStr(value) => value,
                            _ => unreachable!(),
                        };
                        current_block
                            .insert(name, ParseResult::new(token.position, Value::String(value)));
                    }
                    _ => unreachable!(),
                }
            }
            _ => (),
        }
    }
    blocks
}

pub fn parse_lines(input: &str) -> Result<Blocks, Vec<SagError>> {
    let input = Span::new(input);
    let result = lexer(input);
    match result {
        Ok(tokens) => {
            let blocks = tokens_to_blocks(tokens);
            Ok(blocks)
        }
        Err(errors) => {
            let errors = errors
                .iter()
                .map(|e| match e.value {
                    TokenValue::IndentError => SagError::invalid_indentation(e.position),
                    TokenValue::UnparsableError => SagError::unparsable(e.position),
                    _ => unreachable!(),
                })
                .collect();
            Err(errors)
        }
    }
}

// pub type Block<'a> = HashMap<&'a str, Line<'a>>;

pub fn parse_input(input: &str) -> Result<Config<'_>, Vec<SagError>> {
    let blocks = match parse_lines(input) {
        Ok(blocks) => blocks,
        Err(e) => {
            return Err(e);
        }
    };
    let config = match Config::new(&blocks) {
        Ok(config) => config,
        Err(e) => {
            return Err(vec![e]);
        }
    };
    Ok(config)
}
