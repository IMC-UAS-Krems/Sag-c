use nom::bytes::complete::tag;
use nom::character::complete::i16;
use nom::multi::separated_list1;
use nom::IResult;
use std::collections::HashMap;
use std::str::FromStr;
use url::Url;

use crate::errors::SagError;
use crate::parse;
use crate::parser::Blocks;
use crate::parser::Value;

#[derive(Debug)]
pub struct Config<'a> {
    pub service: Service<'a>,
    /// e.g. `<name>: <datasource>`
    pub data_sources: HashMap<&'a str, Datasource<'a>>,
    pub application: Application<'a>,
    pub deployment: Deployment<'a>,
}

#[derive(Debug)]
pub struct Service<'a> {
    pub title: &'a str, // called "name" for Dash, can be 'name' also in this case
    pub scope: Scope,
    pub version: Version,
    pub test: Option<Test<'a>>,
}

#[derive(Debug)]
pub struct Test<'a> {
    pub one: &'a str,
    pub two: &'a str,
}

#[derive(Debug)]
pub enum Scope {
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

#[derive(Debug)]
pub struct Version {
    pub major: i16,
    pub minor: i16,
    pub patch: i16,
}

#[derive(Debug)]
pub struct Datasource<'a> {
    pub provider: Provider,
    pub r#type: SourceType,
    pub uri: Url,
    pub query: &'a str,
}

#[derive(Debug)]
pub enum Provider {
    Fiware,
    Siemens,
}

#[derive(Debug)]
pub enum SourceType {
    SmartMeter,
    Sensor,
}

#[derive(Debug)]
pub struct Application<'a> {
    pub r#type: ApplicationType,
    pub layout: Layout,
    pub roles: Vec<&'a str>,
    pub panels: HashMap<&'a str, PanelTypeUnion<'a>>, // called 'visualizations' for Dash, e.g. <name>: <visualization>
}

#[derive(Debug)]
pub enum ApplicationType {
    Web,
    Mobile,
    Desktop,
    Server,
}

#[derive(Debug)]
pub enum Layout {
    Horizontal,
    Vertical,
    SinglePage,
}

#[derive(Debug)]
pub enum PanelTypeUnion<'a> {
    PieChart(PieChart<'a>),
    TimeSeries(TimeSeries<'a>),
    BarChart(BarChart<'a>),
    GeoMap(GeoMap<'a>),
    XYChart(XYChart<'a>),
    GrafanaMap(GrafanaMap<'a>),
}

#[derive(Debug)]
pub enum PanelType {
    PieChart,
    TimeSeries,
    BarChart,
    GeoMap,
    XYChart,
    GrafanaMap,
}

#[derive(Debug)]
pub struct GeoMap<'a> {
    pub label: &'a str,
    pub r#type: PanelType,
    pub source: &'a str,
    pub data: Vec<&'a str>,
    pub area: Option<&'a str>,
}

#[derive(Debug)]
pub struct GrafanaMap<'a> {
    pub label: &'a str,
    pub r#type: PanelType,
    pub source: &'a str,
    pub data: Vec<&'a str>,
    pub area: Option<&'a str>,
}

#[derive(Debug)]
pub struct PieChart<'a> {
    pub label: &'a str,
    pub r#type: PanelType,
    pub source: &'a str,
    pub traces: Vec<&'a str>,
    pub pie_chart_type: Option<PieChartType>,
}

#[derive(Debug)]
pub enum PieChartType {
    Pie,
    Donut,
}

#[derive(Debug)]
pub struct BarChart<'a> {
    pub label: &'a str,
    pub r#type: PanelType,
    pub source: &'a str,
    pub traces: Vec<&'a str>,
}

#[derive(Debug)]
pub struct TimeSeries<'a> {
    pub label: &'a str,
    pub r#type: PanelType,
    pub source: &'a str,
    pub traces: Vec<&'a str>,
}

#[derive(Debug)]
pub struct XYChart<'a> {
    pub label: &'a str,
    pub r#type: PanelType,
    pub source: &'a str,
    pub traces: Vec<&'a str>,
}

#[derive(Debug)]
pub struct Deployment<'a> {
    pub environments: HashMap<&'a str, Environment<'a>>,
}

