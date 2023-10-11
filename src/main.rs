#![allow(unused)]
#![feature(impl_trait_in_assoc_type)]

use core::iter::{
    Map as IMap,
    Peekable,
};
use erased_serde::serialize_trait_object;
use serde::Serialize;
use std::{
    fmt::Debug,
    rc::Rc,
    sync::Arc,
};
use strum_macros::{
    Display,
    EnumString,
};
use url::Url;

// trait Parse<'c>: Sized {
//     fn parse(input: &'c str) -> Self;
// }

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

    fn next_block<T>(iter: &mut Peekable<T>) -> Option<(Block<'c>, &Peekable<T>)>
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

        Some((Block::new(lines.into_iter()), iter))
    }

    fn blocks(&self) -> Vec<Block<'c>> {
        let mut lines = self.lines().peekable();
        let mut curr_block = None;
        let mut blocks = vec![];
        // let mut its = vec![];

        loop {
            curr_block = Self::next_block(&mut lines);

            match curr_block {
                None => break,
                Some((block, rest)) => {
                    blocks.push(block);
                    let f = rest;
                    // let _rest = rest.clone();
                    // its.push(_rest);
                }
            }
        }
        blocks
    }
}

/// Semantic version
#[derive(Debug, Default, Serialize)]
struct Semantic {
    major: u8,
    minor: u8,
    patch: u8,
}

/// Version
/// Simple version: 1, 2, 3, ...
/// Semantic version: 1.2.3
#[derive(Debug, Serialize, EnumString, Display)]
enum Version {
    Simple(u8),
    Semantic(Semantic),
}

/// Scope
/// Any of the allowed scopes for a service
#[derive(Debug, Serialize, EnumString, Display)]
enum Scope {
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

/// Service Section
///
/// - name: name of the service
/// - version: version of the service
/// - scope: scope of the service
#[derive(Debug, Serialize)]
struct ServiceSection<'c> {
    name: &'c str,
    version: Version,
    scope: Scope,
}

/// Source Type
#[derive(Debug, Serialize, EnumString, Display)]
enum SourceType {
    SmartMeter,
}

/// IoT Provider
#[derive(Debug, Serialize, EnumString, Display)]
enum Provider {
    Fiware,
}

/// URL
#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Serialize)]
struct URL(Url);

/// Query
///
/// Represents how data is transformed from the source
#[derive(Debug, Serialize)]
struct Query<'c>(&'c str);

/// Source
///
/// - name: name of the source
/// - r#type: type of the source
/// - provider: provider of the source
/// - url: url of the source
/// - query: (optional) query that transforms the data
#[derive(Debug, Serialize)]
struct Source<'c> {
    name: &'c str,
    r#type: SourceType,
    provider: Provider,
    url: URL,
    query: Option<Query<'c>>,
}

/// Data Section
///
/// - sources: list of sources
#[derive(Debug, Serialize)]
struct DataSection<'c> {
    #[serde(borrow)]
    sources: Vec<Source<'c>>,
}

/// Application Type
#[derive(Debug, Serialize, EnumString, Display)]
enum ApplicationType {
    Web,
    Mobile,
    Desktop,
    Server,
}

/// Layout
#[derive(Debug, Serialize, EnumString, Display)]
enum Layout {
    SinglePage,
    Horizontal,
    Vertical,
}

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

/// Visualization
#[derive(Debug, Serialize)]
struct Visualization<'c> {
    name: &'c str,
    r#type: VizType<Table, Map, Chart>,
    source: Vec<&'c str>,
    options: Box<dyn Opt>,
}

/// Application Section
#[derive(Debug, Serialize)]
struct ApplicationSection<'c> {
    r#type: ApplicationType,
    layout: Layout,
    visualizations: Vec<Visualization<'c>>,
}

/// Environment Type
#[derive(Debug, Serialize, EnumString, Display)]
enum EnvironmentType {
    Production,
    Development,
    Test,
}

/// Port
#[derive(Debug, Serialize)]
struct Port(u32);

/// Environment
#[derive(Debug, Serialize)]
struct Environment<'c> {
    name: &'c str,
    r#type: EnvironmentType,
    port: Port,
}

/// Deployment Section
#[derive(Debug, Serialize)]
struct DeploymentSection<'c> {
    environments: Vec<Environment<'c>>,
}

/// Source File
#[derive(Debug, Serialize)]
struct SourceFile<'c> {
    service: ServiceSection<'c>,
    data: DataSection<'c>,
    application: ApplicationSection<'c>,
    deployment: DeploymentSection<'c>,
}

fn main() {
    type P = Parser<'static, 4>;

    let parser = P::new(include_str!("../grammars/ssd.improved"));

    for block in parser.blocks() {
        println!("Block");
        println!("{:#?}", block);

        println!("Lines");
        for line in block.lines.iter() {
            println!("{}", line);
        }
    }
}
