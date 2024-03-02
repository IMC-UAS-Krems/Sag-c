use crate::sections::{
    Application, BarChart, Config, Datasource, Deployment, Environment, GeoMap, PanelTypeUnion,
    PieChart, Service, TimeSeries, Version, XYChart,
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
    query: String,
    #[serde(rename = "type")]
    datasource_type: String,
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
struct GrafanaPluginMap {
    #[serde(rename="type")]
    chart_type: String,
    //source: String,
    //data: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
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
            query: val.query.to_string(),
            datasource_type: val.r#type.to_string(),
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
                        PanelTypeUnion::GrafanaMap(_) => GrafanaPanel::NotSupported,
                        PanelTypeUnion::GrafanaSingleLine(_) => GrafanaPanel::NotSupported,
                        PanelTypeUnion::GrafanaMultiLine(_) => GrafanaPanel::NotSupported,
                        PanelTypeUnion::GrafanaExtValues(_) => GrafanaPanel::NotSupported,
                        PanelTypeUnion::GrafanaCalendar(_) => GrafanaPanel::NotSupported,
                    };
                    (name.to_string(), grafana_panel)
                })
                .filter(|(_, panel)| match panel {
                    GrafanaPanel::NotSupported => false,
                    _ => true,
                })
                .collect(),
        }
    }
}

impl<'a> From<Deployment<'a>> for GrafanaDeployment {
    fn from(value: Deployment<'a>) -> Self {
        GrafanaDeployment {
            environments: value
                .environments
                .iter()
                .map(|(name, environment)| (name.to_string(), (*environment).into()))
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