#[derive(Debug, Copy, Clone)]
pub struct Environment<'a> {
    pub uri: &'a str,
    pub port: i32,
    pub r#type: EnvironmentType,
}

#[derive(Debug, Copy, Clone)]
pub enum EnvironmentType {
    Docker,
}

impl<'a> Config<'a> {
    pub fn new(blocks: &Blocks<'a>) -> Result<Self, SagError> {
        let data = blocks
            .get("data")
            .ok_or(SagError::missing_section("data"))?;

        match data {
            Value::Block(_) => (),
            _ => return Err(SagError::error("data section is not a block")),
        }

        let data_sources = parse!(data, Vec<&str>, "data", "sources");

        let service: Service = Service::new(blocks)?;
        let mut data_sources_map: HashMap<&str, Datasource> = HashMap::new();

        for name in data_sources {
            data_sources_map.insert(name, Datasource::new(blocks, name)?);
        }

        let application: Application = Application::new(blocks)?;
        let deployment: Deployment = Deployment::new(blocks)?;

        let config = Config {
            service,
            data_sources: data_sources_map,
            application,
            deployment,
        };

        Config::validate(&config)?;
        Ok(config)
    }

    fn validate(config: &Config) -> Result<(), SagError> {
        Config::validate_datasources(config)?;
        Ok(())
    }

    /// Validate that all datasources referenced in the application panels are defined
    fn validate_datasources(config: &Config) -> Result<(), SagError> {
        for (panel_name, panel) in config.application.panels.iter() {
            if config.data_sources.get(panel.get_source()).is_none() {
                return Err(SagError::parsing_error(
                    panel_name,
                    "source",
                    format!("invalid source: {}", panel.get_source()),
                ));
            }
        }
        Ok(())
    }
}

impl<'a> Service<'a> {
    fn new(blocks: &Blocks<'a>) -> Result<Self, SagError> {
        let block = blocks
            .get("service")
            .ok_or(SagError::missing_section("service"))?;

        match block {
            Value::Block(_) => (),
            _ => return Err(SagError::error("service section is not a block")),
        }

        let title = parse!(block, &str, "service", "title");
        let scope = parse!(block, &str, "service", "scope");
        let version = parse!(block, &str, "service", "version");

        let test = match block {
            Value::Block(block) => match block.get("test") {
                Some(value) => Some(Test::new(value)?),
                None => None,
            },
            _ => return Err(SagError::error("service section is not a block")),
        };

        Ok(Service {
            title,
            scope: Scope::from_str(scope)
                .map_err(|e| SagError::parsing_error("service", "scope", e))?,
            version: Version::from_str(version)
                .map_err(|e| SagError::parsing_error("service", "version", e))?,
            test,
        })
    }
}

impl<'a> Test<'a> {
    fn new(block: &Value<'a>) -> Result<Self, SagError> {
        let one = parse!(block, &str, "service.test", "one");
        let two = parse!(block, &str, "service.test", "two");

        Ok(Test { one, two })
    }
}

impl<'a> Datasource<'a> {
    fn new(blocks: &Blocks<'a>, name: &'a str) -> Result<Self, SagError> {
        let block = blocks.get(name).ok_or(SagError::missing_section(name))?;

        match block {
            Value::Block(_) => (),
            _ => return Err(SagError::error("datasource section is not a block")),
        }

        let provider = parse!(block, &str, name, "provider");
        let r#type = parse!(block, &str, name, "type");
        let uri = parse!(block, &str, name, "uri");
        let query = parse!(block, &str, name, "query");

        Ok(Datasource {
            provider: Provider::from_str(provider)
                .map_err(|e| SagError::parsing_error("datasource", "provider", e))?,
            r#type: SourceType::from_str(r#type).map_err(|e| {
                SagError::parsing_error("datasource", "type", format!("invalid type: {}", e))
            })?,
            uri: Url::parse(uri).map_err(|e| {
                SagError::parsing_error("datasource", "uri", format!("invalid uri: {}", e))
            })?,
            query,
        })
    }
}

