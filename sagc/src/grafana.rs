use crate::sections::{
    Application, BarChart, Config, Datasource, DatasourceConfig, Deployment, Environment, GeoMap,
    GrafanaBnB, GrafanaBulletGraph, GrafanaCalendar, GrafanaExtValues, GrafanaMap,
    GrafanaMultiLine, GrafanaSingleLine, PanelTypeUnion, PieChart, Service, TimeSeries, Version,
    XYChart,
};

use serde::Serialize;
use std::collections::HashMap;
use url::Url;

#[derive(Debug, Serialize)]
pub struct Grafana {
    service: GrafanaService,
    data: GrafanaData,
    application: GrafanaApplication,
    deployment: GrafanaDeployment,
}

#[derive(Debug, Serialize)]
struct GrafanaService {
    title: String,
    scope: String,
    version: GrafanaVersion,
}

#[derive(Debug, Serialize)]
struct GrafanaVersion {
    major: i16,
    minor: i16,
    patch: i16,
}

#[derive(Debug, Serialize)]
struct GrafanaDatasource {
    provider: String,
    uri: Url,
    query: Option<String>,
    #[serde(rename = "type")]
    datasource_type: String,
    config: Option<GrafanaDatasourceConfig>,
}

#[derive(Debug, Serialize)]
struct GrafanaDatasourceConfig {
    company: usize,
    measurements: HashMap<String, usize>,
    token: String,
}

#[derive(Debug, Serialize)]
struct GrafanaData {
    sources: HashMap<String, GrafanaDatasource>,
}

#[derive(Debug, Serialize)]
struct GrafanaGeoMap {
    #[serde(rename = "type")]
    chart_type: String,
    source: String,
    data: Vec<String>,
}

#[derive(Debug, Serialize)]
struct GrafanaPieChart {
    #[serde(rename = "type")]
    chart_type: String,
    source: String,
    traces: Vec<String>,
    #[serde(default)]
    pie_chart_type: Option<String>,
}

#[derive(Debug, Serialize)]
struct GrafanaBarChart {
    #[serde(rename = "type")]
    chart_type: String,
    source: String,
    traces: Vec<String>,
}

#[derive(Debug, Serialize)]
struct GrafanaTimeSeries {
    #[serde(rename = "type")]
    chart_type: String,
    source: String,
    traces: Vec<String>,
}

#[derive(Debug, Serialize)]
struct GrafanaXYChart {
    #[serde(rename = "type")]
    chart_type: String,
    source: String,
    traces: Vec<String>,
}

#[derive(Debug, Serialize)]
struct GrafanaPluginGMap {
    #[serde(rename = "type")]
    chart_type: String,
    source: String,
    traces: Vec<String>,
}

#[derive(Debug, Serialize)]
struct SingleLine {
    #[serde(rename = "type")]
    chart_type: String,
    source: String,
    traces: Vec<String>,
}

#[derive(Debug, Serialize)]
struct MultiLine {
    #[serde(rename = "type")]
    chart_type: String,
    source: String,
    traces: Vec<String>,
    locations: Vec<String>,
}

#[derive(Debug, Serialize)]
struct BnB {
    #[serde(rename = "type")]
    chart_type: String,
    source: String,
    traces: Vec<String>,
    locations: Vec<String>,
}

#[derive(Debug, Serialize)]
struct Calendar {
    #[serde(rename = "type")]
    chart_type: String,
    source: String,
    traces: Vec<String>,
    locations: Vec<String>,
}

#[derive(Debug, Serialize)]
struct BulletGraph {
    #[serde(rename = "type")]
    chart_type: String,
    source: String,
    traces: Vec<String>,
    locations: Vec<String>,
}

