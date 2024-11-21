use crate::{errors::SagError, sections::Config};
use nom::bytes::complete::{tag, take_while};
use nom::character::complete::{line_ending, space0, space1};
use nom::error::context;
use nom::multi::many0;
use nom::sequence::delimited;
use nom::{
    branch::alt,
    combinator::{map_res, recognize, verify},
    multi::{many1_count, separated_list0},
    sequence::terminated,
};
use nom_locate::LocatedSpan;
use nom_unicode::complete::{alpha1, alphanumeric1};
use std::borrow::BorrowMut;
use std::collections::HashMap;

type IResult<'a> = nom::IResult<Span<'a>, Token<'a>>;
type IResultVec<'a> = nom::IResult<Span<'a>, Vec<Token<'a>>>;
pub type Span<'a> = LocatedSpan<&'a str>;
//pub type Blocks<'a> = HashMap<&'a str, ParseResult<'a>>;

#[derive(Debug)]
pub struct Blocks<'a> {
    pub blocks: HashMap<&'a str, ParseResult<'a>>,
}

impl<'a> Blocks<'a> {
    pub fn new() -> Self {
        Blocks {
            blocks: HashMap::new(),
        }
    }
    pub fn get_mut(&mut self, key: &str) -> Option<&mut ParseResult<'a>> {
        let v = self.blocks.get_mut(key);
        if let Some(v) = v {
            v.mark_accessed();
            return Some(v);
        }
        None
    }

    pub fn get(&mut self, key: &str) -> Option<&ParseResult<'a>> {
        let v = self.blocks.get_mut(key);
        if let Some(v) = v {
            v.mark_accessed();
            return Some(&*v);
        }
        None
    }

    pub fn get_mut_without_mark(&mut self, key: &str) -> Option<&mut ParseResult<'a>> {
        let v = self.blocks.get_mut(key);
        if let Some(v) = v {
            return Some(v);
        }
        None
    }

    pub fn get_without_mark(&mut self, key: &str) -> Option<&ParseResult<'a>> {
        let v = self.blocks.get_mut(key);
        if let Some(v) = v {
            return Some(&*v);
        }
        None
    }

    pub fn insert(&mut self, key: &'a str, value: ParseResult<'a>) {
        self.blocks.insert(key, value);
    }

    pub fn iter(&self) -> std::collections::hash_map::Iter<&'a str, ParseResult<'a>> {
        self.blocks.iter()
    }

    pub fn iter_mut(&mut self) -> std::collections::hash_map::IterMut<&'a str, ParseResult<'a>> {
        self.blocks.iter_mut()
    }
}

impl<'a> Default for Blocks<'a> {
    fn default() -> Self {
        Self::new()
    }
}

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
pub struct Token<'a> {
    pub position: Position,
    pub value: TokenValue<'a>,
}

#[derive(Debug)]
pub struct ParseResult<'a> {
    pub position: Position, // NOTE: in case of a block, position is the position of the block name
    pub value: Value<'a>,
    accessed: bool,
}

impl ParseResult<'_> {
    pub fn new(position: Position, value: Value<'_>) -> ParseResult<'_> {
        ParseResult {
            position,
            value,
            accessed: false,
        }
    }

    pub fn mark_accessed(&mut self) {
        self.accessed = true;
    }

    pub fn was_accessed(self) -> bool {
        self.accessed
    }
}

#[derive(Debug)]
pub enum Value<'a> {
    String(&'a str),
    Vec(Vec<&'a str>),
    Block(Blocks<'a>),
}

impl<'a> Value<'a> {
    pub fn is_block(&self) -> bool {
        matches!(self, Value::Block(_))
    }
}

impl<'a> TryInto<&'a str> for &mut Value<'a> {
    type Error = &'a str;
    fn try_into(self) -> Result<&'a str, Self::Error> {
        match self {
            Value::String(value) => Ok(value),
            _ => Err("value is specified in a wrong format"),
        }
    }
}

impl<'a> TryInto<usize> for &mut Value<'a> {
    type Error = &'a str;
    fn try_into(self) -> Result<usize, Self::Error> {
        match self {
            Value::String(value) => value
                .parse()
                .map_err(|_| "value is specified in a wrong format"),
            _ => Err("value is specified in a wrong format"),
        }
    }
}

