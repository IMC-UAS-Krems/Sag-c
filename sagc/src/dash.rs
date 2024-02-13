use crate::sections::{
    Application, BarChart, Config, Deployment, Environment, GeoMap, PanelTypeUnion, PieChart,
    Service, TimeSeries, Version, XYChart,
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
}

#[derive(Debug, Serialize)]
struct DashQuery {
    r#type: String,
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
}

#[derive(Debug, Serialize)]
struct DashPieChart {
    #[serde(rename = "type")]
    chart_type: String,
    source: String,
    traces: Vec<String>,
    #[serde(default)]
    pie_chart_type: Option<String>,
}

#[derive(Debug, Serialize)]
struct DashBarChart {
    #[serde(rename = "type")]
    chart_type: String,
    source: String,
    traces: Vec<String>,
}

#[derive(Debug, Serialize)]
struct DashTimeSeries {
    #[serde(rename = "type")]
    chart_type: String,
    source: String,
    traces: Vec<String>,
}

#[derive(Debug, Serialize)]
struct DashXYChart {
    #[serde(rename = "type")]
    chart_type: String,
    source: String,
    traces: Vec<String>,
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
    NotSupported,
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
            PanelTypeUnion::GrafanaMap(_) => todo!(),
        })
        .flat_map(|trace| match trace {
            PanelTypeUnion::PieChart(pie_chart) => pie_chart.traces.iter(),
            PanelTypeUnion::TimeSeries(time_series) => time_series.traces.iter(),
            PanelTypeUnion::BarChart(bar_chart) => bar_chart.traces.iter(),
            PanelTypeUnion::GeoMap(geo_map) => geo_map.data.iter(),
            PanelTypeUnion::XYChart(xy_chart) => xy_chart.traces.iter(),
            PanelTypeUnion::GrafanaMap(_) => todo!(),
        })
        .map(|f| f.to_string())
        .collect()
}

impl<'a> From<Config<'a>> for Dash {
    fn from(config: Config<'a>) -> Self {
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
                        r#type: datasource.query.to_string(),
                        select: traces.iter().map(|s| s.to_string()).collect(),
                    },
                    datasource_type: datasource.r#type.to_string(),
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

impl From<GeoMap<'_>> for DashGeoMap {
    fn from(value: GeoMap<'_>) -> Self {
        DashGeoMap {
            chart_type: value.r#type.to_string(),
            source: value.source.to_string(),
            data: value.data.iter().map(|f| f.to_string()).collect(),
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
        }
    }
}

impl From<BarChart<'_>> for DashBarChart {
    fn from(value: BarChart<'_>) -> Self {
        DashBarChart {
            chart_type: value.r#type.to_string(),
            source: value.source.to_string(),
            traces: value.traces.iter().map(|f| f.to_string()).collect(),
        }
    }
}

impl From<TimeSeries<'_>> for DashTimeSeries {
    fn from(value: TimeSeries<'_>) -> Self {
        DashTimeSeries {
            chart_type: value.r#type.to_string(),
            source: value.source.to_string(),
            traces: value.traces.iter().map(|f| f.to_string()).collect(),
        }
    }
}

impl From<XYChart<'_>> for DashXYChart {
    fn from(value: XYChart<'_>) -> Self {
        DashXYChart {
            chart_type: value.r#type.to_string(),
            source: value.source.to_string(),
            traces: value.traces.iter().map(|f| f.to_string()).collect(),
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
                        PanelTypeUnion::GrafanaMap(_) => DashPanel::NotSupported,
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
                .iter()
                .map(|(name, environment)| (name.to_string(), (*environment).into()))
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
