use nom::bytes::complete::tag;
use nom::character::complete::i16;
use nom::multi::separated_list1;
use nom::IResult;
use std::collections::HashMap;
use std::fmt::Display;
use std::str::FromStr;
use std::usize;
use url::Url;

use crate::errors::LanguageErrorKind;
use crate::errors::SagError;
use crate::parse;
use crate::parser::Blocks;
use crate::parser::ParseResult;
use crate::parser::Position;
use crate::parser::Value;

#[derive(Debug)]
pub struct Config<'a> {
    pub service: Service<'a>,
    /// e.g. `<name>: <datasource>`
    pub data: SagData<'a>,
    pub application: Application<'a>,
    pub deployment: Deployment<'a>,
}

#[derive(Debug)]
pub struct Service<'a> {
    pub title: &'a str, // called "name" for Dash, can be 'name' also in this case
    pub scope: Scope,
    pub version: Version,
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
pub struct SagData<'a> {
    pub sources: HashMap<&'a str, Datasource<'a>>,
}

#[derive(Debug)]
pub struct Datasource<'a> {
    pub provider: Provider,
    pub r#type: SourceType,
    pub uri: Url,
    pub query: &'a str,
    pub config: Option<DatasourceConfig<'a>>,
}

#[derive(Debug)]
pub enum Provider {
    Fiware,
    Dataskop,
}

#[derive(Debug)]
pub enum SourceType {
    SmartMeter,
    Sensor,
}

#[derive(Debug)]
pub struct DatasourceConfig<'a> {
    pub company: usize,
    pub measurements: HashMap<usize, &'a str>,
    pub token: &'a str,
}

#[derive(Debug)]
pub struct Application<'a> {
    pub r#type: ApplicationType,
    pub dashboard: DashboardType,
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