#[derive(Debug, Serialize)]
struct ExtValues {
    #[serde(rename = "type")]
    chart_type: String,
    source: String,
    traces: Vec<String>,
    locations: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
enum GrafanaPanel {
    #[serde(rename = "pie_chart")]
    PieChart(GrafanaPieChart),
    #[serde(rename = "timeseries")]
    TimeSeries(GrafanaTimeSeries),
    #[serde(rename = "bar_chart")]
    BarChart(GrafanaBarChart),
    #[serde(rename = "geomap")]
    GeoMap(GrafanaGeoMap),
    #[serde(rename = "xy_chart")]
    XYChart(GrafanaXYChart),
    #[serde(rename = "smartcomm-map-panel")]
    GrafanaMap(GrafanaPluginGMap),
    #[serde(rename = "smartcomm-simpleline-panel")]
    GrafanaSingleLine(SingleLine),
    #[serde(rename = "smartcomm-multiplelinechart-panel")]
    GrafanaMultiLine(MultiLine),
    #[serde(rename = "smartcomm-extremevalues-panel")]
    GrafanaExtValues(ExtValues),
    #[serde(rename = "smartcomm-calendar-panel")]
    GrafanaCalendar(Calendar),
    #[serde(rename = "smartcomm-minmaxbarchart-panel")]
    GrafanaBnB(BnB),
    #[serde(rename = "smartcomm-bulletgraph-panel")]
    BulletGraph(BulletGraph),
    NotSupported,
}

#[derive(Debug, Serialize)]
struct GrafanaApplication {
    #[serde(rename = "type")]
    app_type: String,
    layout: String,
    roles: Vec<String>,
    panels: HashMap<String, GrafanaPanel>,
}

#[derive(Debug, Serialize)]
struct GrafanaDeployment {
    environments: HashMap<String, GrafanaEnvironment>,
}

#[derive(Debug, Serialize)]
struct GrafanaEnvironment {
    uri: String,
    port: i32,
    #[serde(rename = "type")]
    deploy_type: String,
}

impl<'a> From<Service<'a>> for GrafanaService {
    fn from(val: Service<'a>) -> Self {
        GrafanaService {
            title: val.title.to_string(),
            scope: val.scope.to_string(),
            version: val.version.into(),
        }
    }
}

impl From<Version> for GrafanaVersion {
    fn from(val: Version) -> Self {
        GrafanaVersion {
            major: val.major,
            minor: val.minor,
            patch: val.patch,
        }
    }
}

impl<'a> From<Datasource<'a>> for GrafanaDatasource {
    fn from(val: Datasource<'a>) -> Self {
        GrafanaDatasource {
            provider: val.provider.to_string(),
            uri: val.uri,
            query: val.query.map(|q| q.to_string()),
            datasource_type: val.r#type.to_string(),
            config: val.config.map(|c| c.into()),
        }
    }
}

impl<'a> From<DatasourceConfig<'a>> for GrafanaDatasourceConfig {
    fn from(val: DatasourceConfig) -> Self {
        GrafanaDatasourceConfig {
            company: val.company,
            measurements: val
                .measurements
                .into_iter()
                .map(|(k, v)| (v.to_string(), k))
                .collect(),
            token: val.token.to_string(),
        }
    }
}

impl<'a> From<GeoMap<'a>> for GrafanaGeoMap {
    fn from(value: GeoMap<'a>) -> Self {
        GrafanaGeoMap {
            chart_type: value.r#type.to_string(),
            source: value.source.to_string(),
            data: value.data.iter().map(|f| f.to_string()).collect(),
        }
    }
}

impl<'a> From<PieChart<'a>> for GrafanaPieChart {
    fn from(value: PieChart<'a>) -> Self {
        GrafanaPieChart {
            chart_type: value.r#type.to_string(),
            source: value.source.to_string(),
            traces: value.traces.iter().map(|f| f.to_string()).collect(),
            pie_chart_type: value.pie_chart_type.map(|f| f.to_string()),
        }
    }
}

impl<'a> From<BarChart<'a>> for GrafanaBarChart {
    fn from(value: BarChart<'a>) -> Self {
        GrafanaBarChart {
            chart_type: value.r#type.to_string(),
            source: value.source.to_string(),
            traces: value.traces.iter().map(|f| f.to_string()).collect(),
        }
    }
}

impl<'a> From<TimeSeries<'a>> for GrafanaTimeSeries {
    fn from(value: TimeSeries<'a>) -> Self {
        GrafanaTimeSeries {
            chart_type: value.r#type.to_string(),
            source: value.source.to_string(),
            traces: value.traces.iter().map(|f| f.to_string()).collect(),
        }
    }
}

impl<'a> From<XYChart<'a>> for GrafanaXYChart {
    fn from(value: XYChart<'a>) -> Self {
        GrafanaXYChart {
            chart_type: value.r#type.to_string(),
            source: value.source.to_string(),
            traces: value.traces.iter().map(|f| f.to_string()).collect(),
        }
    }
}

impl<'a> From<GrafanaMap<'a>> for GrafanaPluginGMap {
    fn from(value: GrafanaMap<'a>) -> Self {
        GrafanaPluginGMap {
            chart_type: value.r#type.to_string(),
            source: value.source.to_string(),
            traces: value.traces.iter().map(|f| f.to_string()).collect(),
        }
    }
}

impl<'a> From<GrafanaSingleLine<'a>> for SingleLine {
    fn from(value: GrafanaSingleLine<'a>) -> Self {
        SingleLine {
            chart_type: value.r#type.to_string(),
            source: value.source.to_string(),
            traces: value.traces.iter().map(|f| f.to_string()).collect(),
        }
    }
}

impl<'a> From<GrafanaMultiLine<'a>> for MultiLine {
    fn from(value: GrafanaMultiLine<'a>) -> Self {
        MultiLine {
            chart_type: value.r#type.to_string(),
            source: value.source.to_string(),
            traces: value.traces.iter().map(|f| f.to_string()).collect(),
            locations: value.locations.iter().map(|f| f.to_string()).collect(),
        }
    }
}

impl<'a> From<GrafanaBnB<'a>> for BnB {
    fn from(value: GrafanaBnB<'a>) -> Self {
        BnB {
            chart_type: value.r#type.to_string(),
            source: value.source.to_string(),
            traces: value.traces.iter().map(|f| f.to_string()).collect(),
            locations: value.locations.iter().map(|f| f.to_string()).collect(),
        }
    }
}

impl<'a> From<GrafanaExtValues<'a>> for ExtValues {
    fn from(value: GrafanaExtValues<'a>) -> Self {
        ExtValues {
            chart_type: value.r#type.to_string(),
            source: value.source.to_string(),
            traces: value.traces.iter().map(|f| f.to_string()).collect(),
            locations: value.locations.iter().map(|f| f.to_string()).collect(),
        }
    }
}

impl<'a> From<GrafanaCalendar<'a>> for Calendar {
    fn from(value: GrafanaCalendar<'a>) -> Self {
        Calendar {
            chart_type: value.r#type.to_string(),
            source: value.source.to_string(),
            traces: value.traces.iter().map(|f| f.to_string()).collect(),
            locations: value.locations.iter().map(|f| f.to_string()).collect(),
        }
    }
}

impl<'a> From<GrafanaBulletGraph<'a>> for BulletGraph {
    fn from(value: GrafanaBulletGraph<'a>) -> Self {
        BulletGraph {
            chart_type: value.r#type.to_string(),
            source: value.source.to_string(),
            traces: value.traces.iter().map(|f| f.to_string()).collect(),
            locations: value.locations.iter().map(|f| f.to_string()).collect(),
        }
    }
}

impl<'a> From<Application<'a>> for GrafanaApplication {
    fn from(value: Application<'a>) -> Self {
        GrafanaApplication {
            app_type: value.r#type.to_string(),
            layout: value.layout.to_string(),
            roles: value.roles.iter().map(|f| f.to_string()).collect(),
            panels: value
                .panels
                .into_iter()
                .map(|(name, panel_type_union)| {
                    let grafana_panel = match panel_type_union {
                        PanelTypeUnion::PieChart(pie_chart) => {
                            GrafanaPanel::PieChart(pie_chart.into())
                        }
                        PanelTypeUnion::TimeSeries(time_series) => {
                            GrafanaPanel::TimeSeries(time_series.into())
                        }
                        PanelTypeUnion::BarChart(bar_chart) => {
                            GrafanaPanel::BarChart(bar_chart.into())
                        }
                        PanelTypeUnion::GeoMap(geo_map) => GrafanaPanel::GeoMap(geo_map.into()),
                        PanelTypeUnion::XYChart(xy_chart) => GrafanaPanel::XYChart(xy_chart.into()),
                        PanelTypeUnion::GrafanaMap(geomap) => {
                            GrafanaPanel::GrafanaMap(geomap.into())
                        }
                        PanelTypeUnion::GrafanaSingleLine(g_sline) => {
                            GrafanaPanel::GrafanaSingleLine(g_sline.into())
                        }
                        PanelTypeUnion::GrafanaMultiLine(g_mline) => {
                            GrafanaPanel::GrafanaMultiLine(g_mline.into())
                        }
                        PanelTypeUnion::GrafanaExtValues(g_ext) => {
                            GrafanaPanel::GrafanaExtValues(g_ext.into())
                        }
                        PanelTypeUnion::GrafanaCalendar(g_calendar) => {
                            GrafanaPanel::GrafanaCalendar(g_calendar.into())
                        }
                        PanelTypeUnion::GrafanaBnB(g_bnb) => GrafanaPanel::GrafanaBnB(g_bnb.into()),
                        PanelTypeUnion::GrafanaBulletGraph(g_bullet) => {
                            GrafanaPanel::BulletGraph(g_bullet.into())
                        }
                    };
                    (name.to_string(), grafana_panel)
                })
                .filter(|(_, panel)| !matches!(panel, GrafanaPanel::NotSupported))
                .collect(),
        }
    }
}

impl<'a> From<Deployment<'a>> for GrafanaDeployment {
    fn from(value: Deployment<'a>) -> Self {
        GrafanaDeployment {
            environments: value
                .environments
                .into_iter()
                .map(|(name, environment)| (name.to_string(), environment.into()))
                .collect(),
        }
    }
}

impl<'a> From<Environment<'a>> for GrafanaEnvironment {
    fn from(value: Environment<'a>) -> Self {
        GrafanaEnvironment {
            uri: value.uri.to_string(),
            port: value.port,
            deploy_type: value.r#type.to_string(),
        }
    }
}

impl<'a> From<Config<'a>> for Grafana {
    fn from(config: Config<'a>) -> Self {
        // Map Config fields to Grafana fields
        let service = config.service.into();
        let data_sources = config
            .data
            .sources
            .into_iter()
            .map(|(name, datasource)| {
                let grafana_datasource = datasource.into();
                (name.to_string(), grafana_datasource)
            })
            .collect();

        let application = config.application.into();
        let deployment = config.deployment.into();

        Grafana {
            service,
            data: GrafanaData {
                sources: data_sources,
            },
            application,
            deployment,
        }
    }
}
