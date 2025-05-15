use crate::sections::{
    Application, BarChart, Config, DatasourceConfig, Deployment, Environment, GeoMap,
    PanelTypeUnion, PieChart, Service, TimeSeries, Version, XYChart,
};

use serde::Serialize;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Serialize)]
struct DashService {
    name: String,
    scope: String,
    version: DashVersion,
}

#[derive(Debug, Serialize)]
struct DashVersion {
    major: i16,
    minor: i16,
    patch: i16,
}

#[derive(Debug, Serialize)]
struct DashDatasource {
    provider: String,
    uri: String,
    query: DashQuery,
    #[serde(rename = "type")]
    datasource_type: String,
    config: Option<DatasourceConfigDash>,
}

#[derive(Debug, Serialize)]
struct DatasourceConfigDash {
    company: usize,
    measurements: HashMap<usize, String>,
    token: String,
}

#[derive(Debug, Serialize)]
struct DashQuery {
    r#type: Option<String>,
    select: Vec<String>,
}

#[derive(Debug, Serialize)]
struct DashData {
    sources: HashMap<String, DashDatasource>,
}

#[derive(Debug, Serialize)]
struct DashGeoMap {
    #[serde(rename = "type")]
    chart_type: String,
    source: String,
    data: Vec<String>,
    name: String,
    area: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")] // Don't serialize if None
    color_by: Option<String>, // New optional field
    #[serde(skip_serializing_if = "Option::is_none")] // Don't serialize if None
    geometry_type: Option<String>, // New optional field
}

#[derive(Debug, Serialize)]
struct DashPieChart {
    #[serde(rename = "type")]
    chart_type: String,
    source: String,
    traces: Vec<String>,
    #[serde(default)]
    pie_chart_type: Option<String>,
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    reduce: Option<String>, // New optional field
}

#[derive(Debug, Serialize)]
struct DashBarChart {
    #[serde(rename = "type")]
    chart_type: String,
    source: String,
    traces: Vec<String>,
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    reduce: Option<String>, // New optional field
}

#[derive(Debug, Serialize)]
struct DashTimeSeries {
    #[serde(rename = "type")]
    chart_type: String,
    source: String,
    traces: Vec<String>,
    name: String,
}

#[derive(Debug, Serialize)]
struct DashXYChart {
    #[serde(rename = "type")]
    chart_type: String,
    source: String,
    traces: Vec<String>,
    name: String,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
enum DashPanel {
    #[serde(rename = "pie_chart")]
    PieChart(DashPieChart),
    #[serde(rename = "timeseries")]
    TimeSeries(DashTimeSeries),
    #[serde(rename = "bar_chart")]
    BarChart(DashBarChart),
    #[serde(rename = "geomap")]
    GeoMap(DashGeoMap),
    #[serde(rename = "xy_chart")]
    XYChart(DashXYChart),
}

#[derive(Debug, Serialize)]
struct DashApplication {
    #[serde(rename = "type")]
    app_type: String,
    layout: String,
    roles: Vec<String>,
    visualizations: HashMap<String, DashPanel>,
}

#[derive(Debug, Serialize)]
struct DashDeployment {
    environments: HashMap<String, DashEnvironment>,
}

#[derive(Debug, Serialize)]
struct DashEnvironment {
    uri: String,
    port: i32,
    #[serde(rename = "type")]
    deploy_type: String,
}

#[derive(Debug, Serialize)]
pub struct Dash {
    service: DashService,
    data_sources: HashMap<String, DashDatasource>,
    application: DashApplication,
    deployment: DashDeployment,
}

fn get_traces_with_data_source_name(
    data_source_name: &str,
    traces: &HashMap<&str, PanelTypeUnion>,
) -> HashSet<String> {
    traces
        .values()
        .filter(|trace| match trace {
            PanelTypeUnion::PieChart(pie_chart) => pie_chart.source == data_source_name,
            PanelTypeUnion::TimeSeries(time_series) => time_series.source == data_source_name,
            PanelTypeUnion::BarChart(bar_chart) => bar_chart.source == data_source_name,
            PanelTypeUnion::GeoMap(geo_map) => geo_map.source == data_source_name,
            PanelTypeUnion::XYChart(xy_chart) => xy_chart.source == data_source_name,
            _ => unreachable!("All unsupported panels should be filtered out by now."),
        })
        .flat_map(|trace| match trace {
            PanelTypeUnion::PieChart(pie_chart) => pie_chart.traces.iter(),
            PanelTypeUnion::TimeSeries(time_series) => time_series.traces.iter(),
            PanelTypeUnion::BarChart(bar_chart) => bar_chart.traces.iter(),
            PanelTypeUnion::GeoMap(geo_map) => geo_map.data.iter(),
            PanelTypeUnion::XYChart(xy_chart) => xy_chart.traces.iter(),
            _ => unreachable!("All unsupported panels should be filtered out by now."),
        })
        .map(|f| f.to_string())
        .collect()
}

impl<'a> From<Config<'a>> for Dash {
    fn from(mut config: Config<'a>) -> Self {
        config.filter_panels(|panel| {
            matches!(
                panel,
                PanelTypeUnion::PieChart(_)
                    | PanelTypeUnion::TimeSeries(_)
                    | PanelTypeUnion::BarChart(_)
                    | PanelTypeUnion::GeoMap(_)
                    | PanelTypeUnion::XYChart(_)
            )
        });
        // Map Config fields to Dash fields
        let service = config.service.into();
        let data_sources = config
            .data
            .sources
            .into_iter()
            .map(|(name, datasource)| {
                let traces = get_traces_with_data_source_name(name, &config.application.panels);
                let grafana_datasource = DashDatasource {
                    provider: datasource.provider.to_string(),
                    uri: datasource.uri.to_string(),
                    query: DashQuery {
                        r#type: datasource.query.map(|q| q.to_string()),
                        select: traces.iter().map(|s| s.to_string()).collect(),
                    },
                    datasource_type: datasource.r#type.to_string(),
                    config: datasource.config.map(|f| f.into()),
                };
                (name.to_string(), grafana_datasource)
            })
            .collect();

        let application = config.application.into();
        let deployment = config.deployment.into();
        Dash {
            service,
            data_sources,
            application,
            deployment,
        }
    }
}

impl From<Service<'_>> for DashService {
    fn from(val: Service<'_>) -> Self {
        DashService {
            name: val.title.to_string(),
            scope: val.scope.to_string(),
            version: val.version.into(),
        }
    }
}