impl<'a> Application<'a> {
    fn new(blocks: &Blocks<'a>) -> Result<Self, SagError> {
        let block = blocks
            .get("application")
            .ok_or(SagError::missing_section("application"))?;

        match block {
            Value::Block(_) => (),
            _ => return Err(SagError::error("application section is not a block")),
        }

        let r#type = parse!(block, &str, "application", "type");
        let layout = parse!(block, &str, "application", "layout");
        let roles = parse!(block, Vec<&str>, "application", "roles");
        let panels = parse!(block, Vec<&str>, "application", "panels");

        let mut panels_map = HashMap::new();

        for panel_name in panels {
            let panel = blocks
                .get(panel_name)
                .ok_or(SagError::missing_section(panel_name))?;

            match panel {
                Value::Block(_) => (),
                _ => {
                    return Err(SagError::error(format!(
                        "{panel_name} section is not a block"
                    )))
                }
            }

            let panel_type = parse!(panel, &str, panel_name, "type");

            let panel_type = PanelType::from_str(panel_type)
                .map_err(|e| SagError::parsing_error(panel_name, "type", e))?;

            let panel_type_union = match panel_type {
                PanelType::PieChart => PanelTypeUnion::PieChart(PieChart::new(blocks, panel_name)?),
                PanelType::TimeSeries => {
                    PanelTypeUnion::TimeSeries(TimeSeries::new(blocks, panel_name)?)
                }
                PanelType::BarChart => PanelTypeUnion::BarChart(BarChart::new(blocks, panel_name)?),
                PanelType::GeoMap => PanelTypeUnion::GeoMap(GeoMap::new(blocks, panel_name)?),
                PanelType::XYChart => PanelTypeUnion::XYChart(XYChart::new(blocks, panel_name)?),
                PanelType::GrafanaMap => PanelTypeUnion::GrafanaMap(GrafanaMap::new(blocks, panel_name)?),
            };

            panels_map.insert(panel_name, panel_type_union);
        }

        Ok(Application {
            r#type: ApplicationType::from_str(r#type)
                .map_err(|e| SagError::parsing_error("application", "type", e))?,
            layout: Layout::from_str(layout)
                .map_err(|e| SagError::parsing_error("application", "layout", e))?,
            roles: roles.to_owned(),
            panels: panels_map,
        })
    }
}

impl<'a> GeoMap<'a> {
    fn new(blocks: &Blocks<'a>, block_name: &'a str) -> Result<Self, SagError> {
        let block = blocks
            .get(block_name)
            .ok_or(SagError::missing_section(block_name))?;

        match block {
            Value::Block(_) => (),
            _ => {
                return Err(SagError::error(format!(
                    "{block_name} section is not a block"
                )))
            }
        }

        let label = parse!(block, &str, block_name, "label");
        let r#type = parse!(block, &str, block_name, "type");
        let source = parse!(block, &str, block_name, "source");
        let data = parse!(block, Vec<&str>, block_name, "data");
        let area = parse!(block, Option<&str>, block_name, "label");

        Ok(GeoMap {
            label,
            r#type: PanelType::from_str(r#type)
                .map_err(|e| SagError::parsing_error(block_name, "type", e))?,
            source,
            data,
            area,
        })
    }
}

impl <'a> GrafanaMap<'a> {
    fn new(blocks: &Blocks<'a>, block_name: &'a str) -> Result<Self, SagError> {
        let block = blocks
            .get(block_name)
            .ok_or(SagError::missing_section(block_name))?;

        match block {
            Value::Block(_) => (),
            _ => {
                return Err(SagError::error(format!(
                    "{block_name} section is not a block"
                )))
            }
        }
        let label = parse!(block, &str, block_name, "label");
        let r#type = parse!(block, &str, block_name, "type");
        let source = parse!(block, &str, block_name, "source");
        let data = parse!(block, Vec<&str>, block_name, "data");
        let area = parse!(block, Option<&str>, block_name, "label");

        Ok(GrafanaMap {
            label,
            r#type: PanelType::from_str(r#type)
                .map_err(|e| SagError::parsing_error(block_name, "type", e))?,
            source,
            data,
            area,
        })
    }
}

