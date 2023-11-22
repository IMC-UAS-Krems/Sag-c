#![allow(unused)]
#![feature(impl_trait_in_assoc_type)]
#![feature(associated_type_defaults)]

use anyhow::Result;
use core::iter::{
    Map as IMap,
    Peekable,
};
use erased_serde::serialize_trait_object;
use nom::{
    branch::alt,
    bytes::streaming::{
        tag,
        take_till1,
        take_until1,
        take_while1,
    },
    character::complete::{
        alphanumeric1,
        digit1,
    },
    combinator::{
        map,
        map_res,
        opt,
    },
    error::Error as NomErr,
    multi::separated_list1,
    sequence::{
        delimited,
        tuple,
    },
    Err,
    IResult,
};
use serde::Serialize;
use set_field::SetField;
use std::{
    cell::RefCell,
    default,
    fmt::Debug,
    process::Output,
    rc::Rc,
    str::FromStr,
    sync::Arc,
};
use strum_macros::{
    Display,
    EnumString,
};
use url::Url;

type ParseResult<'c, O> = IResult<&'c str, O>;

trait Parse<'c>: Sized {
    type Output = Self;

    fn parse(input: &'c str) -> ParseResult<'c, Self::Output>;
}

trait ParseEnum: FromStr {}

impl<'c, T> Parse<'c> for T
where
    T: ParseEnum,
{
    fn parse(input: &'c str) -> ParseResult<Self::Output> {
        map_res(alphanumeric1, Self::from_str)(input)
    }
}

#[derive(Debug)]
struct Block<'c> {
    lines: Rc<[&'c str]>,
}

impl<'c> Block<'c> {
    fn new(lines: impl Iterator<Item = &'c str>) -> Self {
        Self {
            lines: Rc::from(lines.collect::<Vec<_>>()),
        }
    }

    fn joined(self) -> String {
        self.lines.join("\n")
    }
}

impl<'c> Default for Block<'c> {
    fn default() -> Self {
        Self::new([].into_iter())
    }
}

struct Parser<'c, const INDENT_SIZE: u8 = 4> {
    line: u8,
    column: u8,
    content: &'c str,
    indent_size: u8,
}

impl<'c, const I: u8> Parser<'c, I> {
    fn new(content: &'c str) -> Self {
        Self {
            line: 1,
            column: 1,
            content,
            indent_size: I,
        }
    }

    fn lines(&self) -> impl Iterator<Item = (&'c str, u8)> {
        self.content
            .lines()
            // .filter(|line| !line.is_empty())
            .map(|line| {
                // Count how many times the whitespace char ' ' appears at the beginning of the line
                line.chars()
                    .take_while(|c| *c == ' ')
                    .fold((line, 0), |(line, indent), _| (line, (indent + 1)))
            })
    }

