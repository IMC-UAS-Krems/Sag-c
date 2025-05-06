// This file defines, parses and validates a complex configuration structure for a smart city application.

// 'nom' crate for parsing
use nom::bytes::complete::tag;
use nom::character::complete::i16;
use nom::multi::separated_list1;
use nom::IResult;

use std::collections::HashMap;
use std::fmt::Display; // Display trait for formatting
use std::str::FromStr; // FromStr trait for parsing

use url::Url; // Url type for parsing URLs

// Custom error types and parsing utilities
use crate::errors::LanguageErrorKind;
use crate::errors::SagError;
use crate::parse;
use crate::parser::Blocks;
use crate::parser::ParseResult;
use crate::parser::Position;
use crate::parser::Value;

// -----------Section 1: Structs and Enums-----------
// This section contains the definitions of the structs and enums

// Top-level configuration
#[derive(Debug)]
pub struct Config<'a> {
    pub service: Service<'a>,         // Setting it to Grafana or Dash dashboard
    pub data: SagData<'a>,            // Where to get the data from
    pub application: Application<'a>, // How to display the data
    pub deployment: Deployment<'a>,   // Where to deploy the app e.g. local, azure
}

// Setting service (Grafana or Dash) and version
#[derive(Debug)]
pub struct Service<'a> {
    pub title: &'a str,
    pub scope: Scope,
    pub version: Version,
}

// Sector for the smart city
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

// Version of the dashboard
#[derive(Debug)]
pub struct Version {
    pub major: i16,
    pub minor: i16,
    pub patch: i16,
}

// Pointer to the data sources
#[derive(Debug)]
pub struct SagData<'a> {
    pub sources: HashMap<&'a str, Datasource<'a>>,
}

// Data source details, where to get the data from
#[derive(Debug)]
pub struct Datasource<'a> {
    pub provider: Provider,
    pub r#type: SourceType,
    pub uri: Url,               // URL to the data source
    pub query: Option<&'a str>, // Filter which data to be fetched from source
    pub config: Option<DatasourceConfig<'a>>,
}

// Who provided the data source
#[derive(Debug)]
pub enum Provider {
    Fiware,
    Dataskop,
}

// Type of the data source
#[derive(Debug)]
pub enum SourceType {
    SmartMeter,
    Sensor,
}

// Additional details for accessing and interpreting the data
#[derive(Debug)]
pub struct DatasourceConfig<'a> {
    pub company: usize,                        // Who owns the data
    pub measurements: HashMap<usize, &'a str>, // What data is available
    pub token: &'a str,                        // For authentication
}

// How to display the data
#[derive(Debug)]
pub struct Application<'a> {
    pub r#type: ApplicationType,
    pub dashboard: DashboardType,
    pub layout: Layout,
    pub roles: Vec<&'a str>, // Who can access the dashboard
    // forms of visualizations e.g. pie chart, map, etc..
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
pub enum DashboardType {
    Dash,
    Grafana,
}

#[derive(Debug)]
pub enum Layout {
    Horizontal,
    Vertical,
    SinglePage,
}