impl<'a> PieChart<'a> {
    fn new(blocks: &Blocks<'a>, block_name: &'a str) -> Result<Self, SagError> {
        let block = blocks
            .get(block_name)
            .ok_or(SagError::missing_section(block_name))?;

        match block {
            Value::Block(_) => (),
            _ => {
                return Err(SagError::error(format!(
                    "{block_name} section is not a block"
                )))
            }
        }

        let label = parse!(block, &str, block_name, "label");
        let r#type = parse!(block, &str, block_name, "type");
        let source = parse!(block, &str, block_name, "source");
        let traces = parse!(block, Vec<&str>, block_name, "traces");
        let pie_chart_type = parse!(block, Option<&str>, block_name, "pie_chart_type");

        let pie_chart_type = match pie_chart_type {
            Some(pie_chart_type) => Some(
                PieChartType::from_str(pie_chart_type)
                    .map_err(|e| SagError::parsing_error(block_name, "pie_chart_type", e))?,
            ),
            None => None,
        };

        Ok(PieChart {
            label,
            r#type: PanelType::from_str(r#type)
                .map_err(|e| SagError::parsing_error(block_name, "type", e))?,
            source,
            traces,
            pie_chart_type,
        })
    }
}

impl<'a> BarChart<'a> {
    fn new(blocks: &Blocks<'a>, block_name: &'a str) -> Result<Self, SagError> {
        let block = blocks
            .get(block_name)
            .ok_or(SagError::missing_section(block_name))?;

        match block {
            Value::Block(_) => (),
            _ => {
                return Err(SagError::error(format!(
                    "{block_name} section is not a block"
                )))
            }
        }

        let label = parse!(block, &str, block_name, "label");
        let r#type = parse!(block, &str, block_name, "type");
        let source = parse!(block, &str, block_name, "source");
        let traces = parse!(block, Vec<&str>, block_name, "traces");

        Ok(BarChart {
            label,
            r#type: PanelType::from_str(r#type)
                .map_err(|e| SagError::parsing_error(block_name, "type", e))?,
            source,
            traces,
        })
    }
}

impl<'a> TimeSeries<'a> {
    fn new(blocks: &Blocks<'a>, block_name: &'a str) -> Result<Self, SagError> {
        let block = blocks
            .get(block_name)
            .ok_or(SagError::missing_section(block_name))?;

        match block {
            Value::Block(_) => (),
            _ => {
                return Err(SagError::error(format!(
                    "{block_name} section is not a block"
                )))
            }
        }

        let label = parse!(block, &str, block_name, "label");
        let r#type = parse!(block, &str, block_name, "type");
        let source = parse!(block, &str, block_name, "source");
        let traces = parse!(block, Vec<&str>, block_name, "traces");

        Ok(TimeSeries {
            label,
            r#type: PanelType::from_str(r#type)
                .map_err(|e| SagError::parsing_error(block_name, "type", e))?,
            source,
            traces,
        })
    }
}

impl<'a> XYChart<'a> {
    fn new(blocks: &Blocks<'a>, block_name: &'a str) -> Result<Self, SagError> {
        let block = blocks
            .get(block_name)
            .ok_or(SagError::missing_section(block_name))?;

        match block {
            Value::Block(_) => (),
            _ => {
                return Err(SagError::error(format!(
                    "{block_name} section is not a block"
                )))
            }
        }

        let label = parse!(block, &str, block_name, "label");
        let r#type = parse!(block, &str, block_name, "type");
        let source = parse!(block, &str, block_name, "source");
        let traces = parse!(block, Vec<&str>, block_name, "traces");

        Ok(XYChart {
            label,
            r#type: PanelType::from_str(r#type)
                .map_err(|e| SagError::parsing_error(block_name, "type", e))?,
            source,
            traces,
        })
    }
}

impl<'a> Deployment<'a> {
    fn new(blocks: &Blocks<'a>) -> Result<Self, SagError> {
        let block = blocks
            .get("deployment")
            .ok_or(SagError::missing_section("deployment"))?;

        match block {
            Value::Block(_) => (),
            _ => return Err(SagError::error("deployment section is not a block")),
        }

        let environments = parse!(block, Vec<&str>, "deployment", "environments");

        let mut environments_map = HashMap::new();
        for environment_name in environments {
            let environment = Environment::new(blocks, environment_name)?;

            environments_map.insert(environment_name, environment);
        }

        Ok(Deployment {
            environments: environments_map,
        })
    }
}