impl<'a> TryInto<Vec<&'a str>> for &mut Value<'a> {
    type Error = &'a str;
    fn try_into(self) -> Result<Vec<&'a str>, Self::Error> {
        match self {
            Value::Vec(value) => Ok(value.to_vec()),
            _ => Err("value is specified in a wrong format"),
        }
    }
}

impl<'a> TryInto<HashMap<usize, &'a str>> for &mut Value<'a> {
    type Error = &'a str;
    fn try_into(self) -> Result<HashMap<usize, &'a str>, Self::Error> {
        match self {
            Value::Block(value) => {
                let mut new_value = HashMap::new();
                for (k, v) in value.iter_mut() {
                    v.mark_accessed();
                    let number = k
                        .parse::<usize>()
                        .map_err(|_| "value is specified in a wrong format")?;
                    let v = &mut v.value;
                    let string: &'a str = v.try_into()?;
                    new_value.insert(number, string);
                }
                Ok(new_value)
            }
            _ => Err("value is specified in a wrong format"),
        }
    }
}

/// Handle error in the lexer. Error is everythig from error location to the end of the line
pub fn handle_error<'a>(input: Span<'a>, error: TokenValue<'a>) -> IResult<'a> {
    match error {
        TokenValue::IndentError | TokenValue::UnparsableError => (),
        _ => unreachable!("No, no, no... Do not do this"),
    }

    let line = input.location_line() as usize;
    let col_start = input.get_column();

    take_while(|c| !(c == '\r' || c == '\n'))(input).map(|(input, result)| {
        let (input, _) =
            take_while::<_, nom_locate::LocatedSpan<&str>, nom::error::Error<Span>>(|c| {
                c == '\n' || c == '\r'
            })(input)
            .unwrap();

        let position = Position {
            row_start: line,
            row_end: line,
            col_start,
            col_end: col_start + result.len() - 1,
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

pub fn parse_block_name(input: Span) -> IResult {
    let line = input.location_line() as usize;
    let col_start = input.get_column();

    terminated(alpha1, tag(":"))(input).map(|(input, result)| {
        let (input, _) =
            take_while::<_, nom_locate::LocatedSpan<&str>, nom::error::Error<Span>>(|c| {
                c == '\n' || c == '\r'
            })(input)
            .unwrap();

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

    recognize(many1_count(alt((alphanumeric1, tag("_")))))(input).map(|(input, result)| {
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

fn parse_separator(input: Span) -> IResult {
    let line = input.location_line() as usize;
    let col_start = input.get_column();

    alt((
        delimited(space1, tag("->"), space1),
        delimited(space1, tag("is"), space1),
    ))(input)
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
                value: if *result.fragment() == "->" {
                    TokenValue::Arrow
                } else {
                    TokenValue::Is
                },
            },
        )
    })
}

fn parse_vec(input: Span) -> IResult {
    let line = input.location_line() as usize;
    let col_start = input.get_column();

    separated_list0(
        alt((tag(", "), tag(","))),
        recognize(many1_count(alt((
            alphanumeric1,
            space1, // NOTE: this gives error on the separator `, `
            tag("."),
            tag("_"),
        )))),
    )(input)
    .map(|(input, result)| {
        let position = Position {
            row_start: line,
            row_end: line,
            col_start,
            col_end: col_start + result.iter().fold(0, |acc, s| acc + s.len()),
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

    take_while(|c| !(c == '\r' || c == '\n'))(input).map(|(input, result)| {
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

    let (input, _) = take_while::<_, nom_locate::LocatedSpan<&str>, nom::error::Error<Span>>(|c| {
        c == '\r' || c == '\n'
    })(input)
    .unwrap();

    Ok((input, to_return))
}

fn parse_indent(input: Span) -> IResult {
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

fn seek_to_input(input: Span) -> Span {
    let result = many0::<_, _, nom::error::Error<Span>, _>(alt((space1, line_ending)))(input);
    match result {
        Ok((input, _)) => input,
        Err(_) => input,
    }
}

fn parse_whitespace_line(input: Span) -> nom::IResult<Span, Span> {
    let result = take_while::<_, nom_locate::LocatedSpan<&str>, nom::error::Error<Span>>(|c| {
        c == ' ' || c == '\t'
    })(input);

    match result {
        Ok((input, _)) => line_ending(input),
        Err(_) => line_ending(input),
    }
}

pub fn lexer(input: Span) -> Result<Vec<Token>, Vec<Token>> {
    let mut input = input;
    let mut tokens = Vec::new();
    let mut errors = Vec::new();
    let mut last_indent = 0;
    let mut last_block_indent = 0;
    // first indent in any section should be `last_block_indent` + 1
    let mut first_field_indent = true;

    // input = seek_to_input(input);

    loop {
        let result = parse_indent(input);

        // if error, skip the line and continue
        match result {
            Ok((i, token)) => {
                input = i;
                last_indent = match token.value {
                    TokenValue::Indent(indent) => indent,
                    _ => unreachable!(),
                };
                tokens.push(token);

                if let Ok((i, _)) = parse_whitespace_line(input) {
                    input = i;
                    continue;
                }
            }
            Err(_) => {
                //check if the line only consists of spaces or tabs
                let white_parse = parse_whitespace_line(input);
                match white_parse {
                    //if yes continue as usual
                    Ok((i, _)) => {
                        input = i;
                        continue;
                    }
                    //if not push an error
                    Err(_) => {
                        let (i, token) = handle_error(input, TokenValue::IndentError).unwrap();
                        input = i;
                        errors.push(token);
                        continue;
                    }
                }
            }
        }

        // if error, try to parse section line (like `alt` in nom)
        if let Ok((i, result)) = parse_block_name(input) {
            input = i;
            last_block_indent = last_indent;
            first_field_indent = true;
            tokens.push(result);
            if input.is_empty() {
                break;
            }
            continue;
        }
        // final parser, either parse section line or handle error
        let result = parse_section_line(input);
        match result {
            Ok((i, token)) => {
                if last_indent == 0
                    || last_indent - last_block_indent > 1
                    || first_field_indent && last_indent - last_block_indent != 1
                {
                    let (i, token) = handle_error(input, TokenValue::IndentError).unwrap();
                    input = i;
                    errors.push(token);
                    continue;
                }
                first_field_indent = false;
                input = i;
                tokens.extend(token);
            }
            Err(nom::Err::Error(e)) => {
                dbg!(&e.input);
                let (i, token) = handle_error(e.input, TokenValue::UnparsableError).unwrap();
                input = i;
                errors.push(token);
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

/// Convert tokens to HashMap for easy access
pub fn tokens_to_blocks(tokens: Vec<Token>) -> Blocks {
    let mut blocks = Blocks::new();
    // vec to track keys of blocks, e.g ["a", "b", "c"] -> a.b.c
    let mut last_blocks: Vec<&str> = Vec::new();
    let mut tokens = tokens.iter();

    // 3 main cases: indent, block, section
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
                        .get_mut_without_mark(k)
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
                    ParseResult::new(token.position, Value::Block(Blocks::new())),
                );
                last_blocks.push(name);
            }

            TokenValue::Section(name) => {
                let current_block = last_blocks.iter().fold(blocks.borrow_mut(), |b, k| {
                    if let Value::Block(b) = b
                        .get_mut_without_mark(k)
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

fn parse_lines(input: &str) -> Result<Blocks, Vec<SagError>> {
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

fn check_used_fields(blocks: &Blocks<'_>) {
    for (k, v) in blocks.iter() {
        if let Value::Block(b) = &v.value {
            check_used_fields(b);
        }
        if !v.accessed {
            log::warn!("Field {} is not used", k);
        }
    }
}

pub fn parse_input(input: &str) -> Result<Config<'_>, Vec<SagError>> {
    let mut blocks = match parse_lines(input) {
        Ok(blocks) => blocks,
        Err(e) => {
            return Err(e);
        }
    };

    let config = match Config::new(&mut blocks) {
        Ok(config) => config,
        Err(e) => {
            return Err(e
                .into_iter()
                .filter(|e| matches!(e, SagError::LanguageError(_e)))
                .collect());
        }
    };
    //dbg!(&blocks);
    check_used_fields(&blocks);

    Ok(config)
}