// Different types of visualizations with actual data
#[derive(Debug)]
pub enum PanelTypeUnion<'a> {
    BarChart(BarChart<'a>),
    GeoMap(GeoMap<'a>),
    GrafanaBnB(GrafanaBnB<'a>),
    GrafanaBulletGraph(GrafanaBulletGraph<'a>),
    GrafanaCalendar(GrafanaCalendar<'a>),
    GrafanaExtValues(GrafanaExtValues<'a>),
    GrafanaMap(GrafanaMap<'a>),
    GrafanaMultiLine(GrafanaMultiLine<'a>),
    GrafanaSingleLine(GrafanaSingleLine<'a>),
    PieChart(PieChart<'a>),
    TimeSeries(TimeSeries<'a>),
    XYChart(XYChart<'a>),
}

// Only specifying the type of visualization
#[derive(Debug)]
pub enum PanelType {
    BarChart,
    GeoMap,
    GrafanaBnB,
    GrafanaBulletGraph,
    GrafanaCalendar,
    GrafanaExtValues,
    GrafanaMap,
    GrafanaMultiLine,
    GrafanaSingleLine,
    PieChart,
    TimeSeries,
    XYChart,
}

// -----------Panel Types-----------

#[derive(Debug)]
pub struct BarChart<'a> {
    pub label: &'a str,
    pub r#type: PanelType,
    pub source: &'a str,
    pub traces: Vec<&'a str>,
}

#[derive(Debug)]
pub struct GeoMap<'a> {
    pub label: &'a str,
    pub r#type: PanelType,
    pub source: &'a str,
    pub data: Vec<&'a str>,
    pub area: Option<&'a str>,
    pub color_by: Option<&'a str>,
}

#[derive(Debug)]
pub struct GrafanaBnB<'a> {
    pub r#type: PanelType,
    pub source: &'a str,
    pub locations: Vec<&'a str>,
    pub traces: Vec<&'a str>,
}

#[derive(Debug)]
pub struct GrafanaBulletGraph<'a> {
    pub r#type: PanelType,
    pub source: &'a str,
    pub locations: Vec<&'a str>,
    pub traces: Vec<&'a str>,
}

#[derive(Debug)]
pub struct GrafanaCalendar<'a> {
    pub r#type: PanelType,
    pub source: &'a str,
    pub locations: Vec<&'a str>,
    pub traces: Vec<&'a str>,
}

#[derive(Debug)]
pub struct GrafanaExtValues<'a> {
    pub r#type: PanelType,
    pub source: &'a str,
    pub locations: Vec<&'a str>,
    pub traces: Vec<&'a str>,
}

#[derive(Debug)]
pub struct GrafanaMap<'a> {
    pub r#type: PanelType,
    pub source: &'a str,
    pub traces: Vec<&'a str>,
}

#[derive(Debug)]
pub struct GrafanaMultiLine<'a> {
    pub r#type: PanelType,
    pub source: &'a str,
    pub locations: Vec<&'a str>,
    pub traces: Vec<&'a str>,
}

#[derive(Debug)]
pub struct GrafanaSingleLine<'a> {
    pub r#type: PanelType,
    pub source: &'a str,
    pub traces: Vec<&'a str>,
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

// -----------End of Panel Types-----------

// Where to deploy the application e.g. local, azure
#[derive(Debug)]
pub struct Deployment<'a> {
    pub environments: HashMap<&'a str, Environment<'a>>,
}

// Details of the deployment environment
#[derive(Debug)]
pub struct Environment<'a> {
    pub uri: &'a str,
    pub port: i32,
    pub r#type: EnvironmentType,
}

#[derive(Debug)]
pub enum EnvironmentType {
    Docker,
    Azure,
}

// -----------Section 2: Implementations-----------
// This section contains the implementations of the previously defined structs and enums

impl<'a> Config<'a> {
    pub fn new(blocks: &mut Blocks<'a>) -> Result<Self, Vec<SagError>> {
        let mut errors = Vec::new();

        // Parse the sections
        let service = Service::new(blocks);
        let data = SagData::new(blocks);
        let application = Application::new(blocks);
        let deployment = Deployment::new(blocks);

        // Check if there are any errors in the parsing
        let service = match service {
            Ok(service) => Some(service),
            Err(e) => {
                errors.extend(e);
                None
            }
        };
        let data = match data {
            Ok(data) => Some(data),
            Err(e) => {
                errors.extend(e);
                None
            }
        };
        let application = match application {
            Ok(application) => Some(application),
            Err(e) => {
                errors.extend(e);
                None
            }
        };
        let deployment = match deployment {
            Ok(deployment) => Some(deployment),
            Err(e) => {
                errors.extend(e);
                None
            }
        };

        // If there are any errors, return them
        if !errors.is_empty() {
            return Err(errors);
        }

        // Create the configuration
        let config = Config {
            service: service.unwrap(),
            data: data.unwrap(),
            application: application.unwrap(),
            deployment: deployment.unwrap(),
        };

        //Config::validate(&config)?; // TODO: delete?

        // Return the configuration if there are no errors
        Ok(config)
    }

    // TODO: Will this stay here?
    //fn validate(config: &Config) -> Result<(), Vec<SagError>> {
    //    Config::validate_datasources(config)?;
    //    Ok(())
    //}
    // Validate that all datasources referenced in the application panels are defined
    //fn validate_datasources(config: &Config) -> Result<(), Vec<SagError>> {
    //    let mut errors = Vec::new();
    //    for (panel_name, panel) in config.application.panels.iter() {
    //        if config.data.sources.get(panel.get_source()).is_none() {
    //            return Err(SagError::language_error(
    //                panel_name,
    //                Some("source"),
    //                format!("invalid source: {}", panel.get_source()),
    //            ));
    //        }
    //    }
    //    Ok(())
    //}

    // Iterates over the panels and keep only panels that pass the filter
    pub fn filter_panels<F>(&mut self, filter: F)
    where
        F: Fn(&PanelTypeUnion) -> bool,
    {
        self.application.panels.retain(|_, panel| filter(panel));
    }
}

impl<'a> Service<'a> {
    // Check if the service section is correctly defined
    // Return the title, scope and version or errors if there are any
    fn check(blocks: &mut Blocks<'a>) -> Result<(&'a str, Scope, Version), Vec<SagError>> {
        const SECTION_NAME: &str = "service";
        let mut errors = Vec::new();

        // Get the service section from the blocks
        let service = blocks
            .get_mut(SECTION_NAME)
            .ok_or(SagError::internal_error(format!(
                "Missing section {}",
                SECTION_NAME
            )));

        // If there is an error, return it
        if let Err(e) = service {
            errors.push(e);
            return Err(errors);
        }

        let service = service.unwrap();

        // Check if the service section is formatted correctly
        if !service.value.is_block() {
            errors.push(SagError::language_error(
                LanguageErrorKind::InvalidType(),
                service.position,
            ));
            return Err(errors);
        }

        // Parse the title, scope and version
        let title = parse!(service, &str, SECTION_NAME, "title");
        let scope = parse!(service, &str, SECTION_NAME, "scope");
        let version = parse!(service, &str, SECTION_NAME, "version");

        // Check if there are any errors in the parsing
        let title: Option<&str> = match title {
            Ok((title, _)) => Some(title),
            Err(e) => {
                errors.push(e);
                None
            }
        };
        let scope: Option<Scope> = match scope {
            Ok((scope, scope_pos)) => Scope::from_str(scope)
                .map_err(|_| {
                    errors.push(SagError::language_error(
                        LanguageErrorKind::InvalidValue(scope.to_string()),
                        scope_pos,
                    ));
                })
                .ok(),
            Err(e) => {
                errors.push(e);
                None
            }
        };
        let version: Option<Version> = match version {
            Ok((version, version_pos)) => Version::from_str(version)
                .map_err(|_| {
                    errors.push(SagError::language_error(
                        LanguageErrorKind::InvalidValue(version.to_string()),
                        version_pos,
                    ));
                })
                .ok(),
            Err(e) => {
                errors.push(e);
                None
            }
        };

        // If there are any errors, return them
        if !errors.is_empty() {
            return Err(errors);
        }

        // Return the title, scope and version
        Ok((title.unwrap(), scope.unwrap(), version.unwrap()))
    }

    // Create a new service section, return the title, scope and version
    pub fn new(blocks: &mut Blocks<'a>) -> Result<Self, Vec<SagError>> {
        // Check if the service section is correctly defined
        let (title, scope, version) = Service::check(blocks)?;

        // Return the service section
        Ok(Service {
            title,
            scope,
            version,
        })
    }
}

impl<'a> SagData<'a> {
    // Check if the data section is correctly defined
    // Return the data sources or errors if there are any
    fn check(blocks: &mut Blocks<'a>) -> Result<(Vec<&'a str>, Position), Vec<SagError>> {
        const SECTION_NAME: &str = "data";
        let mut errors = Vec::new();

        // Get the data section from the blocks
        let data = blocks
            .get_mut(SECTION_NAME)
            .ok_or(SagError::internal_error(format!(
                "Missing section {}",
                SECTION_NAME
            )));

        // If there is an error, return it
        if let Err(e) = data {
            errors.push(e);
            return Err(errors);
        }

        let data = data.unwrap();

        // Check if the data section is formatted correctly
        if !data.value.is_block() {
            errors.push(SagError::language_error(
                LanguageErrorKind::InvalidType(),
                data.position,
            ));
            return Err(errors);
        }

        // Parse the data sources
        let data_sources = parse!(data, Vec<&str>, SECTION_NAME, "sources");

        // If there are any errors in the parsing, return them
        if let Err(e) = data_sources {
            errors.push(e);
            return Err(errors);
        }

        let data_sources = data_sources.unwrap();

        // Return the data sources
        Ok(data_sources)
    }

    // Create a new data section, return the data sources
    pub fn new(blocks: &mut Blocks<'a>) -> Result<Self, Vec<SagError>> {
        const SECTION_NAME: &str = "data";

        let mut errors = Vec::new();

        // Check if the data section is correctly defined
        let data_sources = SagData::check(blocks)?;

        let mut data_sources_map = HashMap::new();

        // Create a datasource for each data source
        for name in data_sources.0 {
            let datasource = Datasource::new(blocks, name, data_sources.1);
            if let Err(e) = datasource {
                errors.extend(e.into_iter());
            } else {
                data_sources_map.insert(name, datasource.unwrap());
            }
        }

        // If there are any errors, return them
        if !errors.is_empty() {
            return Err(errors);
        }

        // Return the data section
        Ok(SagData {
            sources: data_sources_map,
        })
    }
}

impl<'a> Datasource<'a> {
    // Check if the datasource section is correctly defined
    // Return the provider, type, uri, query, config or errors if there are any
    fn check(
        blocks: &mut Blocks<'a>,
        source_name: &'a str,
        source_name_position: Position,
    ) -> Result<
        (
            Provider,
            SourceType,
            Url,
            Option<&'a str>,
            Option<DatasourceConfig<'a>>,
        ),
        Vec<SagError>,
    > {
        let mut errors = Vec::new();

        // Get the datasource section from the blocks
        let datasource = blocks.get_mut(source_name).ok_or(SagError::language_error(
            LanguageErrorKind::MissingSection(source_name.to_string()),
            source_name_position,
        ));

        // If there is an error, return it
        if let Err(e) = datasource {
            errors.push(e);
            return Err(errors);
        }

        let datasource = datasource.unwrap();

        // Check if the datasource section is formatted correctly
        if !datasource.value.is_block() {
            errors.push(SagError::language_error(
                LanguageErrorKind::InvalidType(),
                datasource.position,
            ));
            return Err(errors);
        }

        // Parse the provider, type, uri, query and config
        let provider = parse!(datasource, &str, source_name, "provider");
        let r#type = parse!(datasource, &str, source_name, "type");
        let uri = parse!(datasource, &str, source_name, "uri");
        let query = parse!(datasource, Option<&str>, source_name, "query");
        let config = DatasourceConfig::new(datasource);

        // Check if there are any errors in the parsing
        let provider: Option<Provider> = match provider {
            Ok((provider, provider_pos)) => Provider::from_str(provider)
                .map_err(|_| {
                    errors.push(SagError::language_error(
                        LanguageErrorKind::InvalidValue(provider.to_string()),
                        provider_pos,
                    ));
                })
                .ok(),
            Err(e) => {
                errors.push(e);
                None
            }
        };
        let r#type: Option<SourceType> = match r#type {
            Ok((r#type, r#type_pos)) => SourceType::from_str(r#type)
                .map_err(|_| {
                    errors.push(SagError::language_error(
                        LanguageErrorKind::InvalidValue(r#type.to_string()),
                        r#type_pos,
                    ));
                })
                .ok(),
            Err(e) => {
                errors.push(e);
                None
            }
        };
        let uri: Option<Url> = match uri {
            Ok((uri, uri_pos)) => Url::parse(uri)
                .map_err(|_| {
                    errors.push(SagError::language_error(
                        LanguageErrorKind::InvalidValue(uri.to_string()),
                        uri_pos,
                    ));
                })
                .ok(),
            Err(e) => {
                errors.push(e);
                None
            }
        };
        let query: Option<&str> = match query {
            Some((query, _)) => Some(query),
            None => match provider {
                Some(Provider::Fiware) => {
                    errors.push(SagError::language_error(
                        LanguageErrorKind::MissingSection("query".to_string()),
                        datasource.position,
                    ));
                    None
                }
                _ => None,
            },
        };
        let config: Option<DatasourceConfig<'a>> = match (config, &provider) {
            (Ok(config), None) => config,
            (Ok(config), Some(Provider::Fiware)) => config,
            (Ok(Some(config)), Some(Provider::Dataskop)) => Some(config),
            (Ok(None), Some(Provider::Dataskop)) => {
                errors.push(SagError::language_error(
                    LanguageErrorKind::MissingSection("config".to_string()),
                    datasource.position,
                ));
                None
            }
            (Err(e), _) => {
                errors.extend(e);
                None
            }
        };

        // If there are any errors, return them
        if !errors.is_empty() {
            return Err(errors);
        }

        // Return the provider, type, uri, query and config
        Ok((
            provider.unwrap(),
            r#type.unwrap(),
            uri.unwrap(),
            query,
            config,
        ))
    }

    // Create a new datasource section, return the provider, type, uri, query and config
    pub fn new(
        blocks: &mut Blocks<'a>,
        source_name: &'a str,
        source_name_position: Position,
    ) -> Result<Self, Vec<SagError>> {
        // Check if the datasource section is correctly defined
        let (provider, r#type, uri, query, config) =
            Datasource::check(blocks, source_name, source_name_position)?;

        // Return the datasource section
        Ok(Datasource {
            provider,
            r#type,
            uri,
            query,
            config,
        })
    }
}

impl<'a> DatasourceConfig<'a> {
    // Check if the config section is correctly defined
    // Return the company, measurements and token or errors if there are any
    fn check(
        datasource: &mut ParseResult<'a>,
    ) -> Result<Option<(usize, HashMap<usize, &'a str>, &'a str)>, Vec<SagError>> {
        const SECTION_NAME: &str = "config";
        let mut errors = Vec::new();

        // Get the datasource section
        let datasource = match &mut datasource.value {
            Value::Block(block) => block,
            _ => unreachable!(),
        };

        // Get the config section from the datasource
        let config = datasource.get_mut(SECTION_NAME);

        // If config section is missing, return None
        if config.is_none() {
            return Ok(None);
        }

        let config = config.unwrap();

        // Check if the config section is formatted correctly
        if !config.value.is_block() {
            errors.push(SagError::language_error(
                LanguageErrorKind::InvalidType(),
                config.position,
            ));
            return Err(errors);
        }

        // Parse the company, measurements and token
        let company = parse!(config, usize, SECTION_NAME, "company");
        let measurement = parse!(config, HashMap<usize, &str>, SECTION_NAME, "measurements");
        let token = parse!(config, &str, SECTION_NAME, "token");

        // Check if there are any errors in the parsing
        let company: Option<usize> = match company {
            Ok((company, _)) => Some(company),
            Err(e) => {
                errors.push(e);
                None
            }
        };
        let measurement: Option<HashMap<usize, &str>> = match measurement {
            Ok((measurement, _)) => Some(measurement),
            Err(e) => {
                errors.push(e);
                None
            }
        };
        let token: Option<&str> = match token {
            Ok((token, _)) => Some(token),
            Err(e) => {
                errors.push(e);
                None
            }
        };

        // If there are any errors, return them
        if !errors.is_empty() {
            return Err(errors);
        }

        // Return the company, measurements and token
        Ok(Some((
            company.unwrap(),
            measurement.unwrap(),
            token.unwrap(),
        )))
    }

    // Create a new config section, return the company, measurements and token
    pub fn new(block: &mut ParseResult<'a>) -> Result<Option<Self>, Vec<SagError>> {
        // Check if the config section is correctly defined
        let result = DatasourceConfig::check(block)?;

        // If config exist, create it, otherwise return None
        if let Some((company, measurements, token)) = result {
            return Ok(Some(DatasourceConfig {
                company,
                measurements,
                token,
            }));
        }
        Ok(None)
    }
}

impl<'a> Application<'a> {
    // Check if the application section is correctly defined
    // Return the type, dashboard, layout, roles, panels or errors if there are any
    fn check(
        blocks: &mut Blocks<'a>,
    ) -> Result<
        (
            ApplicationType,
            DashboardType,
            Layout,
            Vec<&'a str>,
            (Vec<&'a str>, Position),
        ),
        Vec<SagError>,
    > {
        const SECTION_NAME: &str = "application";
        let mut errors = Vec::new();

        // Get the application section from the blocks
        let application = blocks
            .get_mut(SECTION_NAME)
            .ok_or(SagError::internal_error(format!(
                "Missing section {}",
                SECTION_NAME
            )));

        // If there is an error, return it
        if let Err(e) = application {
            errors.push(e);
            return Err(errors);
        }

        let application = application.unwrap();

        // Check if the application section is formatted correctly
        if !application.value.is_block() {
            errors.push(SagError::language_error(
                LanguageErrorKind::InvalidType(),
                application.position,
            ));
            return Err(errors);
        }

        // Parse the type, dashboard, layout, roles and panels
        let r#type = parse!(application, &str, SECTION_NAME, "type");
        let dashboard = parse!(application, &str, SECTION_NAME, "dashboard");
        let layout = parse!(application, &str, SECTION_NAME, "layout");
        let roles = parse!(application, Vec<&str>, SECTION_NAME, "roles");
        let panels = parse!(application, Vec<&str>, SECTION_NAME, "panels");

        // Check if there are any errors in the parsing
        let r#type: Option<ApplicationType> = match r#type {
            Ok((r#type, r#type_pos)) => ApplicationType::from_str(r#type)
                .map_err(|_| {
                    errors.push(SagError::language_error(
                        LanguageErrorKind::InvalidValue(r#type.to_string()),
                        r#type_pos,
                    ));
                })
                .ok(),
            Err(e) => {
                errors.push(e);
                None
            }
        };
        let dashboard: Option<DashboardType> = match dashboard {
            Ok((dashboard, dashboard_pos)) => DashboardType::from_str(dashboard)
                .map_err(|_| {
                    errors.push(SagError::language_error(
                        LanguageErrorKind::InvalidValue(dashboard.to_string()),
                        dashboard_pos,
                    ));
                })
                .ok(),
            Err(e) => {
                errors.push(e);
                None
            }
        };
        let layout: Option<Layout> = match layout {
            Ok((layout, layout_pos)) => Layout::from_str(layout)
                .map_err(|_| {
                    errors.push(SagError::language_error(
                        LanguageErrorKind::InvalidValue(layout.to_string()),
                        layout_pos,
                    ));
                })
                .ok(),
            Err(e) => {
                errors.push(e);
                None
            }
        };
        let roles: Option<Vec<&str>> = match roles {
            Ok((roles, _)) => Some(roles),
            Err(e) => {
                errors.push(e);
                None
            }
        };
        let panels: Option<(Vec<&str>, Position)> = match panels {
            Ok((panels, pos)) => Some((panels, pos)),
            Err(e) => {
                errors.push(e);
                None
            }
        };

        // If there are any errors, return them
        if !errors.is_empty() {
            return Err(errors);
        }

        // Return the type, dashboard, layout, roles and panels
        Ok((
            r#type.unwrap(),
            dashboard.unwrap(),
            layout.unwrap(),
            roles.unwrap(),
            panels.unwrap(),
        ))
    }

    pub fn new(blocks: &mut Blocks<'a>) -> Result<Self, Vec<SagError>> {
        // Check if the application section is correctly defined
        let (r#type, dashboard, layout, roles, panels) = Application::check(blocks)?;
        let mut errors = Vec::new();

        // Create a panel for each panel in the application
        let mut panels_map = HashMap::new();

        for panel_name in panels.0 {
            let panel = PanelTypeUnion::new(blocks, panel_name, panels.1);
            if let Err(e) = panel {
                errors.extend(e);
            } else {
                panels_map.insert(panel_name, panel.unwrap());
            }
        }
        if !errors.is_empty() {
            return Err(errors);
        }

        // Check if the panels are valid for the dashboard type
        Application::check_panels(&panels_map, &dashboard, &panels.1)?;

        // Return the application section
        Ok(Application {
            r#type,
            dashboard,
            layout,
            roles,
            panels: panels_map,
        })
    }

    // Check if the panels are valid for the dashboard type
    fn check_panels(
        panels: &HashMap<&str, PanelTypeUnion>,
        dashboard: &DashboardType,
        position: &Position,
    ) -> Result<(), Vec<SagError>> {
        let mut errors = Vec::new();

        // If the dashboard is Dash, check if the panels are valid, otherwise return Err()
        // If the dashboard is Grafana, return Ok()
        match dashboard {
            DashboardType::Dash => {
                for panel in panels.values() {
                    if !matches!(
                        panel,
                        PanelTypeUnion::PieChart(_)
                            | PanelTypeUnion::TimeSeries(_)
                            | PanelTypeUnion::BarChart(_)
                            | PanelTypeUnion::GeoMap(_)
                            | PanelTypeUnion::XYChart(_)
                    ) {
                        errors.push(SagError::language_error(
                            LanguageErrorKind::InvalidPanelType(panel.to_string()),
                            *position,
                        ));
                    }
                }
                if !errors.is_empty() {
                    return Err(errors);
                }
                Ok(())
            }
            DashboardType::Grafana => Ok(()),
        }
    }
}

impl<'a> PanelTypeUnion<'a> {
    // Check if the panel type is correctly defined
    // Return the panel type or errors if there are any
    fn check(
        blocks: &mut Blocks<'a>,
        block_name: &'a str,
        block_ref_position: Position,
    ) -> Result<PanelType, Vec<SagError>> {
        let mut errors = Vec::new();

        // Get the panel type from the blocks
        let panels = blocks.get_mut(block_name).ok_or(SagError::language_error(
            LanguageErrorKind::IncorrectPanelName(block_name.to_string()),
            block_ref_position,
        ));

        // If there is an error, return it
        if let Err(e) = panels {
            errors.push(e);
            return Err(errors);
        }

        let panels = panels.unwrap();

        // Check if the panel type is formatted correctly
        if !panels.value.is_block() {
            errors.push(SagError::language_error(
                LanguageErrorKind::InvalidType(),
                panels.position,
            ));
            return Err(errors);
        }

        // Parse the panel type
        let panel_type = parse!(panels, &str, block_name, "type");

        // Check if there are any errors in the parsing
        if let Err(e) = panel_type {
            errors.push(e);
            return Err(errors);
        }

        // Return the panel type
        let panel_type = panel_type.unwrap();
        let panel_type = PanelType::from_str(panel_type.0).map_err(|_| {
            SagError::language_error(
                LanguageErrorKind::InvalidValue(panel_type.0.to_string()),
                panel_type.1,
            )
        });
        if let Err(e) = panel_type {
            errors.push(e);
            return Err(errors);
        }
        Ok(panel_type.unwrap())
    }

    // Create a new panel type, return the panel type
    pub fn new(
        blocks: &mut Blocks<'a>,
        block_name: &'a str,
        block_ref_position: Position,
    ) -> Result<Self, Vec<SagError>> {
        let panel_type = PanelTypeUnion::check(blocks, block_name, block_ref_position)?;

        // Match the panel type and create the corresponding panel
        let panel_type_union = match panel_type {
            PanelType::PieChart => PanelTypeUnion::PieChart(PieChart::new(blocks, block_name)?),
            PanelType::TimeSeries => {
                PanelTypeUnion::TimeSeries(TimeSeries::new(blocks, block_name)?)
            }
            PanelType::BarChart => PanelTypeUnion::BarChart(BarChart::new(blocks, block_name)?),
            PanelType::GeoMap => PanelTypeUnion::GeoMap(GeoMap::new(blocks, block_name)?),
            PanelType::XYChart => PanelTypeUnion::XYChart(XYChart::new(blocks, block_name)?),
            PanelType::GrafanaMap => {
                PanelTypeUnion::GrafanaMap(GrafanaMap::new(blocks, block_name)?)
            }
            PanelType::GrafanaSingleLine => {
                PanelTypeUnion::GrafanaSingleLine(GrafanaSingleLine::new(blocks, block_name)?)
            }
            PanelType::GrafanaMultiLine => {
                PanelTypeUnion::GrafanaMultiLine(GrafanaMultiLine::new(blocks, block_name)?)
            }
            PanelType::GrafanaExtValues => {
                PanelTypeUnion::GrafanaExtValues(GrafanaExtValues::new(blocks, block_name)?)
            }
            PanelType::GrafanaCalendar => {
                PanelTypeUnion::GrafanaCalendar(GrafanaCalendar::new(blocks, block_name)?)
            }
            PanelType::GrafanaBnB => {
                PanelTypeUnion::GrafanaBnB(GrafanaBnB::new(blocks, block_name)?)
            }
            PanelType::GrafanaBulletGraph => {
                PanelTypeUnion::GrafanaBulletGraph(GrafanaBulletGraph::new(blocks, block_name)?)
            }
        };

        // Return the panel type
        Ok(panel_type_union)
    }
}

impl<'a> GeoMap<'a> {
    // Check if the GeoMap section is correctly defined
    // Return the label, type, source, data, area or errors if there are any
    fn check(
        blocks: &mut Blocks<'a>,
        block_name: &'a str,
    ) -> Result<
        (
            &'a str,
            PanelType,
            &'a str,
            Vec<&'a str>,
            Option<&'a str>,
            Option<&'a str>,
        ),
        Vec<SagError>,
    > {
        let mut errors = Vec::new();
        // Get the GeoMap section from the blocks
        let geomap = blocks.get_mut(block_name).unwrap();

        // Parse the label, type, source, data and area
        let label = parse!(geomap, &str, block_name, "label");
        let r#type = parse!(geomap, &str, block_name, "type");
        let source = parse!(geomap, &str, block_name, "source");
        let data = parse!(geomap, Vec<&str>, block_name, "data");
        let area = parse!(geomap, Option<&str>, block_name, "area");
        let color_by = parse!(geomap, Option<&str>, block_name, "color_by"); // Parse new field

        // Check if there are any errors in the parsing
        let label = match label {
            Ok((label, _)) => Some(label),
            Err(e) => {
                errors.push(e);
                None
            }
        };
        let r#type = match r#type {
            Ok((r#type, r#type_pos)) => PanelType::from_str(r#type)
                .map_err(|_| {
                    errors.push(SagError::language_error(
                        LanguageErrorKind::InvalidValue(r#type.to_string()),
                        r#type_pos,
                    ))
                })
                .ok(),
            Err(e) => {
                errors.push(e);
                None
            }
        };
        let source = match source {
            Ok((source, _)) => Some(source),
            Err(e) => {
                errors.push(e);
                None
            }
        };
        let data = match data {
            Ok((data, _)) => Some(data),
            Err(e) => {
                errors.push(e);
                None
            }
        };
        let area = area.map(|area| area.0);

        // If there are any errors, return them
        if !errors.is_empty() {
            return Err(errors);
        }

        // Return the label, type, source, data and area
        Ok((
            label.unwrap(),
            r#type.unwrap(),
            source.unwrap(),
            data.unwrap(),
            area,
            color_by.map(|v| v.0),
        ))
    }

    // Create a new GeoMap section, return the label, type, source, data and area
    pub fn new(blocks: &mut Blocks<'a>, block_name: &'a str) -> Result<Self, Vec<SagError>> {
        let (label, r#type, source, data, area, color_by) = GeoMap::check(blocks, block_name)?;
        Ok(GeoMap {
            label,
            r#type,
            source,
            data,
            area,
            color_by,
        })
    }
}

impl<'a> GrafanaMap<'a> {
    // Check if the GrafanaMap section is correctly defined
    // Return the type, source, traces or errors if there are any
    fn check(
        blocks: &mut Blocks<'a>,
        block_name: &'a str,
    ) -> Result<(PanelType, &'a str, Vec<&'a str>), Vec<SagError>> {
        let mut errors = Vec::new();

        // Get the GrafanaMap section from the blocks
        let grafanaMap = blocks.get_mut(block_name).unwrap();

        // Parse the type, source and traces
        let r#type = parse!(grafanaMap, &str, block_name, "type");
        let source = parse!(grafanaMap, &str, block_name, "source");
        let traces = parse!(grafanaMap, Vec<&str>, block_name, "traces");

        // Check if there are any errors in the parsing
        let r#type = match r#type {
            Ok((r#type, r#type_pos)) => PanelType::from_str(r#type)
                .map_err(|_| {
                    errors.push(SagError::language_error(
                        LanguageErrorKind::InvalidValue(r#type.to_string()),
                        r#type_pos,
                    ))
                })
                .ok(),
            Err(e) => {
                errors.push(e);
                None
            }
        };
        let source = match source {
            Ok((source, _)) => Some(source),
            Err(e) => {
                errors.push(e);
                None
            }
        };
        let traces = match traces {
            Ok((traces, _)) => Some(traces),
            Err(e) => {
                errors.push(e);
                None
            }
        };

        // If there are any errors, return them
        if !errors.is_empty() {
            return Err(errors);
        }

        // Return the type, source and traces
        Ok((r#type.unwrap(), source.unwrap(), traces.unwrap()))
    }

    // Create a new GrafanaMap section, return the type, source and traces
    pub fn new(blocks: &mut Blocks<'a>, block_name: &'a str) -> Result<Self, Vec<SagError>> {
        let (r#type, source, traces) = GrafanaMap::check(blocks, block_name)?;

        Ok(GrafanaMap {
            r#type,
            source,
            traces,
        })
    }
}

impl<'a> PieChart<'a> {
    // Check if the PieChart section is correctly defined
    // Return the label, type, source, traces, pie_chart_type or errors if there are any
    fn check(
        blocks: &mut Blocks<'a>,
        block_name: &'a str,
    ) -> Result<
        (
            &'a str,
            PanelType,
            &'a str,
            Vec<&'a str>,
            Option<PieChartType>,
        ),
        Vec<SagError>,
    > {
        let mut errors = Vec::new();

        // Get the PieChart section from the blocks
        let pieChart = blocks.get_mut(block_name).unwrap();

        // Parse the label, type, source, traces, pie_chart_type
        let label = parse!(pieChart, &str, block_name, "label");
        let r#type = parse!(pieChart, &str, block_name, "type");
        let source = parse!(pieChart, &str, block_name, "source");
        let traces = parse!(pieChart, Vec<&str>, block_name, "traces");
        let pie_chart_type = parse!(pieChart, Option<&str>, block_name, "pie_chart_type");

        // Check if there are any errors in the parsing
        let pie_chart_type = match pie_chart_type {
            Some(pie_chart_type) => PieChartType::from_str(pie_chart_type.0)
                .map_err(|_| {
                    errors.push(SagError::language_error(
                        LanguageErrorKind::InvalidValue(pie_chart_type.0.to_string()),
                        pie_chart_type.1,
                    ))
                })
                .ok(),
            None => None,
        };
        let r#type = match r#type {
            Ok((r#type, r#type_pos)) => PanelType::from_str(r#type)
                .map_err(|_| {
                    errors.push(SagError::language_error(
                        LanguageErrorKind::InvalidValue(r#type.to_string()),
                        r#type_pos,
                    ))
                })
                .ok(),
            Err(e) => {
                errors.push(e);
                None
            }
        };
        let label = match label {
            Ok((label, _)) => Some(label),
            Err(e) => {
                errors.push(e);
                None
            }
        };
        let source = match source {
            Ok((source, _)) => Some(source),
            Err(e) => {
                errors.push(e);
                None
            }
        };
        let traces = match traces {
            Ok((traces, _)) => Some(traces),
            Err(e) => {
                errors.push(e);
                None
            }
        };

        // If there are any errors, return them
        if !errors.is_empty() {
            return Err(errors);
        }

        // Return the label, type, source, traces and pie_chart_type
        Ok((
            label.unwrap(),
            r#type.unwrap(),
            source.unwrap(),
            traces.unwrap(),
            pie_chart_type,
        ))
    }

    // Create a new PieChart section, return the label, type, source, traces and pie_chart_type
    pub fn new(blocks: &mut Blocks<'a>, block_name: &'a str) -> Result<Self, Vec<SagError>> {
        let (label, r#type, source, traces, pie_chart_type) = PieChart::check(blocks, block_name)?;

        Ok(PieChart {
            label,
            r#type,
            source,
            traces,
            pie_chart_type,
        })
    }
}

impl<'a> BarChart<'a> {
    // Check if the BarChart section is correctly defined
    // Return the label, type, source, traces or errors if there are any
    fn check(
        blocks: &mut Blocks<'a>,
        block_name: &'a str,
    ) -> Result<(&'a str, PanelType, &'a str, Vec<&'a str>), Vec<SagError>> {
        let mut errors = Vec::new();
        let barChart = blocks.get_mut(block_name).unwrap(); // Get the BarChart section

        // Parse the label, type, source, traces
        let label = parse!(barChart, &str, block_name, "label");
        let r#type = parse!(barChart, &str, block_name, "type");
        let source = parse!(barChart, &str, block_name, "source");
        let traces = parse!(barChart, Vec<&str>, block_name, "traces");

        // Check if there are any errors in the parsing
        let label = match label {
            Ok((label, _)) => Some(label),
            Err(e) => {
                errors.push(e);
                None
            }
        };
        let r#type = match r#type {
            Ok((r#type, r#type_pos)) => PanelType::from_str(r#type)
                .map_err(|_| {
                    errors.push(SagError::language_error(
                        LanguageErrorKind::InvalidValue(r#type.to_string()),
                        r#type_pos,
                    ))
                })
                .ok(),
            Err(e) => {
                errors.push(e);
                None
            }
        };
        let source = match source {
            Ok((source, _)) => Some(source),
            Err(e) => {
                errors.push(e);
                None
            }
        };
        let traces = match traces {
            Ok((traces, _)) => Some(traces),
            Err(e) => {
                errors.push(e);
                None
            }
        };

        // If there are any errors, return them
        if !errors.is_empty() {
            return Err(errors);
        }

        // Return the label, type, source and traces
        Ok((
            label.unwrap(),
            r#type.unwrap(),
            source.unwrap(),
            traces.unwrap(),
        ))
    }

    // Create a new BarChart section, return the label, type, source and traces
    pub fn new(blocks: &mut Blocks<'a>, block_name: &'a str) -> Result<Self, Vec<SagError>> {
        let (label, r#type, source, traces) = BarChart::check(blocks, block_name)?;

        Ok(BarChart {
            label,
            r#type,
            source,
            traces,
        })
    }
}

impl<'a> TimeSeries<'a> {
    // Check if the TimeSeries section is correctly defined
    // Return the label, type, source, traces or errors if there are any
    fn check(
        blocks: &mut Blocks<'a>,
        block_name: &'a str,
    ) -> Result<(&'a str, PanelType, &'a str, Vec<&'a str>), Vec<SagError>> {
        let mut errors = Vec::new();
        let timeSeries = blocks.get_mut(block_name).unwrap(); // Get the TimeSeries section

        // Parse the label, type, source, traces
        let label = parse!(timeSeries, &str, block_name, "label");
        let r#type = parse!(timeSeries, &str, block_name, "type");
        let source = parse!(timeSeries, &str, block_name, "source");
        let traces = parse!(timeSeries, Vec<&str>, block_name, "traces");

        // Check if there are any errors in the parsing
        let label = match label {
            Ok((label, _)) => Some(label),
            Err(e) => {
                errors.push(e);
                None
            }
        };
        let r#type = match r#type {
            Ok((r#type, r#type_pos)) => PanelType::from_str(r#type)
                .map_err(|_| {
                    errors.push(SagError::language_error(
                        LanguageErrorKind::InvalidValue(r#type.to_string()),
                        r#type_pos,
                    ))
                })
                .ok(),
            Err(e) => {
                errors.push(e);
                None
            }
        };
        let source = match source {
            Ok((source, _)) => Some(source),
            Err(e) => {
                errors.push(e);
                None
            }
        };
        let traces = match traces {
            Ok((traces, _)) => Some(traces),
            Err(e) => {
                errors.push(e);
                None
            }
        };

        // If there are any errors, return them
        if !errors.is_empty() {
            return Err(errors);
        }

        // Return the label, type, source and traces
        Ok((
            label.unwrap(),
            r#type.unwrap(),
            source.unwrap(),
            traces.unwrap(),
        ))
    }

    // Create a new TimeSeries section, return the label, type, source and traces
    pub fn new(blocks: &mut Blocks<'a>, block_name: &'a str) -> Result<Self, Vec<SagError>> {
        let (label, r#type, source, traces) = TimeSeries::check(blocks, block_name)?;

        Ok(TimeSeries {
            label,
            r#type,
            source,
            traces,
        })
    }
}

impl<'a> XYChart<'a> {
    // Check if the XYChart section is correctly defined
    // Return the label, type, source, traces or errors if there are any
    fn check(
        blocks: &mut Blocks<'a>,
        block_name: &'a str,
    ) -> Result<(&'a str, PanelType, &'a str, Vec<&'a str>), Vec<SagError>> {
        let mut errors = Vec::new();
        let xyChart = blocks.get_mut(block_name).unwrap(); // Get the XYChart section

        // Parse the label, type, source, traces
        let label = parse!(xyChart, &str, block_name, "label");
        let r#type = parse!(xyChart, &str, block_name, "type");
        let source = parse!(xyChart, &str, block_name, "source");
        let traces = parse!(xyChart, Vec<&str>, block_name, "traces");

        // Check if there are any errors in the parsing
        let label = match label {
            Ok((label, _)) => Some(label),
            Err(e) => {
                errors.push(e);
                None
            }
        };
        let r#type = match r#type {
            Ok((r#type, r#type_pos)) => PanelType::from_str(r#type)
                .map_err(|_| {
                    errors.push(SagError::language_error(
                        LanguageErrorKind::InvalidValue(r#type.to_string()),
                        r#type_pos,
                    ))
                })
                .ok(),
            Err(e) => {
                errors.push(e);
                None
            }
        };
        let source = match source {
            Ok((source, _)) => Some(source),
            Err(e) => {
                errors.push(e);
                None
            }
        };
        let traces = match traces {
            Ok((traces, _)) => Some(traces),
            Err(e) => {
                errors.push(e);
                None
            }
        };

        // If there are any errors, return them
        if !errors.is_empty() {
            return Err(errors);
        }

        // Return the label, type, source and traces
        Ok((
            label.unwrap(),
            r#type.unwrap(),
            source.unwrap(),
            traces.unwrap(),
        ))
    }

    // Create a new XYChart section, return the label, type, source and traces
    pub fn new(blocks: &mut Blocks<'a>, block_name: &'a str) -> Result<Self, Vec<SagError>> {
        let (label, r#type, source, traces) = XYChart::check(blocks, block_name)?;

        Ok(XYChart {
            label,
            r#type,
            source,
            traces,
        })
    }
}

impl<'a> GrafanaSingleLine<'a> {
    // Check if the GrafanaSingleLine section is correctly defined
    // Return the type, source, traces or errors if there are any
    fn check(
        blocks: &mut Blocks<'a>,
        block_name: &'a str,
    ) -> Result<(PanelType, &'a str, Vec<&'a str>), Vec<SagError>> {
        let mut errors = Vec::new();
        let grafanaSingleLine = blocks.get_mut(block_name).unwrap(); // Get the GrafanaSingleLine section

        // Parse the type, source, traces
        let r#type = parse!(grafanaSingleLine, &str, block_name, "type");
        let source = parse!(grafanaSingleLine, &str, block_name, "source");
        let traces = parse!(grafanaSingleLine, Vec<&str>, block_name, "traces");

        // Check if there are any errors in the parsing
        let r#type = match r#type {
            Ok((r#type, r#type_pos)) => PanelType::from_str(r#type)
                .map_err(|_| {
                    errors.push(SagError::language_error(
                        LanguageErrorKind::InvalidValue(r#type.to_string()),
                        r#type_pos,
                    ))
                })
                .ok(),
            Err(e) => {
                errors.push(e);
                None
            }
        };
        let source = match source {
            Ok((source, _)) => Some(source),
            Err(e) => {
                errors.push(e);
                None
            }
        };
        let traces = match traces {
            Ok((traces, _)) => Some(traces),
            Err(e) => {
                errors.push(e);
                None
            }
        };

        // If there are any errors, return them
        if !errors.is_empty() {
            return Err(errors);
        }

        // Return the type, source and traces
        Ok((r#type.unwrap(), source.unwrap(), traces.unwrap()))
    }

    // Create a new GrafanaSingleLine section, return the type, source and traces
    pub fn new(blocks: &mut Blocks<'a>, block_name: &'a str) -> Result<Self, Vec<SagError>> {
        let (r#type, source, traces) = GrafanaSingleLine::check(blocks, block_name)?;

        Ok(GrafanaSingleLine {
            r#type,
            source,
            traces,
        })
    }
}

impl<'a> GrafanaMultiLine<'a> {
    // Check if the GrafanaMultiLine section is correctly defined
    // Return the type, source, locations, traces or errors if there are any
    fn check(
        blocks: &mut Blocks<'a>,
        block_name: &'a str,
    ) -> Result<(PanelType, &'a str, Vec<&'a str>, Vec<&'a str>), Vec<SagError>> {
        let mut errors = Vec::new();
        let grafanaMultiLine = blocks.get_mut(block_name).unwrap(); // Get the GrafanaMultiLine section

        // Parse the type, source, locations, traces
        let r#type = parse!(grafanaMultiLine, &str, block_name, "type");
        let source = parse!(grafanaMultiLine, &str, block_name, "source");
        let locations = parse!(grafanaMultiLine, Vec<&str>, block_name, "locations");
        let traces = parse!(grafanaMultiLine, Vec<&str>, block_name, "traces");

        // Check if there are any errors in the parsing
        let r#type = match r#type {
            Ok((r#type, r#type_pos)) => PanelType::from_str(r#type)
                .map_err(|_| {
                    errors.push(SagError::language_error(
                        LanguageErrorKind::InvalidValue(r#type.to_string()),
                        r#type_pos,
                    ))
                })
                .ok(),
            Err(e) => {
                errors.push(e);
                None
            }
        };
        let source = match source {
            Ok((source, _)) => Some(source),
            Err(e) => {
                errors.push(e);
                None
            }
        };
        let locations = match locations {
            Ok((locations, _)) => Some(locations),
            Err(e) => {
                errors.push(e);
                None
            }
        };
        let traces = match traces {
            Ok((traces, _)) => Some(traces),
            Err(e) => {
                errors.push(e);
                None
            }
        };

        // If there are any errors, return them
        if !errors.is_empty() {
            return Err(errors);
        }

        // Return the type, source, locations and traces
        Ok((
            r#type.unwrap(),
            source.unwrap(),
            locations.unwrap(),
            traces.unwrap(),
        ))
    }

    // Create a new GrafanaMultiLine section, return the type, source, locations and traces
    pub fn new(blocks: &mut Blocks<'a>, block_name: &'a str) -> Result<Self, Vec<SagError>> {
        let (r#type, source, locations, traces) = GrafanaMultiLine::check(blocks, block_name)?;

        Ok(GrafanaMultiLine {
            r#type,
            source,
            locations,
            traces,
        })
    }
}

impl<'a> GrafanaExtValues<'a> {
    // Check if the GrafanaExtValues section is correctly defined
    // Return the type, source, locations, traces or errors if there are any
    fn check(
        blocks: &mut Blocks<'a>,
        block_name: &'a str,
    ) -> Result<(PanelType, &'a str, Vec<&'a str>, Vec<&'a str>), Vec<SagError>> {
        let mut errors = Vec::new();
        let grafanaExtraValues = blocks.get_mut(block_name).unwrap(); // Get the GrafanaExtValues section

        // Parse the type, source, locations, traces
        let r#type = parse!(grafanaExtraValues, &str, block_name, "type");
        let source = parse!(grafanaExtraValues, &str, block_name, "source");
        let locations = parse!(grafanaExtraValues, Vec<&str>, block_name, "locations");
        let traces = parse!(grafanaExtraValues, Vec<&str>, block_name, "traces");

        // Check if there are any errors in the parsing
        let r#type = match r#type {
            Ok((r#type, r#type_pos)) => PanelType::from_str(r#type)
                .map_err(|_| {
                    errors.push(SagError::language_error(
                        LanguageErrorKind::InvalidValue(r#type.to_string()),
                        r#type_pos,
                    ))
                })
                .ok(),
            Err(e) => {
                errors.push(e);
                None
            }
        };
        let source = match source {
            Ok((source, _)) => Some(source),
            Err(e) => {
                errors.push(e);
                None
            }
        };
        let locations = match locations {
            Ok((locations, _)) => Some(locations),
            Err(e) => {
                errors.push(e);
                None
            }
        };
        let traces = match traces {
            Ok((traces, _)) => Some(traces),
            Err(e) => {
                errors.push(e);
                None
            }
        };

        // If there are any errors, return them
        if !errors.is_empty() {
            return Err(errors);
        }

        // Return the type, source, locations and traces
        Ok((
            r#type.unwrap(),
            source.unwrap(),
            locations.unwrap(),
            traces.unwrap(),
        ))
    }

    // Create a new GrafanaExtValues section, return the type, source, locations and traces
    pub fn new(blocks: &mut Blocks<'a>, block_name: &'a str) -> Result<Self, Vec<SagError>> {
        let (r#type, source, locations, traces) = GrafanaExtValues::check(blocks, block_name)?;

        Ok(GrafanaExtValues {
            r#type,
            source,
            locations,
            traces,
        })
    }
}

impl<'a> GrafanaCalendar<'a> {
    // Check if the GrafanaCalendar section is correctly defined
    // Return the type, source, locations, traces or errors if there are any
    fn check(
        blocks: &mut Blocks<'a>,
        block_name: &'a str,
    ) -> Result<(PanelType, &'a str, Vec<&'a str>, Vec<&'a str>), Vec<SagError>> {
        let mut errors = Vec::new();
        let grafanaCalendar = blocks.get_mut(block_name).unwrap(); // Get the GrafanaCalendar section

        // Parse the type, source, locations, traces
        let r#type = parse!(grafanaCalendar, &str, block_name, "type");
        let source = parse!(grafanaCalendar, &str, block_name, "source");
        let locations = parse!(grafanaCalendar, Vec<&str>, block_name, "locations");
        let traces = parse!(grafanaCalendar, Vec<&str>, block_name, "traces");

        // Check if there are any errors in the parsing
        let r#type = match r#type {
            Ok((r#type, r#type_pos)) => PanelType::from_str(r#type)
                .map_err(|_| {
                    errors.push(SagError::language_error(
                        LanguageErrorKind::InvalidValue(r#type.to_string()),
                        r#type_pos,
                    ))
                })
                .ok(),
            Err(e) => {
                errors.push(e);
                None
            }
        };
        let source = match source {
            Ok((source, _)) => Some(source),
            Err(e) => {
                errors.push(e);
                None
            }
        };
        let locations = match locations {
            Ok((locations, _)) => Some(locations),
            Err(e) => {
                errors.push(e);
                None
            }
        };
        let traces = match traces {
            Ok((traces, _)) => Some(traces),
            Err(e) => {
                errors.push(e);
                None
            }
        };

        // If there are any errors, return them
        if !errors.is_empty() {
            return Err(errors);
        }

        // Return the type, source, locations and traces
        Ok((
            r#type.unwrap(),
            source.unwrap(),
            locations.unwrap(),
            traces.unwrap(),
        ))
    }

    // Create a new GrafanaCalendar section, return the type, source, locations and traces
    pub fn new(blocks: &mut Blocks<'a>, block_name: &'a str) -> Result<Self, Vec<SagError>> {
        let (r#type, source, locations, traces) = GrafanaCalendar::check(blocks, block_name)?;

        Ok(GrafanaCalendar {
            r#type,
            source,
            locations,
            traces,
        })
    }
}

impl<'a> GrafanaBulletGraph<'a> {
    // Check if the GrafanaBulletGraph section is correctly defined
    // Return the type, source, locations, traces or errors if there are any
    fn check(
        blocks: &mut Blocks<'a>,
        block_name: &'a str,
    ) -> Result<(PanelType, &'a str, Vec<&'a str>, Vec<&'a str>), Vec<SagError>> {
        let mut errors = Vec::new();
        let grafanaBulletGraph = blocks.get_mut(block_name).unwrap(); // Get the GrafanaBulletGraph section

        // Parse the type, source, locations, traces
        let r#type = parse!(grafanaBulletGraph, &str, block_name, "type");
        let source = parse!(grafanaBulletGraph, &str, block_name, "source");
        let locations = parse!(grafanaBulletGraph, Vec<&str>, block_name, "locations");
        let traces = parse!(grafanaBulletGraph, Vec<&str>, block_name, "traces");

        // Check if there are any errors in the parsing
        let r#type = match r#type {
            Ok((r#type, r#type_pos)) => PanelType::from_str(r#type)
                .map_err(|_| {
                    errors.push(SagError::language_error(
                        LanguageErrorKind::InvalidValue(r#type.to_string()),
                        r#type_pos,
                    ))
                })
                .ok(),
            Err(e) => {
                errors.push(e);
                None
            }
        };
        let source = match source {
            Ok((source, _)) => Some(source),
            Err(e) => {
                errors.push(e);
                None
            }
        };
        let locations = match locations {
            Ok((locations, _)) => Some(locations),
            Err(e) => {
                errors.push(e);
                None
            }
        };
        let traces = match traces {
            Ok((traces, _)) => Some(traces),
            Err(e) => {
                errors.push(e);
                None
            }
        };

        // If there are any errors, return them
        if !errors.is_empty() {
            return Err(errors);
        }

        // Return the type, source, locations and traces
        Ok((
            r#type.unwrap(),
            source.unwrap(),
            locations.unwrap(),
            traces.unwrap(),
        ))
    }

    // Create a new GrafanaBulletGraph section, return the type, source, locations and traces
    pub fn new(blocks: &mut Blocks<'a>, block_name: &'a str) -> Result<Self, Vec<SagError>> {
        dbg!("GrafanaBulletGraph::new");
        let (r#type, source, locations, traces) = GrafanaBulletGraph::check(blocks, block_name)?;

        Ok(GrafanaBulletGraph {
            r#type,
            source,
            locations,
            traces,
        })
    }
}

impl<'a> Deployment<'a> {
    // Check if the Deployment section is correctly defined
    // Return the environments or errors if there are any
    fn check(blocks: &mut Blocks<'a>) -> Result<(Vec<&'a str>, Position), Vec<SagError>> {
        const SECTION_NAME: &str = "deployment";
        let mut errors = Vec::new();

        // Get the Deployment section from the blocks
        let deployment = blocks
            .get_mut(SECTION_NAME)
            .ok_or(SagError::internal_error(format!(
                "Missing section {SECTION_NAME}"
            )));

        // Check if there are any errors in the parsing
        if let Err(e) = deployment {
            errors.push(e);
            return Err(errors);
        }

        let deployment = deployment.unwrap();

        // Check if the Deployment section is structured correctly
        if !deployment.value.is_block() {
            errors.push(SagError::language_error(
                LanguageErrorKind::InvalidType(),
                deployment.position,
            ));
            return Err(errors);
        }

        // Parse the environments
        let environments = parse!(deployment, Vec<&str>, SECTION_NAME, "environments");

        // Check if there are any errors in the parsing
        if let Err(e) = environments {
            errors.push(e);
            return Err(errors);
        }

        // Return the environments
        Ok(environments.unwrap())
    }

    // Create a new Deployment section, return the environments
    pub fn new(blocks: &mut Blocks<'a>) -> Result<Self, Vec<SagError>> {
        let environments = Deployment::check(blocks)?;
        let mut errors = Vec::new();

        // Create environments map
        let mut environments_map = HashMap::new();
        for environment_name in environments.0 {
            let environment = Environment::new(blocks, environment_name, environments.1);
            if let Err(e) = environment {
                errors.extend(e.into_iter());
            } else {
                environments_map.insert(environment_name, environment.unwrap());
            }
        }

        // If there are any errors, return them
        if !errors.is_empty() {
            return Err(errors);
        }

        // Create a new Deployment section
        Ok(Deployment {
            environments: environments_map,
        })
    }
}

impl<'a> Environment<'a> {
    // Check if the Environment section is correctly defined
    // Return the uri, port, type or errors if there are any
    fn check(
        blocks: &mut Blocks<'a>,
        block_name: &'a str,
        block_ref_pos: Position,
    ) -> Result<(&'a str, i32, EnvironmentType), Vec<SagError>> {
        let mut errors = Vec::new();

        // Get the Environment section from the blocks
        let environment = blocks.get_mut(block_name).ok_or(SagError::language_error(
            LanguageErrorKind::MissingSection(block_name.to_string()),
            block_ref_pos,
        ));

        // Check if there are any errors in the parsing
        if let Err(e) = environment {
            errors.push(e);
            return Err(errors);
        }

        let environment = environment.unwrap();

        // Check if the Environment section is structured correctly
        if !environment.value.is_block() {
            errors.push(SagError::language_error(
                LanguageErrorKind::InvalidType(),
                environment.position,
            ));
            return Err(errors);
        }

        // Parse the uri, port, type
        let uri = parse!(environment, &str, block_name, "uri");
        let port = parse!(environment, &str, block_name, "port");
        let r#type = parse!(environment, &str, block_name, "type");

        // Check if there are any errors in the parsing
        let port = match port {
            Ok((port, _)) => port
                .parse::<i32>()
                .map_err(|_| {
                    errors.push(SagError::language_error(
                        LanguageErrorKind::InvalidValue(port.to_string()),
                        environment.position,
                    ))
                })
                .ok(),
            Err(e) => {
                errors.push(e);
                None
            }
        };
        let uri = match uri {
            Ok((uri, _)) => Some(uri),
            Err(e) => {
                errors.push(e);
                None
            }
        };
        let r#type = match r#type {
            Ok((r#type, r#type_pos)) => EnvironmentType::from_str(r#type)
                .map_err(|_| {
                    errors.push(SagError::language_error(
                        LanguageErrorKind::InvalidValue(r#type.to_string()),
                        r#type_pos,
                    ))
                })
                .ok(),
            Err(e) => {
                errors.push(e);
                None
            }
        };

        // If there are any errors, return them
        if !errors.is_empty() {
            return Err(errors);
        }

        // Return the uri, port and type
        Ok((uri.unwrap(), port.unwrap(), r#type.unwrap()))
    }

    // Create a new Environment section, return the uri, port and type
    pub fn new(
        blocks: &mut Blocks<'a>,
        block_name: &'a str,
        block_ref_pos: Position,
    ) -> Result<Self, Vec<SagError>> {
        let (uri, port, r#type) = Environment::check(blocks, block_name, block_ref_pos)?;

        Ok(Environment { uri, port, r#type })
    }
}

impl<'a> GrafanaBnB<'a> {
    // Check if the GrafanaBnB section is correctly defined
    // Return the type, source, locations, traces or errors if there are any
    fn check(
        blocks: &mut Blocks<'a>,
        block_name: &'a str,
    ) -> Result<(PanelType, &'a str, Vec<&'a str>, Vec<&'a str>), Vec<SagError>> {
        let mut errors = Vec::new();
        let grafanaBNB = blocks.get_mut(block_name).unwrap(); // Get the GrafanaBnB section

        // Parse the type, source, locations, traces
        let r#type = parse!(grafanaBNB, &str, block_name, "type");
        let source = parse!(grafanaBNB, &str, block_name, "source");
        let locations = parse!(grafanaBNB, Vec<&str>, block_name, "locations");
        let traces = parse!(grafanaBNB, Vec<&str>, block_name, "traces");

        // Check if there are any errors in the parsing
        let r#type = match r#type {
            Ok((r#type, r#type_pos)) => PanelType::from_str(r#type)
                .map_err(|_| {
                    errors.push(SagError::language_error(
                        LanguageErrorKind::InvalidValue(r#type.to_string()),
                        r#type_pos,
                    ))
                })
                .ok(),
            Err(e) => {
                errors.push(e);
                None
            }
        };
        let source = match source {
            Ok((source, _)) => Some(source),
            Err(e) => {
                errors.push(e);
                None
            }
        };
        let locations = match locations {
            Ok((locations, _)) => Some(locations),
            Err(e) => {
                errors.push(e);
                None
            }
        };
        let traces = match traces {
            Ok((traces, _)) => Some(traces),
            Err(e) => {
                errors.push(e);
                None
            }
        };

        // If there are any errors, return them
        if !errors.is_empty() {
            return Err(errors);
        }

        // Return the type, source, locations and traces
        Ok((
            r#type.unwrap(),
            source.unwrap(),
            locations.unwrap(),
            traces.unwrap(),
        ))
    }

    // Create a new GrafanaBnB section, return the type, source, locations and traces
    pub fn new(blocks: &mut Blocks<'a>, block_name: &'a str) -> Result<Self, Vec<SagError>> {
        let (r#type, source, locations, traces) = GrafanaBnB::check(blocks, block_name)?;

        Ok(GrafanaBnB {
            r#type,
            source,
            locations,
            traces,
        })
    }
}

// -----------Section 3: Panels Implementation------------
// This section contains the implementation of the Panel trait for each PanelTypeUnion
// TODO: Will it stay here or be deleted?

// Shared implementation for all PanelTypeUnion
trait Panel {
    fn get_source(&self) -> &str;
    fn get_label(&self) -> Option<&str>;
    fn get_traces(&self) -> &Vec<&str>;
    fn get_locations(&self) -> Option<&Vec<&str>>;
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
            PanelTypeUnion::GrafanaSingleLine(gsl) => gsl.source,
            PanelTypeUnion::GrafanaMultiLine(gml) => gml.source,
            PanelTypeUnion::GrafanaExtValues(gextv) => gextv.source,
            PanelTypeUnion::GrafanaCalendar(gc) => gc.source,
            PanelTypeUnion::GrafanaBnB(bb) => bb.source,
            PanelTypeUnion::GrafanaBulletGraph(gbg) => gbg.source,
        }
    }

    fn get_label(&self) -> Option<&str> {
        match self {
            PanelTypeUnion::GeoMap(map) => Some(map.label),
            PanelTypeUnion::XYChart(xy) => Some(xy.label),
            PanelTypeUnion::PieChart(pie) => Some(pie.label),
            PanelTypeUnion::BarChart(bar) => Some(bar.label),
            PanelTypeUnion::TimeSeries(ts) => Some(ts.label),
            PanelTypeUnion::GrafanaMap(_) => None,
            PanelTypeUnion::GrafanaSingleLine(_) => None,
            PanelTypeUnion::GrafanaMultiLine(_) => None,
            PanelTypeUnion::GrafanaExtValues(_) => None,
            PanelTypeUnion::GrafanaCalendar(_) => None,
            PanelTypeUnion::GrafanaBnB(_) => None,
            PanelTypeUnion::GrafanaBulletGraph(_) => None,
        }
    }

    fn get_traces(&self) -> &Vec<&str> {
        match self {
            PanelTypeUnion::GeoMap(map) => &map.data,
            PanelTypeUnion::XYChart(xy) => &xy.traces,
            PanelTypeUnion::PieChart(pie) => &pie.traces,
            PanelTypeUnion::BarChart(bar) => &bar.traces,
            PanelTypeUnion::TimeSeries(ts) => &ts.traces,
            PanelTypeUnion::GrafanaMap(gm) => &gm.traces,
            PanelTypeUnion::GrafanaSingleLine(gsl) => &gsl.traces,
            PanelTypeUnion::GrafanaMultiLine(gml) => &gml.traces,
            PanelTypeUnion::GrafanaExtValues(gextv) => &gextv.traces,
            PanelTypeUnion::GrafanaCalendar(gc) => &gc.traces,
            PanelTypeUnion::GrafanaBnB(bb) => &bb.traces,
            PanelTypeUnion::GrafanaBulletGraph(gbg) => &gbg.traces,
        }
    }

    fn get_locations(&self) -> Option<&Vec<&str>> {
        match self {
            PanelTypeUnion::GeoMap(_) => None,
            PanelTypeUnion::XYChart(_) => None,
            PanelTypeUnion::PieChart(_) => None,
            PanelTypeUnion::BarChart(_) => None,
            PanelTypeUnion::TimeSeries(_) => None,
            PanelTypeUnion::GrafanaMap(_) => None,
            PanelTypeUnion::GrafanaSingleLine(_) => None,
            PanelTypeUnion::GrafanaMultiLine(gml) => Some(&gml.locations),
            PanelTypeUnion::GrafanaExtValues(gextv) => Some(&gextv.locations),
            PanelTypeUnion::GrafanaCalendar(gc) => Some(&gc.locations),
            PanelTypeUnion::GrafanaBnB(bb) => Some(&bb.locations),
            PanelTypeUnion::GrafanaBulletGraph(gbg) => Some(&gbg.locations),
        }
    }
}

// -----------Section 4: FromStr Implementations------------
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
            "Dataskop" => Ok(Provider::Dataskop),
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

impl FromStr for DashboardType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Dash" => Ok(DashboardType::Dash),
            "Grafana" => Ok(DashboardType::Grafana),
            _ => Err(format!("invalid dashboard type: {}", s)),
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
            "smartcomm-map-panel" => Ok(PanelType::GrafanaMap),
            "smartcomm-simpleline-panel" => Ok(PanelType::GrafanaSingleLine),
            "smartcomm-multiplelinechart-panel" => Ok(PanelType::GrafanaMultiLine),
            "smartcomm-extremevalues-panel" => Ok(PanelType::GrafanaExtValues),
            "smartcomm-calendar-panel" => Ok(PanelType::GrafanaCalendar),
            "smartcomm-minmaxbarchart-panel" => Ok(PanelType::GrafanaBnB),
            "smartcomm-bulletgraph-panel" => Ok(PanelType::GrafanaBulletGraph),
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
            "Azure" => Ok(EnvironmentType::Azure),
            _ => Err(format!("invalid environment type: {}", s)),
        }
    }
}

// ToString implementations

impl Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl Display for Scope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Scope::Service => write!(f, "service"),
            Scope::Industry => write!(f, "industry"),
            Scope::Manifacturing => write!(f, "manifacturing"),
            Scope::Education => write!(f, "education"),
            Scope::Healthcare => write!(f, "healthcare"),
            Scope::SocialPrograms => write!(f, "social_programs"),
            Scope::Government => write!(f, "government"),
            Scope::Energy => write!(f, "energy"),
            Scope::Water => write!(f, "water"),
            Scope::Environment => write!(f, "environment"),
            Scope::Transportation => write!(f, "transportation"),
            Scope::Communication => write!(f, "communication"),
            Scope::PublicSafety => write!(f, "public_safety"),
            Scope::UrbanPlanning => write!(f, "urban_planning"),
            Scope::Infrastructure => write!(f, "infrastructure"),
        }
    }
}

impl Display for SourceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SourceType::SmartMeter => write!(f, "SmartMeter"),
            SourceType::Sensor => write!(f, "Sensor"),
        }
    }
}

impl Display for Provider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Provider::Fiware => write!(f, "Fiware"),
            Provider::Dataskop => write!(f, "Dataskop"),
        }
    }
}

impl Display for ApplicationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApplicationType::Web => write!(f, "Web"),
            ApplicationType::Mobile => write!(f, "Mobile"),
            ApplicationType::Desktop => write!(f, "Desktop"),
            ApplicationType::Server => write!(f, "Server"),
        }
    }
}

impl Display for DashboardType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DashboardType::Dash => write!(f, "Dash"),
            DashboardType::Grafana => write!(f, "Grafana"),
        }
    }
}

impl Display for Layout {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Layout::Horizontal => write!(f, "Horizontal"),
            Layout::Vertical => write!(f, "Vertical"),
            Layout::SinglePage => write!(f, "SinglePage"),
        }
    }
}

impl Display for PanelTypeUnion<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PanelTypeUnion::PieChart(_) => write!(f, "pie_chart"),
            PanelTypeUnion::TimeSeries(_) => write!(f, "timeseries"),
            PanelTypeUnion::BarChart(_) => write!(f, "bar_chart"),
            PanelTypeUnion::GeoMap(_) => write!(f, "geomap"),
            PanelTypeUnion::XYChart(_) => write!(f, "xy_chart"),
            PanelTypeUnion::GrafanaMap(_) => write!(f, "smartcomm-map-panel"),
            PanelTypeUnion::GrafanaSingleLine(_) => write!(f, "smartcomm-simpleline-panel"),
            PanelTypeUnion::GrafanaMultiLine(_) => {
                write!(f, "smartcomm-multiplelinechart-panel")
            }
            PanelTypeUnion::GrafanaExtValues(_) => write!(f, "smartcomm-extremevalues-panel"),
            PanelTypeUnion::GrafanaCalendar(_) => write!(f, "smartcomm-calendar-panel"),
            PanelTypeUnion::GrafanaBnB(_) => write!(f, "smartcomm-minmaxbarchart-panel"),
            PanelTypeUnion::GrafanaBulletGraph(_) => write!(f, "smartcomm-bulletgraph-panel"),
        }
    }
}

impl Display for PanelType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PanelType::PieChart => write!(f, "pie_chart"),
            PanelType::TimeSeries => write!(f, "timeseries"),
            PanelType::BarChart => write!(f, "bar_chart"),
            PanelType::GeoMap => write!(f, "geomap"),
            PanelType::XYChart => write!(f, "xy_chart"),
            PanelType::GrafanaMap => write!(f, "smartcomm-map-panel"),
            PanelType::GrafanaSingleLine => write!(f, "smartcomm-simpleline-panel"),
            PanelType::GrafanaMultiLine => write!(f, "smartcomm-multiplelinechart-panel"),
            PanelType::GrafanaExtValues => write!(f, "smartcomm-extremevalues-panel"),
            PanelType::GrafanaCalendar => write!(f, "smartcomm-calendar-panel"),
            PanelType::GrafanaBnB => write!(f, "smartcomm-minmaxbarchart-panel"),
            PanelType::GrafanaBulletGraph => write!(f, "smartcomm-bulletgraph-panel"),
        }
    }
}

impl Display for PieChartType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PieChartType::Pie => write!(f, "pie"),
            PieChartType::Donut => write!(f, "donut"),
        }
    }
}

impl Display for EnvironmentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EnvironmentType::Docker => write!(f, "Docker"),
            EnvironmentType::Azure => write!(f, "Azure"),
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
            Provider::Dataskop => "Siemens",
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
            PanelTypeUnion::GrafanaMap(_) => "smartcomm-map-panel",
            PanelTypeUnion::GrafanaSingleLine(_) => "smartcomm-simpleline-panel",
            PanelTypeUnion::GrafanaMultiLine(_) => "smartcomm-multiplelinechart-panel",
            PanelTypeUnion::GrafanaExtValues(_) => "smartcomm-extremevalues-panel",
            PanelTypeUnion::GrafanaCalendar(_) => "smartcomm-calendar-panel",
            PanelTypeUnion::GrafanaBnB(_) => "smartcomm-minmaxbarchart-panel",
            PanelTypeUnion::GrafanaBulletGraph(_) => "smartcomm-bulletgraph-panel",
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
            PanelType::GrafanaMap => "smartcomm-map-panel",
            PanelType::GrafanaSingleLine => "smartcomm-simpleline-panel",
            PanelType::GrafanaMultiLine => "smartcomm-multiplelinechart-panel",
            PanelType::GrafanaExtValues => "smartcomm-extremevalues-panel",
            PanelType::GrafanaCalendar => "smartcomm-calendar-panel",
            PanelType::GrafanaBnB => "smartcomm-minmaxbarchart-panel",
            PanelType::GrafanaBulletGraph => "smartcomm-bulletgraph-panel",
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
            EnvironmentType::Azure => "Azure",
        }
    }
}