#[derive(Debug)]
pub enum PanelTypeUnion<'a> {
    PieChart(PieChart<'a>),
    TimeSeries(TimeSeries<'a>),
    BarChart(BarChart<'a>),
    GeoMap(GeoMap<'a>),
    XYChart(XYChart<'a>),
    GrafanaMap(GrafanaMap<'a>),
    GrafanaSingleLine(GrafanaSingleLine<'a>),
    GrafanaMultiLine(GrafanaMultiLine<'a>),
    GrafanaExtValues(GrafanaExtValues<'a>),
    GrafanaCalendar(GrafanaCalendar<'a>),
    GrafanaBnB(GrafanaBnB<'a>),
}

#[derive(Debug)]
pub enum PanelType {
    PieChart,
    TimeSeries,
    BarChart,
    GeoMap,
    XYChart,
    GrafanaMap,
    GrafanaSingleLine,
    GrafanaMultiLine,
    GrafanaExtValues,
    GrafanaCalendar,
    GrafanaBnB,
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
pub struct GrafanaSingleLine<'a> {
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
pub struct GrafanaExtValues<'a> {
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
pub struct GrafanaBnB<'a> {
    pub r#type: PanelType,
    pub source: &'a str,
    pub locations: Vec<&'a str>,
    pub traces: Vec<&'a str>,
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

#[derive(Debug)]
pub struct Environment<'a> {
    pub uri: &'a str,
    pub port: i32,
    pub r#type: EnvironmentType,
}

#[derive(Debug)]
pub enum EnvironmentType {
    Docker,
}

impl<'a> Config<'a> {
    pub fn new(blocks: &mut Blocks<'a>) -> Result<Self, Vec<SagError>> {
        let mut errors = Vec::new();

        let application = Application::new(blocks);
        let service = Service::new(blocks);
        let data = SagData::new(blocks);
        let deployment = Deployment::new(blocks);

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

        if !errors.is_empty() {
            return Err(errors);
        }

        let config = Config {
            service: service.unwrap(),
            data: data.unwrap(),
            application: application.unwrap(),
            deployment: deployment.unwrap(),
        };

        //Config::validate(&config)?;
        Ok(config)
    }

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
    pub fn filter_panels<F>(&mut self, filter: F)
    where
        F: Fn(&PanelTypeUnion) -> bool,
    {
        self.application.panels.retain(|_, panel| filter(panel));
    }
}

impl<'a> Service<'a> {
    fn check(blocks: &mut Blocks<'a>) -> Result<(&'a str, Scope, Version), Vec<SagError>> {
        const SECTION_NAME: &str = "service";
        let mut errors = Vec::new();

        let block = blocks
            .get_mut(SECTION_NAME)
            .ok_or(SagError::internal_error(format!(
                "Missing section {}",
                SECTION_NAME
            )));

        if let Err(e) = block {
            errors.push(e);
            return Err(errors);
        }

        let block = block.unwrap();

        if !block.value.is_block() {
            errors.push(SagError::language_error(
                LanguageErrorKind::InvalidType(),
                block.position,
            ));
            return Err(errors);
        }

        let title = parse!(block, &str, SECTION_NAME, "title");
        let scope = parse!(block, &str, SECTION_NAME, "scope");
        let version = parse!(block, &str, SECTION_NAME, "version");

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

        if !errors.is_empty() {
            return Err(errors);
        }

        Ok((title.unwrap(), scope.unwrap(), version.unwrap()))
    }

    fn new(blocks: &mut Blocks<'a>) -> Result<Self, Vec<SagError>> {
        let (title, scope, version) = Service::check(blocks)?;

        Ok(Service {
            title,
            scope,
            version,
        })
    }
}

impl<'a> SagData<'a> {
    fn check(blocks: &mut Blocks<'a>) -> Result<(Vec<&'a str>, Position), Vec<SagError>> {
        const SECTION_NAME: &str = "data";
        let mut errors = Vec::new();

        let data = blocks
            .get_mut(SECTION_NAME)
            .ok_or(SagError::internal_error(format!(
                "Missing section {}",
                SECTION_NAME
            )));

        if let Err(e) = data {
            errors.push(e);
            return Err(errors);
        }

        let data = data.unwrap();

        if !data.value.is_block() {
            errors.push(SagError::language_error(
                LanguageErrorKind::InvalidType(),
                data.position,
            ));
            return Err(errors);
        }

        let data_sources = parse!(data, Vec<&str>, SECTION_NAME, "sources");
        if let Err(e) = data_sources {
            errors.push(e);
            return Err(errors);
        }

        let data_sources = data_sources.unwrap();

        Ok(data_sources)
    }

    fn new(blocks: &mut Blocks<'a>) -> Result<Self, Vec<SagError>> {
        const SECTION_NAME: &str = "data";

        let mut errors = Vec::new();

        let data_sources = SagData::check(blocks)?;

        let mut data_sources_map = HashMap::new();

        for name in data_sources.0 {
            let datasource = Datasource::new(blocks, name, data_sources.1);
            if let Err(e) = datasource {
                errors.extend(e.into_iter());
            } else {
                data_sources_map.insert(name, datasource.unwrap());
            }
        }

        if !errors.is_empty() {
            return Err(errors);
        }

        Ok(SagData {
            sources: data_sources_map,
        })
    }
}

impl<'a> Datasource<'a> {
    fn check(
        blocks: &mut Blocks<'a>,
        source_name: &'a str,
        source_name_position: Position,
    ) -> Result<
        (
            Provider,
            SourceType,
            Url,
            &'a str,
            Option<DatasourceConfig<'a>>,
        ),
        Vec<SagError>,
    > {
        let mut errors = Vec::new();

        let datasource = blocks.get_mut(source_name).ok_or(SagError::language_error(
            LanguageErrorKind::MissingSection(source_name.to_string()),
            source_name_position,
        ));

        if let Err(e) = datasource {
            errors.push(e);
            return Err(errors);
        }

        let datasource = datasource.unwrap();

        if !datasource.value.is_block() {
            errors.push(SagError::language_error(
                LanguageErrorKind::InvalidType(),
                datasource.position,
            ));
            return Err(errors);
        }

        let provider = parse!(datasource, &str, source_name, "provider");
        let r#type = parse!(datasource, &str, source_name, "type");
        let uri = parse!(datasource, &str, source_name, "uri");
        let query = parse!(datasource, &str, source_name, "query");
        let config = DatasourceConfig::new(datasource);

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
            Ok((query, _)) => Some(query),
            Err(e) => {
                errors.push(e);
                None
            }
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

        if !errors.is_empty() {
            return Err(errors);
        }

        Ok((
            provider.unwrap(),
            r#type.unwrap(),
            uri.unwrap(),
            query.unwrap(),
            config,
        ))
    }

    fn new(
        blocks: &mut Blocks<'a>,
        source_name: &'a str,
        source_name_position: Position,
    ) -> Result<Self, Vec<SagError>> {
        let (provider, r#type, uri, query, config) =
            Datasource::check(blocks, source_name, source_name_position)?;

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
    fn check(
        datasource: &mut ParseResult<'a>,
    ) -> Result<Option<(usize, HashMap<usize, &'a str>, &'a str)>, Vec<SagError>> {
        let mut errors = Vec::new();
        const SECTION_NAME: &str = "config";

        let block = match &mut datasource.value {
            Value::Block(block) => block,
            _ => unreachable!(),
        };

        let block = block.get_mut(SECTION_NAME);

        if block.is_none() {
            return Ok(None);
        }

        let block = block.unwrap();

        if !block.value.is_block() {
            errors.push(SagError::language_error(
                LanguageErrorKind::InvalidType(),
                block.position,
            ));
            return Err(errors);
        }

        let company = parse!(block, usize, SECTION_NAME, "company");
        let measurement = parse!(block, HashMap<usize, &str>, SECTION_NAME, "measurements");
        let token = parse!(block, &str, SECTION_NAME, "token");

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

        if !errors.is_empty() {
            return Err(errors);
        }

        Ok(Some((
            company.unwrap(),
            measurement.unwrap(),
            token.unwrap(),
        )))
    }

    fn new(block: &mut ParseResult<'a>) -> Result<Option<Self>, Vec<SagError>> {
        let result = DatasourceConfig::check(block)?;
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

        let block = blocks
            .get_mut(SECTION_NAME)
            .ok_or(SagError::internal_error(format!(
                "Missing section {}",
                SECTION_NAME
            )));

        if let Err(e) = block {
            errors.push(e);
            return Err(errors);
        }
        let block = block.unwrap();

        if !block.value.is_block() {
            errors.push(SagError::language_error(
                LanguageErrorKind::InvalidType(),
                block.position,
            ));
            return Err(errors);
        }

        let r#type = parse!(block, &str, SECTION_NAME, "type");
        let dashboard = parse!(block, &str, SECTION_NAME, "dashboard");
        let layout = parse!(block, &str, SECTION_NAME, "layout");
        let roles = parse!(block, Vec<&str>, SECTION_NAME, "roles");
        let panels = parse!(block, Vec<&str>, SECTION_NAME, "panels");

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

        if !errors.is_empty() {
            return Err(errors);
        }

        Ok((
            r#type.unwrap(),
            dashboard.unwrap(),
            layout.unwrap(),
            roles.unwrap(),
            panels.unwrap(),
        ))
    }

    fn new(blocks: &mut Blocks<'a>) -> Result<Self, Vec<SagError>> {
        let (r#type, dashboard, layout, roles, panels) = Application::check(blocks)?;
        let mut errors = Vec::new();

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

        Application::check_panels(&panels_map, &dashboard, &panels.1)?;

        Ok(Application {
            r#type,
            dashboard,
            layout,
            roles,
            panels: panels_map,
        })
    }

    fn check_panels(
        panels: &HashMap<&str, PanelTypeUnion>,
        dashboard: &DashboardType,
        position: &Position,
    ) -> Result<(), Vec<SagError>> {
        let mut errors = Vec::new();
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
    fn check(
        blocks: &mut Blocks<'a>,
        block_name: &'a str,
        block_ref_position: Position,
    ) -> Result<PanelType, Vec<SagError>> {
        let mut errors = Vec::new();

        let block = blocks.get(block_name).ok_or(SagError::language_error(
            LanguageErrorKind::IncorrectPanelName(block_name.to_string()),
            block_ref_position,
        ));

        if let Err(e) = block {
            errors.push(e);
            return Err(errors);
        }

        let block = block.unwrap();

        if !block.value.is_block() {
            errors.push(SagError::language_error(
                LanguageErrorKind::InvalidType(),
                block.position,
            ));
            return Err(errors);
        }

        let panel_type = parse!(block, &str, block_name, "type");

        if let Err(e) = panel_type {
            errors.push(e);
            return Err(errors);
        }

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

    fn new(
        blocks: &mut Blocks<'a>,
        block_name: &'a str,
        block_ref_position: Position,
    ) -> Result<Self, Vec<SagError>> {
        let panel_type = PanelTypeUnion::check(blocks, block_name, block_ref_position)?;

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
        };
        Ok(panel_type_union)
    }
}

impl<'a> GeoMap<'a> {
    fn check(
        blocks: &mut Blocks<'a>,
        block_name: &'a str,
    ) -> Result<(&'a str, PanelType, &'a str, Vec<&'a str>, Option<&'a str>), Vec<SagError>> {
        let mut errors = Vec::new();

        let block = blocks.get_mut(block_name).unwrap();

        let label = parse!(block, &str, block_name, "label");
        let r#type = parse!(block, &str, block_name, "type");
        let source = parse!(block, &str, block_name, "source");
        let data = parse!(block, Vec<&str>, block_name, "data");
        let area = parse!(block, Option<&str>, block_name, "area");

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

        if !errors.is_empty() {
            return Err(errors);
        }

        Ok((
            label.unwrap(),
            r#type.unwrap(),
            source.unwrap(),
            data.unwrap(),
            area,
        ))
    }

    fn new(blocks: &mut Blocks<'a>, block_name: &'a str) -> Result<Self, Vec<SagError>> {
        let (label, r#type, source, data, area) = GeoMap::check(blocks, block_name)?;

        Ok(GeoMap {
            label,
            r#type,
            source,
            data,
            area,
        })
    }
}

impl<'a> GrafanaMap<'a> {
    fn check(
        blocks: &mut Blocks<'a>,
        block_name: &'a str,
    ) -> Result<(PanelType, &'a str, Vec<&'a str>), Vec<SagError>> {
        let mut errors = Vec::new();
        let block = blocks.get_mut(block_name).unwrap();

        let r#type = parse!(block, &str, block_name, "type");
        let source = parse!(block, &str, block_name, "source");
        let traces = parse!(block, Vec<&str>, block_name, "traces");

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

        if !errors.is_empty() {
            return Err(errors);
        }

        Ok((r#type.unwrap(), source.unwrap(), traces.unwrap()))
    }

    fn new(blocks: &mut Blocks<'a>, block_name: &'a str) -> Result<Self, Vec<SagError>> {
        let (r#type, source, traces) = GrafanaMap::check(blocks, block_name)?;

        Ok(GrafanaMap {
            r#type,
            source,
            traces,
        })
    }
}

impl<'a> PieChart<'a> {
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
        let block = blocks.get_mut(block_name).unwrap();

        let label = parse!(block, &str, block_name, "label");
        let r#type = parse!(block, &str, block_name, "type");
        let source = parse!(block, &str, block_name, "source");
        let traces = parse!(block, Vec<&str>, block_name, "traces");
        let pie_chart_type = parse!(block, Option<&str>, block_name, "pie_chart_type");

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

        if !errors.is_empty() {
            return Err(errors);
        }

        Ok((
            label.unwrap(),
            r#type.unwrap(),
            source.unwrap(),
            traces.unwrap(),
            pie_chart_type,
        ))
    }

    fn new(blocks: &mut Blocks<'a>, block_name: &'a str) -> Result<Self, Vec<SagError>> {
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
    fn check(
        blocks: &mut Blocks<'a>,
        block_name: &'a str,
    ) -> Result<(&'a str, PanelType, &'a str, Vec<&'a str>), Vec<SagError>> {
        let mut errors = Vec::new();
        let block = blocks.get_mut(block_name).unwrap();

        let label = parse!(block, &str, block_name, "label");
        let r#type = parse!(block, &str, block_name, "type");
        let source = parse!(block, &str, block_name, "source");
        let traces = parse!(block, Vec<&str>, block_name, "traces");

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

        if !errors.is_empty() {
            return Err(errors);
        }

        Ok((
            label.unwrap(),
            r#type.unwrap(),
            source.unwrap(),
            traces.unwrap(),
        ))
    }

    fn new(blocks: &mut Blocks<'a>, block_name: &'a str) -> Result<Self, Vec<SagError>> {
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
    fn check(
        blocks: &mut Blocks<'a>,
        block_name: &'a str,
    ) -> Result<(&'a str, PanelType, &'a str, Vec<&'a str>), Vec<SagError>> {
        let mut errors = Vec::new();
        let block = blocks.get_mut(block_name).unwrap();

        let label = parse!(block, &str, block_name, "label");
        let r#type = parse!(block, &str, block_name, "type");
        let source = parse!(block, &str, block_name, "source");
        let traces = parse!(block, Vec<&str>, block_name, "traces");

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

        if !errors.is_empty() {
            return Err(errors);
        }

        Ok((
            label.unwrap(),
            r#type.unwrap(),
            source.unwrap(),
            traces.unwrap(),
        ))
    }

    fn new(blocks: &mut Blocks<'a>, block_name: &'a str) -> Result<Self, Vec<SagError>> {
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
    fn check(
        blocks: &mut Blocks<'a>,
        block_name: &'a str,
    ) -> Result<(&'a str, PanelType, &'a str, Vec<&'a str>), Vec<SagError>> {
        let mut errors = Vec::new();
        let block = blocks.get_mut(block_name).unwrap();

        let label = parse!(block, &str, block_name, "label");
        let r#type = parse!(block, &str, block_name, "type");
        let source = parse!(block, &str, block_name, "source");
        let traces = parse!(block, Vec<&str>, block_name, "traces");

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

        if !errors.is_empty() {
            return Err(errors);
        }

        Ok((
            label.unwrap(),
            r#type.unwrap(),
            source.unwrap(),
            traces.unwrap(),
        ))
    }

    fn new(blocks: &mut Blocks<'a>, block_name: &'a str) -> Result<Self, Vec<SagError>> {
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
    fn check(
        blocks: &mut Blocks<'a>,
        block_name: &'a str,
    ) -> Result<(PanelType, &'a str, Vec<&'a str>), Vec<SagError>> {
        let mut errors = Vec::new();
        let block = blocks.get_mut(block_name).unwrap();

        let r#type = parse!(block, &str, block_name, "type");
        let source = parse!(block, &str, block_name, "source");
        let traces = parse!(block, Vec<&str>, block_name, "traces");

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

        if !errors.is_empty() {
            return Err(errors);
        }

        Ok((r#type.unwrap(), source.unwrap(), traces.unwrap()))
    }

    fn new(blocks: &mut Blocks<'a>, block_name: &'a str) -> Result<Self, Vec<SagError>> {
        let (r#type, source, traces) = GrafanaSingleLine::check(blocks, block_name)?;

        Ok(GrafanaSingleLine {
            r#type,
            source,
            traces,
        })
    }
}

impl<'a> GrafanaMultiLine<'a> {
    fn check(
        blocks: &mut Blocks<'a>,
        block_name: &'a str,
    ) -> Result<(PanelType, &'a str, Vec<&'a str>, Vec<&'a str>), Vec<SagError>> {
        let mut errors = Vec::new();
        let block = blocks.get_mut(block_name).unwrap();

        let r#type = parse!(block, &str, block_name, "type");
        let source = parse!(block, &str, block_name, "source");
        let locations = parse!(block, Vec<&str>, block_name, "locations");
        let traces = parse!(block, Vec<&str>, block_name, "traces");

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

        if !errors.is_empty() {
            return Err(errors);
        }

        Ok((
            r#type.unwrap(),
            source.unwrap(),
            locations.unwrap(),
            traces.unwrap(),
        ))
    }

    fn new(blocks: &mut Blocks<'a>, block_name: &'a str) -> Result<Self, Vec<SagError>> {
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
    fn check(
        blocks: &mut Blocks<'a>,
        block_name: &'a str,
    ) -> Result<(PanelType, &'a str, Vec<&'a str>, Vec<&'a str>), Vec<SagError>> {
        let mut errors = Vec::new();
        let block = blocks.get_mut(block_name).unwrap();

        let r#type = parse!(block, &str, block_name, "type");
        let source = parse!(block, &str, block_name, "source");
        let locations = parse!(block, Vec<&str>, block_name, "locations");
        let traces = parse!(block, Vec<&str>, block_name, "traces");

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

        if !errors.is_empty() {
            return Err(errors);
        }

        Ok((
            r#type.unwrap(),
            source.unwrap(),
            locations.unwrap(),
            traces.unwrap(),
        ))
    }

    fn new(blocks: &mut Blocks<'a>, block_name: &'a str) -> Result<Self, Vec<SagError>> {
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
    fn check(
        blocks: &mut Blocks<'a>,
        block_name: &'a str,
    ) -> Result<(PanelType, &'a str, Vec<&'a str>, Vec<&'a str>), Vec<SagError>> {
        let mut errors = Vec::new();
        let block = blocks.get_mut(block_name).unwrap();

        let r#type = parse!(block, &str, block_name, "type");
        let source = parse!(block, &str, block_name, "source");
        let locations = parse!(block, Vec<&str>, block_name, "locations");
        let traces = parse!(block, Vec<&str>, block_name, "traces");

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

        if !errors.is_empty() {
            return Err(errors);
        }

        Ok((
            r#type.unwrap(),
            source.unwrap(),
            locations.unwrap(),
            traces.unwrap(),
        ))
    }

    fn new(blocks: &mut Blocks<'a>, block_name: &'a str) -> Result<Self, Vec<SagError>> {
        let (r#type, source, locations, traces) = GrafanaCalendar::check(blocks, block_name)?;

        Ok(GrafanaCalendar {
            r#type,
            source,
            locations,
            traces,
        })
    }
}

impl<'a> Deployment<'a> {
    fn check(blocks: &mut Blocks<'a>) -> Result<(Vec<&'a str>, Position), Vec<SagError>> {
        const SECTION_NAME: &str = "deployment";
        let mut errors = Vec::new();

        let block = blocks
            .get_mut(SECTION_NAME)
            .ok_or(SagError::internal_error(format!(
                "Missing section {SECTION_NAME}"
            )));
        if let Err(e) = block {
            errors.push(e);
            return Err(errors);
        }

        let block = block.unwrap();

        if !block.value.is_block() {
            errors.push(SagError::language_error(
                LanguageErrorKind::InvalidType(),
                block.position,
            ));
            return Err(errors);
        }

        let environments = parse!(block, Vec<&str>, SECTION_NAME, "environments");

        if let Err(e) = environments {
            errors.push(e);
            return Err(errors);
        }

        Ok(environments.unwrap())
    }

    fn new(blocks: &mut Blocks<'a>) -> Result<Self, Vec<SagError>> {
        let environments = Deployment::check(blocks)?;
        let mut errors = Vec::new();

        let mut environments_map = HashMap::new();
        for environment_name in environments.0 {
            let environment = Environment::new(blocks, environment_name, environments.1);
            if let Err(e) = environment {
                errors.extend(e.into_iter());
            } else {
                environments_map.insert(environment_name, environment.unwrap());
            }
        }

        if !errors.is_empty() {
            return Err(errors);
        }

        Ok(Deployment {
            environments: environments_map,
        })
    }
}

impl<'a> Environment<'a> {
    fn check(
        blocks: &mut Blocks<'a>,
        block_name: &'a str,
        block_ref_pos: Position,
    ) -> Result<(&'a str, i32, EnvironmentType), Vec<SagError>> {
        let mut errors = Vec::new();

        let block = blocks.get_mut(block_name).ok_or(SagError::language_error(
            LanguageErrorKind::MissingSection(block_name.to_string()),
            block_ref_pos,
        ));

        if let Err(e) = block {
            errors.push(e);
            return Err(errors);
        }

        let block = block.unwrap();

        if !block.value.is_block() {
            errors.push(SagError::language_error(
                LanguageErrorKind::InvalidType(),
                block.position,
            ));
            return Err(errors);
        }

        let uri = parse!(block, &str, block_name, "uri");
        let port = parse!(block, &str, block_name, "port");
        let r#type = parse!(block, &str, block_name, "type");

        let port = match port {
            Ok((port, _)) => port
                .parse::<i32>()
                .map_err(|_| {
                    errors.push(SagError::language_error(
                        LanguageErrorKind::InvalidValue(port.to_string()),
                        block.position,
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

        if !errors.is_empty() {
            return Err(errors);
        }

        Ok((uri.unwrap(), port.unwrap(), r#type.unwrap()))
    }

    fn new(
        blocks: &mut Blocks<'a>,
        block_name: &'a str,
        block_ref_pos: Position,
    ) -> Result<Self, Vec<SagError>> {
        let (uri, port, r#type) = Environment::check(blocks, block_name, block_ref_pos)?;

        Ok(Environment { uri, port, r#type })
    }
}

impl<'a> GrafanaBnB<'a> {
    fn check(
        blocks: &Blocks<'a>,
        block_name: &'a str,
    ) -> Result<(PanelType, &'a str, Vec<&'a str>, Vec<&'a str>), Vec<SagError>> {
        let mut errors = Vec::new();
        let block = blocks.get(block_name).unwrap();

        let r#type = parse!(block, &str, block_name, "type");
        let source = parse!(block, &str, block_name, "source");
        let locations = parse!(block, Vec<&str>, block_name, "locations");
        let traces = parse!(block, Vec<&str>, block_name, "traces");

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

        if !errors.is_empty() {
            return Err(errors);
        }

        Ok((
            r#type.unwrap(),
            source.unwrap(),
            locations.unwrap(),
            traces.unwrap(),
        ))
    }

    fn new(blocks: &Blocks<'a>, block_name: &'a str) -> Result<Self, Vec<SagError>> {
        let (r#type, source, locations, traces) = GrafanaBnB::check(blocks, block_name)?;

        Ok(GrafanaBnB {
            r#type,
            source,
            locations,
            traces,
        })
    }
}


// Panel implementation

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
            "smartcomm-bars-and-bubbles" => Ok(PanelType::GrafanaBnB),
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
            Provider::Dataskop => write!(f, "Siemens"),
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
            PanelTypeUnion::GrafanaExtValues(_) => String::from("smartcomm-extremevalues-panel"),
            PanelTypeUnion::GrafanaCalendar(_) => String::from("smartcomm-calendar-panel"),
            PanelTypeUnion::GrafanaBnB(_) => String::from("smartcomm-bars-and-bubbles"),
        }
    }
}

impl Display for PanelType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PanelType::PieChart => String::from("pie_chart"),
            PanelType::TimeSeries => String::from("timeseries"),
            PanelType::BarChart => String::from("bar_chart"),
            PanelType::GeoMap => String::from("geomap"),
            PanelType::XYChart => String::from("xy_chart"),
            PanelType::GrafanaMap => String::from("smartcomm-map-panel"),
            PanelType::GrafanaSingleLine => String::from("smartcomm-simpleline-panel"),
            PanelType::GrafanaMultiLine => String::from("smartcomm-multiplelinechart-panel"),
            PanelType::GrafanaExtValues => String::from("smartcomm-extremevalues-panel"),
            PanelType::GrafanaCalendar => String::from("smartcomm-calendar-panel"),
            PanelType::GrafanaBnB => String::from("smartcomm-bars-and-bubbles"),
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
            PanelTypeUnion::GrafanaBnB(_) => "smartcomm-bars-and-bubbles",
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
            PanelType::GrafanaBnB => "smartcomm-bars-and-bubbles",
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