impl<'a> Environment<'a> {
    fn new(blocks: &Blocks<'a>, block_name: &'a str) -> Result<Self, SagError> {
        let block = blocks
            .get(block_name)
            .ok_or(SagError::missing_section(block_name))?;

        match block {
            Value::Block(_) => (),
            _ => {
                return Err(SagError::error(format!(
                    "{block_name} section is not a block"
                )))
            }
        }

        let uri = parse!(block, &str, block_name, "uri");
        let port = parse!(block, &str, block_name, "port");
        let r#type = parse!(block, &str, block_name, "type");

        let port = port
            .parse::<i32>()
            .map_err(|_| SagError::parsing_error(block_name, "port", "port is not an integer"))?;

        Ok(Environment {
            uri,
            port,
            r#type: EnvironmentType::from_str(r#type)
                .map_err(|e| SagError::parsing_error(block_name, "type", e))?,
        })
    }
}

// Panel implementation

trait Panel {
    fn get_source(&self) -> &str;
    fn get_label(&self) -> &str;
    fn get_traces(&self) -> &Vec<&str>;
}

impl<'a> Panel for PanelTypeUnion<'a> {
    fn get_source(&self) -> &str {
        match self {
            PanelTypeUnion::GeoMap(map) => map.source,
            PanelTypeUnion::XYChart(xy) => xy.source,
            PanelTypeUnion::PieChart(pie) => pie.source,
            PanelTypeUnion::BarChart(bar) => bar.source,
            PanelTypeUnion::TimeSeries(ts) => ts.source,
            PanelTypeUnion::GrafanaMap(gm) => gm.source,
        }
    }

    fn get_label(&self) -> &str {
        match self {
            PanelTypeUnion::GeoMap(map) => map.label,
            PanelTypeUnion::XYChart(xy) => xy.label,
            PanelTypeUnion::PieChart(pie) => pie.label,
            PanelTypeUnion::BarChart(bar) => bar.label,
            PanelTypeUnion::TimeSeries(ts) => ts.label,
            PanelTypeUnion::GrafanaMap(gm) => gm.label,
        }
    }

    fn get_traces(&self) -> &Vec<&str> {
        match self {
            PanelTypeUnion::GeoMap(map) => &map.data,
            PanelTypeUnion::GrafanaMap(gm) => &gm.data,
            PanelTypeUnion::XYChart(xy) => &xy.traces,
            PanelTypeUnion::PieChart(pie) => &pie.traces,
            PanelTypeUnion::BarChart(bar) => &bar.traces,
            PanelTypeUnion::TimeSeries(ts) => &ts.traces,
        }
    }
}

// FromStr implementations

impl FromStr for Scope {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Service" => Ok(Scope::Service),
            "Industry" => Ok(Scope::Industry),
            "Manifacturing" => Ok(Scope::Manifacturing),
            "Education" => Ok(Scope::Education),
            "Healthcare" => Ok(Scope::Healthcare),
            "Social_programs" => Ok(Scope::SocialPrograms),
            "Government" => Ok(Scope::Government),
            "Energy" => Ok(Scope::Energy),
            "Water" => Ok(Scope::Water),
            "Environment" => Ok(Scope::Environment),
            "Transportation" => Ok(Scope::Transportation),
            "Communication" => Ok(Scope::Communication),
            "Public_safety" => Ok(Scope::PublicSafety),
            "Urban_planning" => Ok(Scope::UrbanPlanning),
            "Infrastructure" => Ok(Scope::Infrastructure),
            _ => Err(format!("invalid scope: {}", s)),
        }
    }
}

impl FromStr for Version {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let versions: IResult<&str, Vec<i16>> = separated_list1(tag("."), i16)(s);

        match versions {
            Ok((_, versions)) => {
                // if versions.len() != 3 {
                //     return Err(format!("invalid version: {}", s));
                // }
                let major = versions.get(0).map_or(0, |v| *v);
                let minor = versions.get(1).map_or(0, |v| *v);
                let patch = versions.get(2).map_or(0, |v| *v);

                Ok(Version {
                    major,
                    minor,
                    patch,
                })
            }
            Err(e) => Err(e.to_string()),
        }
    }
}

impl FromStr for SourceType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "SmartMeter" => Ok(SourceType::SmartMeter),
            "Sensor" => Ok(SourceType::Sensor),
            _ => Err(format!("invalid source type: {}", s)),
        }
    }
}