    fn next_block<T>(iter: &mut Peekable<T>) -> Option<(Block<'c>, Rc<RefCell<&mut Peekable<T>>>)>
    where
        T: Iterator<Item = (&'c str, u8)> + Sized,
    {
        let indent = if let Some((_, indent)) = iter.peek() {
            Some(*indent / I)
        } else {
            return None;
        };

        let indent = indent.expect("Can safely unwrap");

        let mut lines = Vec::<&'c str>::default();

        lines.push(iter.next().unwrap().0);

        loop {
            if let None = iter.peek() {
                break;
            }

            let (content, indentation) = iter.next().unwrap();

            if (indentation / I) <= indent {
                break;
            }

            lines.push(content);
        }

        Some((Block::new(lines.into_iter()), Rc::from(RefCell::from(iter))))
    }

    fn blocks(&self) -> Vec<Block<'c>> {
        let mut lines = self.lines().peekable();
        // let mut lines_ref = Rc::from(&mut lines);
        let mut curr_block = None;
        let mut blocks = vec![];
        // let mut its = vec![];

        loop {
            curr_block = Self::next_block(&mut lines);

            match curr_block {
                None => break,
                Some((block, rest)) => {
                    blocks.push(block);
                    // let f = rest;
                    // let rest = rest.clone();
                    println!("{:#?}", rest.borrow_mut().collect::<Vec<_>>());
                    // its.push(Rc::clone(&rest));
                }
            }
        }
        blocks

        // todo!()
    }
}

// ! HERE STARTS THE PARSER CODE

#[derive(Debug, Serialize)]
struct Str(Rc<str>);

impl Str {
    fn new(s: &str) -> Self {
        Self(Rc::from(s))
    }
}

impl Default for Str {
    fn default() -> Self {
        Self(Rc::from(""))
    }
}

/// Semantic version
#[derive(Debug, Default, Serialize, SetField)]
struct Semantic {
    major: u8,
    minor: u8,
    patch: u8,
}

impl<'c> Parse<'c> for Semantic {
    fn parse(input: &'c str) -> ParseResult<'c, Self::Output> {
        let (input, (major, _, minor, _, patch)) = tuple((
            map_res(digit1, str::parse::<u8>),
            tag("."),
            map_res(digit1, str::parse::<u8>),
            tag("."),
            map_res(digit1, str::parse::<u8>),
        ))(input)?;

        Ok((
            input,
            Semantic {
                major,
                minor,
                patch,
            },
        ))
    }
}

/// Version
/// Simple version: 1, 2, 3, ...
/// Semantic version: 1.2.3
#[derive(Debug, Serialize, EnumString, Display)]
enum Version {
    Simple(u8),
    Semantic(Semantic),
}

impl Default for Version {
    fn default() -> Self {
        Version::Simple(0)
    }
}

impl<'c> Parse<'c> for Version {
    fn parse(input: &'c str) -> ParseResult<'c, Self::Output> {
        match Semantic::parse(input) {
            Ok((input, semantic)) => Ok((input, Version::Semantic(semantic))),
            Err(_) => {
                let (input, simple) = map_res(digit1, str::parse::<u8>)(input)?;
                Ok((input, Version::Simple(simple)))
            }
        }
    }
}

/// Scope
/// Any of the allowed scopes for a service
#[derive(Debug, Serialize, EnumString, Display, Default)]
enum Scope {
    #[default]
    Service,
    Industry,
    Manifacturing,
    Education,
    Healthcare,
    SocialPrograms,
    Government,
    Energy,
    Water,
    Environment,
    Transportation,
    Communication,
    PublicSafety,
    UrbanPlanning,
    Infrastructure,
}

impl ParseEnum for Scope {}

/// Service Section
///
/// - name: name of the service
/// - version: version of the service
/// - scope: scope of the service
#[derive(Debug, Serialize, SetField, Default)]
struct ServiceSection {
    name: Str,
    version: Version,
    scope: Scope,
}

/// Source Type
#[derive(Debug, Serialize, EnumString, Display, Default)]
enum SourceType {
    #[default]
    SmartMeter,
}

impl ParseEnum for SourceType {}

/// IoT Provider
#[derive(Debug, Serialize, EnumString, Display, Default)]
enum Provider {
    #[default]
    Fiware,
}

impl ParseEnum for Provider {}

/// URL
#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Serialize)]
struct URL(Url);

impl Default for URL {
    fn default() -> Self {
        URL(Url::parse("https://fiware.org").unwrap())
    }
}

impl<'c> Parse<'c> for URL {
    fn parse(input: &'c str) -> ParseResult<'c, Self::Output> {
        let (input, url) = map_res(alphanumeric1, Url::parse)(input)?;
        Ok((input, URL(url)))
    }
}

// TODO! PARSE

// MAKE VEC PARSE

impl<'c, T> Parse<'c> for Vec<T>
where
    T: Parse<'c, Output = T>,
{
    fn parse(input: &'c str) -> ParseResult<'c, Self::Output> {
        let (input, items) = delimited(
            opt(tag(" ")),
            separated_list1(tag(", "), T::parse),
            opt(tag(" ")),
        )(input)?;

        Ok((input, items))
    }
}

// MAKE ASSIGNMENT PARSE

/// Query
///
/// Represents how data is transformed from the source
#[derive(Debug, Serialize, Default)]
struct Query(Str);

impl<'c> Query {
    fn new(content: &'c str) -> Self {
        Self(Str::new(content))
    }
}

impl<'c> Parse<'c> for Query {
    fn parse(input: &'c str) -> ParseResult<'c, Self::Output> {
        let (input, query) = take_till1(|c| c == ' ' || c == '\n')(input)?;
        Ok((input, Query(Str::new(query))))
    }
}

/// Source
///
/// - name: name of the source
/// - r#type: type of the source
/// - provider: provider of the source
/// - url: url of the source
/// - query: (optional) query that transforms the data
#[derive(Debug, Serialize, SetField, Default)]
struct Source {
    name: Str,
    r#type: SourceType,
    provider: Provider,
    url: URL,
    query: Option<Query>,
}

/// Data Section
///
/// - sources: list of sources
#[derive(Debug, Serialize, SetField, Default)]
struct DataSection {
    sources: Vec<Source>,
}

/// Application Type
#[derive(Debug, Serialize, EnumString, Display, Default)]
enum ApplicationType {
    #[default]
    Web,
    Mobile,
    Desktop,
    Server,
}

impl ParseEnum for ApplicationType {}

/// Layout
#[derive(Debug, Serialize, EnumString, Display, Default)]
enum Layout {
    #[default]
    SinglePage,
    Horizontal,
    Vertical,
}

impl ParseEnum for Layout {}

/// Defines the options for a visualization
trait Opt: Debug + erased_serde::Serialize {}

serialize_trait_object!(Opt);

/// A Visualization Struct
trait Viz {
    type Option: Opt;
}

/// Options for a Table Graph
#[derive(Debug, Default, Serialize)]
struct TableOpt;

impl Opt for TableOpt {}

/// Table Graph
#[derive(Debug, Default, Serialize)]
struct Table;

impl Viz for Table {
    type Option = TableOpt;
}

/// Options for a Map Graph
#[derive(Debug, Default, Serialize)]
struct MapOpt;

impl Opt for MapOpt {}

/// Map Graph
#[derive(Debug, Default, Serialize)]
struct Map;

impl Viz for Map {
    type Option = MapOpt;
}

/// Options for a Chart Graph
#[derive(Debug, Default, Serialize)]
struct ChartOpt;

impl Opt for ChartOpt {}

/// Chart Graph
#[derive(Debug, Default, Serialize)]
struct Chart;

impl Viz for Chart {
    type Option = ChartOpt;
}

/// Wraps a Visualization
#[derive(Debug, Serialize)]
enum VizType<T, U, V>
where
    T: Viz,
    U: Viz,
    V: Viz,
{
    Table(T),
    Map(U),
    Chart(V),
}

impl Default for VizType<Table, Map, Chart> {
    fn default() -> Self {
        Self::Table(Table)
    }
}

/// Visualization
#[derive(Debug, Serialize, SetField, Default)]
struct Visualization {
    name: Str,
    r#type: VizType<Table, Map, Chart>,
    source: Vec<Str>,
}

/// Application Section
#[derive(Debug, Serialize, SetField, Default)]
struct ApplicationSection {
    r#type: ApplicationType,
    layout: Layout,
    visualizations: Vec<Visualization>,
}

/// Environment Type
#[derive(Debug, Serialize, EnumString, Display, Default)]
enum EnvironmentType {
    Production,
    Development,
    #[default]
    Test,
}

impl ParseEnum for EnvironmentType {}

/// Port
#[derive(Debug, Serialize)]
struct Port(u32);

impl Default for Port {
    fn default() -> Self {
        Self(8080)
    }
}

impl Parse<'_> for Port {
    fn parse(input: &str) -> ParseResult<Self> {
        let (input, port) = map_res(digit1, str::parse::<u32>)(input)?;
        Ok((input, Port(port)))
    }
}

/// Environment
#[derive(Debug, Serialize, SetField, Default)]
struct Environment {
    name: Str,
    r#type: EnvironmentType,
    port: Port,
}

/// Deployment Section
#[derive(Debug, Serialize, SetField, Default)]
struct DeploymentSection {
    environments: Vec<Environment>,
}

/// Source File
#[derive(Debug, Serialize, SetField, Default)]
struct SourceFile {
    service: ServiceSection,
    data: DataSection,
    application: ApplicationSection,
    deployment: DeploymentSection,
}

// ! HERE ENDS THE PARSER CODE

fn main() {
    // type P = Parser<'static, 4>;
    //
    // let parser = P::new(include_str!("../grammars/ssd.improved"));
    //
    // for block in parser.blocks() {
    //     println!("Block");
    //     println!("{:#?}", block);
    //
    //     println!("Lines");
    //     for line in block.lines.iter() {
    //         println!("{}", line);
    //     }
    // }

    // let s = Version::parse("1.2.3");
    //
    // match s {
    //     Ok(s) => println!("{:#?}", s),
    //     Err(e) => println!("{:#?}", e),
    // }

    let (input, result): (&str, Vec<Port>) = Vec::parse("8080, 8081, 8082 ").unwrap();

    println!(
        "Rest: {:#?}\nPorts: {:#?}",
        input,
        result.iter().map(|p| p.0).collect::<Vec<_>>()
    );
}