impl From<Version> for DashVersion {
    fn from(val: Version) -> Self {
        DashVersion {
            major: val.major,
            minor: val.minor,
            patch: val.patch,
        }
    }
}

impl<'a> From<DatasourceConfig<'a>> for DatasourceConfigDash {
    fn from(val: DatasourceConfig) -> Self {
        DatasourceConfigDash {
            company: val.company,
            measurements: val
                .measurements
                .into_iter()
                .map(|(k, v)| (k, v.to_string()))
                .collect(),
            token: val.token.to_string(),
        }
    }
}

impl From<GeoMap<'_>> for DashGeoMap {
    fn from(value: GeoMap<'_>) -> Self {
        DashGeoMap {
            chart_type: value.r#type.to_string(),
            source: value.source.to_string(),
            data: value.data.iter().map(|f| f.to_string()).collect(),
            name: value.label.to_string(),
            area: value.area.map(|f| f.to_string()),
            color_by: value.color_by.map(|s| s.to_string()),
            geometry_type: value.geometry_type.map(|s| s.to_string()),
        }
    }
}

impl From<PieChart<'_>> for DashPieChart {
    fn from(value: PieChart<'_>) -> Self {
        DashPieChart {
            chart_type: value.r#type.to_string(),
            source: value.source.to_string(),
            traces: value.traces.iter().map(|f| f.to_string()).collect(),
            pie_chart_type: value.pie_chart_type.map(|f| f.to_string()),
            name: value.label.to_string(),
            reduce: value.reduce.map(|s| s.to_string()),
        }
    }
}

impl From<BarChart<'_>> for DashBarChart {
    fn from(value: BarChart<'_>) -> Self {
        DashBarChart {
            chart_type: value.r#type.to_string(),
            source: value.source.to_string(),
            traces: value.traces.iter().map(|f| f.to_string()).collect(),
            name: value.label.to_string(),
            reduce: value.reduce.map(|s| s.to_string()),
        }
    }
}

impl From<TimeSeries<'_>> for DashTimeSeries {
    fn from(value: TimeSeries<'_>) -> Self {
        DashTimeSeries {
            chart_type: value.r#type.to_string(),
            source: value.source.to_string(),
            traces: value.traces.iter().map(|f| f.to_string()).collect(),
            name: value.label.to_string(),
        }
    }
}

impl From<XYChart<'_>> for DashXYChart {
    fn from(value: XYChart<'_>) -> Self {
        DashXYChart {
            chart_type: value.r#type.to_string(),
            source: value.source.to_string(),
            traces: value.traces.iter().map(|f| f.to_string()).collect(),
            name: value.label.to_string(),
        }
    }
}

impl From<Application<'_>> for DashApplication {
    fn from(value: Application<'_>) -> Self {
        DashApplication {
            app_type: value.r#type.to_string(),
            layout: value.layout.to_string(),
            roles: value.roles.iter().map(|f| f.to_string()).collect(),
            visualizations: value
                .panels
                .into_iter()
                .map(|(name, panel_type_union)| {
                    let grafana_panel = match panel_type_union {
                        PanelTypeUnion::PieChart(pie_chart) => {
                            DashPanel::PieChart(pie_chart.into())
                        }
                        PanelTypeUnion::TimeSeries(time_series) => {
                            DashPanel::TimeSeries(time_series.into())
                        }
                        PanelTypeUnion::BarChart(bar_chart) => {
                            DashPanel::BarChart(bar_chart.into())
                        }
                        PanelTypeUnion::GeoMap(geo_map) => DashPanel::GeoMap(geo_map.into()),
                        PanelTypeUnion::XYChart(xy_chart) => DashPanel::XYChart(xy_chart.into()),
                        _ => unreachable!("All unsupported panels should be filtered out by now."),
                    };
                    (name.to_string(), grafana_panel)
                })
                .collect(),
        }
    }
}

impl From<Deployment<'_>> for DashDeployment {
    fn from(value: Deployment<'_>) -> Self {
        DashDeployment {
            environments: value
                .environments
                .into_iter()
                .map(|(name, environment)| (name.to_string(), environment.into()))
                .collect(),
        }
    }
}

impl From<Environment<'_>> for DashEnvironment {
    fn from(value: Environment<'_>) -> Self {
        DashEnvironment {
            uri: value.uri.to_string(),
            port: value.port,
            deploy_type: value.r#type.to_string(),
        }
    }
}