impl FromStr for Provider {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Fiware" => Ok(Provider::Fiware),
            "Siemens" => Ok(Provider::Siemens),
            _ => Err(format!("invalid provider: {}", s)),
        }
    }
}

impl FromStr for ApplicationType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Web" => Ok(ApplicationType::Web),
            "Mobile" => Ok(ApplicationType::Mobile),
            "Desktop" => Ok(ApplicationType::Desktop),
            "Server" => Ok(ApplicationType::Server),
            _ => Err(format!("invalid application type: {}", s)),
        }
    }
}

impl FromStr for Layout {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Horizontal" => Ok(Layout::Horizontal),
            "Vertical" => Ok(Layout::Vertical),
            "SinglePage" => Ok(Layout::SinglePage),
            _ => Err(format!("invalid layout: {}", s)),
        }
    }
}

impl FromStr for PanelType {
    type Err = String;
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "pie_chart" => Ok(PanelType::PieChart),
            "timeseries" => Ok(PanelType::TimeSeries),
            "bar_chart" => Ok(PanelType::BarChart),
            "geomap" => Ok(PanelType::GeoMap),
            "xy_chart" => Ok(PanelType::XYChart),
            "grafana-map" => Ok(PanelType::GrafanaMap),
            _ => Err(format!("invalid panel type: {}", input)),
        }
    }
}

impl FromStr for PieChartType {
    type Err = String;
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "pie" => Ok(PieChartType::Pie),
            "donut" => Ok(PieChartType::Donut),
            _ => Err(format!("invalid pie_chart_type: {}", input)),
        }
    }
}

impl FromStr for EnvironmentType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Docker" => Ok(EnvironmentType::Docker),
            _ => Err(format!("invalid environment type: {}", s)),
        }
    }
}

// ToString implementations

impl ToString for Version {
    fn to_string(&self) -> String {
        format!("{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl ToString for Scope {
    fn to_string(&self) -> String {
        match self {
            Scope::Service => String::from("service"),
            Scope::Industry => String::from("industry"),
            Scope::Manifacturing => String::from("manifacturing"),
            Scope::Education => String::from("education"),
            Scope::Healthcare => String::from("healthcare"),
            Scope::SocialPrograms => String::from("social_programs"),
            Scope::Government => String::from("government"),
            Scope::Energy => String::from("energy"),
            Scope::Water => String::from("water"),
            Scope::Environment => String::from("environment"),
            Scope::Transportation => String::from("transportation"),
            Scope::Communication => String::from("communication"),
            Scope::PublicSafety => String::from("public_safety"),
            Scope::UrbanPlanning => String::from("urban_planning"),
            Scope::Infrastructure => String::from("infrastructure"),
        }
    }
}

impl ToString for SourceType {
    fn to_string(&self) -> String {
        match self {
            SourceType::SmartMeter => String::from("SmartMeter"),
            SourceType::Sensor => String::from("Sensor"),
        }
    }
}

impl ToString for Provider {
    fn to_string(&self) -> String {
        match self {
            Provider::Fiware => String::from("Fiware"),
            Provider::Siemens => String::from("Siemens"),
        }
    }
}

impl ToString for ApplicationType {
    fn to_string(&self) -> String {
        match self {
            ApplicationType::Web => String::from("Web"),
            ApplicationType::Mobile => String::from("Mobile"),
            ApplicationType::Desktop => String::from("Desktop"),
            ApplicationType::Server => String::from("Server"),
        }
    }
}

impl ToString for Layout {
    fn to_string(&self) -> String {
        match self {
            Layout::Horizontal => String::from("Horizontal"),
            Layout::Vertical => String::from("Vertical"),
            Layout::SinglePage => String::from("SinglePage"),
        }
    }
}

impl ToString for PanelTypeUnion<'_> {
    fn to_string(&self) -> String {
        match self {
            PanelTypeUnion::PieChart(_) => String::from("pie_chart"),
            PanelTypeUnion::TimeSeries(_) => String::from("timeseries"),
            PanelTypeUnion::BarChart(_) => String::from("bar_chart"),
            PanelTypeUnion::GeoMap(_) => String::from("geomap"),
            PanelTypeUnion::XYChart(_) => String::from("xy_chart"),
            PanelTypeUnion::GrafanaMap(_) => String::from("grafana-map"),
        }
    }
}

impl ToString for PanelType {
    fn to_string(&self) -> String {
        match self {
            PanelType::PieChart => String::from("pie_chart"),
            PanelType::TimeSeries => String::from("timeseries"),
            PanelType::BarChart => String::from("bar_chart"),
            PanelType::GeoMap => String::from("geomap"),
            PanelType::XYChart => String::from("xy_chart"),
            PanelType::GrafanaMap => String::from("grafana-map"),
        }
    }
}

impl ToString for PieChartType {
    fn to_string(&self) -> String {
        match self {
            PieChartType::Pie => String::from("pie"),
            PieChartType::Donut => String::from("donut"),
        }
    }
}

impl ToString for EnvironmentType {
    fn to_string(&self) -> String {
        match self {
            EnvironmentType::Docker => String::from("Docker"),
        }
    }
}

// From implementation

impl<'a> From<Scope> for &'a str {
    fn from(s: Scope) -> &'a str {
        match s {
            Scope::Service => "Service",
            Scope::Industry => "Industry",
            Scope::Manifacturing => "Manifacturing",
            Scope::Education => "Education",
            Scope::Healthcare => "Healthcare",
            Scope::SocialPrograms => "Social_programs",
            Scope::Government => "Government",
            Scope::Energy => "Energy",
            Scope::Water => "Water",
            Scope::Environment => "Environment",
            Scope::Transportation => "Transportation",
            Scope::Communication => "Communication",
            Scope::PublicSafety => "Public_safety",
            Scope::UrbanPlanning => "Urban_planning",
            Scope::Infrastructure => "Infrastructure",
        }
    }
}

