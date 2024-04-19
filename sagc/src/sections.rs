use nom::bytes::complete::tag;
use nom::character::complete::i16;
use nom::multi::separated_list1;
use nom::IResult;
use std::collections::HashMap;
use std::str::FromStr;
use url::Url;

use crate::errors::LanguageErrorKind;
use crate::errors::SagError;
use crate::parse;
use crate::parser::Blocks;
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
    GrafanaSingleLine(GrafanaSingleLine<'a>),
    GrafanaMultiLine(GrafanaMultiLine<'a>),
    GrafanaExtValues(GrafanaExtValues<'a>),
    GrafanaCalendar(GrafanaCalendar<'a>),
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
    pub fn new(blocks: Blocks<'a>) -> Result<Self, Vec<SagError>> {
        let mut errors = Vec::new();

        let application = Application::new(&blocks);
        let service = Service::new(&blocks);
        let data = SagData::new(&blocks);
        let deployment = Deployment::new(&blocks);

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
}

impl<'a> Service<'a> {
    fn check(blocks: &Blocks<'a>) -> Result<(&'a str, Scope, Version), Vec<SagError>> {
        const SECTION_NAME: &str = "service";
        let mut errors = Vec::new();

        let block = blocks
            .get(SECTION_NAME)
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

    fn new(blocks: &Blocks<'a>) -> Result<Self, Vec<SagError>> {
        let (title, scope, version) = Service::check(blocks)?;

        Ok(Service {
            title,
            scope,
            version,
        })
    }
}

impl<'a> SagData<'a> {
    fn check(blocks: &Blocks<'a>) -> Result<(Vec<&'a str>, Position), Vec<SagError>> {
        const SECTION_NAME: &str = "data";
        let mut errors = Vec::new();

        let data = blocks
            .get(SECTION_NAME)
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

    fn new(blocks: &Blocks<'a>) -> Result<Self, Vec<SagError>> {
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
        blocks: &Blocks<'a>,
        source_name: &'a str,
        source_name_position: Position,
    ) -> Result<(Provider, SourceType, Url, &'a str), Vec<SagError>> {
        let mut errors = Vec::new();

        let datasource = blocks.get(source_name).ok_or(SagError::language_error(
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

        if !errors.is_empty() {
            return Err(errors);
        }

        Ok((
            provider.unwrap(),
            r#type.unwrap(),
            uri.unwrap(),
            query.unwrap(),
        ))
    }

    fn new(
        blocks: &Blocks<'a>,
        source_name: &'a str,
        source_name_position: Position,
    ) -> Result<Self, Vec<SagError>> {
        let (provider, r#type, uri, query) =
            Datasource::check(blocks, source_name, source_name_position)?;

        Ok(Datasource {
            provider,
            r#type,
            uri,
            query,
        })
    }
}

impl<'a> Application<'a> {
    fn check(
        blocks: &Blocks<'a>,
    ) -> Result<
        (
            ApplicationType,
            Layout,
            Vec<&'a str>,
            (Vec<&'a str>, Position),
        ),
        Vec<SagError>,
    > {
        const SECTION_NAME: &str = "application";
        let mut errors = Vec::new();

        let block = blocks
            .get(SECTION_NAME)
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
            layout.unwrap(),
            roles.unwrap(),
            panels.unwrap(),
        ))
    }

    fn new(blocks: &Blocks<'a>) -> Result<Self, Vec<SagError>> {
        let (r#type, layout, roles, panels) = Application::check(blocks)?;
        let mut errors = Vec::new();

        let mut panels_map = HashMap::new();

        for panel_name in panels.0 {
            let panel = PanelTypeUnion::new(blocks, panel_name, panels.1);
            if let Err(e) = panel {
                errors.extend(e.into_iter());
            } else {
                panels_map.insert(panel_name, panel.unwrap());
            }
        }

        if !errors.is_empty() {
            return Err(errors);
        }

        Ok(Application {
            r#type,
            layout,
            roles,
            panels: panels_map,
        })
    }
}

impl<'a> PanelTypeUnion<'a> {
    fn check(
        blocks: &'a Blocks<'a>,
        block_name: &'a str,
        block_ref_position: Position,
    ) -> Result<PanelType, Vec<SagError>> {
        let mut errors = Vec::new();

        let block = blocks.get(block_name).ok_or(SagError::language_error(
            LanguageErrorKind::MissingSection(block_name.to_string()),
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
        blocks: &Blocks<'a>,
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
        };
        Ok(panel_type_union)
    }
}

impl<'a> GeoMap<'a> {
    fn check(
        blocks: &Blocks<'a>,
        block_name: &'a str,
    ) -> Result<(&'a str, PanelType, &'a str, Vec<&'a str>, Option<&'a str>), Vec<SagError>> {
        let mut errors = Vec::new();

        let block = blocks.get(block_name).unwrap();
        let label = parse!(block, &str, block_name, "label");
        let r#type = parse!(block, &str, block_name, "type");
        let source = parse!(block, &str, block_name, "source");
        let data = parse!(block, Vec<&str>, block_name, "data");
        let area = parse!(block, Option<&str>, block_name, "label");

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

    fn new(blocks: &Blocks<'a>, block_name: &'a str) -> Result<Self, Vec<SagError>> {
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
        blocks: &Blocks<'a>,
        block_name: &'a str,
    ) -> Result<(PanelType, &'a str, Vec<&'a str>), Vec<SagError>> {
        let mut errors = Vec::new();
        let block = blocks.get(block_name).unwrap();

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

    fn new(blocks: &Blocks<'a>, block_name: &'a str) -> Result<Self, Vec<SagError>> {
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
        blocks: &Blocks<'a>,
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
        let block = blocks.get(block_name).unwrap();

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

    fn new(blocks: &Blocks<'a>, block_name: &'a str) -> Result<Self, Vec<SagError>> {
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
        blocks: &Blocks<'a>,
        block_name: &'a str,
    ) -> Result<(&'a str, PanelType, &'a str, Vec<&'a str>), Vec<SagError>> {
        let mut errors = Vec::new();
        let block = blocks.get(block_name).unwrap();

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

    fn new(blocks: &Blocks<'a>, block_name: &'a str) -> Result<Self, Vec<SagError>> {
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
        blocks: &Blocks<'a>,
        block_name: &'a str,
    ) -> Result<(&'a str, PanelType, &'a str, Vec<&'a str>), Vec<SagError>> {
        let mut errors = Vec::new();
        let block = blocks.get(block_name).unwrap();

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

    fn new(blocks: &Blocks<'a>, block_name: &'a str) -> Result<Self, Vec<SagError>> {
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
        blocks: &Blocks<'a>,
        block_name: &'a str,
    ) -> Result<(&'a str, PanelType, &'a str, Vec<&'a str>), Vec<SagError>> {
        let mut errors = Vec::new();
        let block = blocks.get(block_name).unwrap();

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

    fn new(blocks: &Blocks<'a>, block_name: &'a str) -> Result<Self, Vec<SagError>> {
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
        blocks: &Blocks<'a>,
        block_name: &'a str,
    ) -> Result<(PanelType, &'a str, Vec<&'a str>), Vec<SagError>> {
        let mut errors = Vec::new();
        let block = blocks.get(block_name).unwrap();

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

    fn new(blocks: &Blocks<'a>, block_name: &'a str) -> Result<Self, Vec<SagError>> {
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
    fn check(blocks: &Blocks<'a>) -> Result<(Vec<&'a str>, Position), Vec<SagError>> {
        const SECTION_NAME: &str = "deployment";
        let mut errors = Vec::new();

        let block = blocks
            .get(SECTION_NAME)
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

    fn new(blocks: &Blocks<'a>) -> Result<Self, Vec<SagError>> {
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
        blocks: &Blocks<'a>,
        block_name: &'a str,
        block_ref_pos: Position,
    ) -> Result<(&'a str, i32, EnvironmentType), Vec<SagError>> {
        let mut errors = Vec::new();

        let block = blocks.get(block_name).ok_or(SagError::language_error(
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
        blocks: &Blocks<'a>,
        block_name: &'a str,
        block_ref_pos: Position,
    ) -> Result<Self, Vec<SagError>> {
        let (uri, port, r#type) = Environment::check(blocks, block_name, block_ref_pos)?;

        Ok(Environment { uri, port, r#type })
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
            "smartcomm-map-panel" => Ok(PanelType::GrafanaMap),
            "smartcomm-simpleline-panel" => Ok(PanelType::GrafanaSingleLine),
            "smartcomm-multiplelinechart-panel" => Ok(PanelType::GrafanaMultiLine),
            "smartcomm-extremevalues-panel" => Ok(PanelType::GrafanaExtValues),
            "smartcomm-calendar-panel" => Ok(PanelType::GrafanaCalendar),
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
            PanelTypeUnion::GrafanaMap(_) => String::from("smartcomm-map-panel"),
            PanelTypeUnion::GrafanaSingleLine(_) => String::from("smartcomm-simpleline-panel"),
            PanelTypeUnion::GrafanaMultiLine(_) => {
                String::from("smartcomm-multiplelinechart-panel")
            }
            PanelTypeUnion::GrafanaExtValues(_) => String::from("smartcomm-extremevalues-panel"),
            PanelTypeUnion::GrafanaCalendar(_) => String::from("smartcomm-calendar-panel"),
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
            PanelType::GrafanaMap => String::from("smartcomm-map-panel"),
            PanelType::GrafanaSingleLine => String::from("smartcomm-simpleline-panel"),
            PanelType::GrafanaMultiLine => String::from("smartcomm-multiplelinechart-panel"),
            PanelType::GrafanaExtValues => String::from("smartcomm-extremevalues-panel"),
            PanelType::GrafanaCalendar => String::from("smartcomm-calendar-panel"),
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
            PanelTypeUnion::GrafanaMap(_) => "smartcomm-map-panel",
            PanelTypeUnion::GrafanaSingleLine(_) => "smartcomm-simpleline-panel",
            PanelTypeUnion::GrafanaMultiLine(_) => "smartcomm-multiplelinechart-panel",
            PanelTypeUnion::GrafanaExtValues(_) => "smartcomm-extremevalues-panel",
            PanelTypeUnion::GrafanaCalendar(_) => "smartcomm-calendar-panel",
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