impl<'a> From<SourceType> for &'a str {
    fn from(st: SourceType) -> &'a str {
        match st {
            SourceType::SmartMeter => "SmartMeter",
            SourceType::Sensor => "Sensor",
        }
    }
}

impl<'a> From<Provider> for &'a str {
    fn from(p: Provider) -> &'a str {
        match p {
            Provider::Fiware => "Fiware",
            Provider::Siemens => "Siemens",
        }
    }
}

impl<'a> From<ApplicationType> for &'a str {
    fn from(a: ApplicationType) -> &'a str {
        match a {
            ApplicationType::Web => "Web",
            ApplicationType::Mobile => "Mobile",
            ApplicationType::Desktop => "Desktop",
            ApplicationType::Server => "Server",
        }
    }
}

impl<'a> From<Layout> for &'a str {
    fn from(l: Layout) -> &'a str {
        match l {
            Layout::Horizontal => "Horizontal",
            Layout::Vertical => "Vertical",
            Layout::SinglePage => "SinglePage",
        }
    }
}

impl<'a> From<PanelTypeUnion<'a>> for &'a str {
    fn from(ptu: PanelTypeUnion) -> &'a str {
        match ptu {
            PanelTypeUnion::PieChart(_) => "pie_chart",
            PanelTypeUnion::TimeSeries(_) => "timeseries",
            PanelTypeUnion::BarChart(_) => "bar_chart",
            PanelTypeUnion::GeoMap(_) => "geomap",
            PanelTypeUnion::XYChart(_) => "xy_chart",
            PanelTypeUnion::GrafanaMap(_) => "grafana-map",
        }
    }
}

impl<'a> From<PanelType> for &'a str {
    fn from(pt: PanelType) -> &'a str {
        match pt {
            PanelType::PieChart => "pie_chart",
            PanelType::TimeSeries => "timeseries",
            PanelType::BarChart => "bar_chart",
            PanelType::GeoMap => "geomap",
            PanelType::XYChart => "xy_chart",
            PanelType::GrafanaMap => "grafana-map",
        }
    }
}

impl<'a> From<PieChartType> for &'a str {
    fn from(pct: PieChartType) -> &'a str {
        match pct {
            PieChartType::Pie => "pie",
            PieChartType::Donut => "donut",
        }
    }
}

impl<'a> From<EnvironmentType> for &'a str {
    fn from(et: EnvironmentType) -> &'a str {
        match et {
            EnvironmentType::Docker => "Docker",
        }
    }
}
