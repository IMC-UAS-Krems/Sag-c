#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use sagc::sections::{GeoMap,
        XYChart,
        GrafanaSingleLine,
        BarChart,
        PieChart,
        PieChartType,
        PanelType, 
        GrafanaMap,
        TimeSeries,
        GrafanaMultiLine,
        GrafanaExtValues,
        GrafanaBnB,
        GrafanaBulletGraph,
        GrafanaCalendar};
    use sagc::parser::{Blocks, Value, Position, ParseResult, Span, Token, TokenValue};
    use sagc::errors::{SagError};
    const PH_POS: Position = Position{ row_start: 1, row_end: 1, col_start: 1, col_end: 5 }; //general placeholder position

    //=====GeoMap=====
    #[test]
    fn test_geomap_new_correct() {
        let mut blocks = Blocks::new();

        //makes a custom block
        let block_name = "Map";
        blocks.insert(
            block_name,
            ParseResult::new(
                PH_POS,
                Value::Block({
                    let mut inner = HashMap::new();
                    inner.insert("label", ParseResult::new(PH_POS, Value::String("Map Label")));
                    inner.insert("type", ParseResult::new(PH_POS, Value::String("geomap")));
                    inner.insert("source", ParseResult::new(PH_POS, Value::String("source_data")));
                    inner.insert(
                        "data",
                        ParseResult::new(PH_POS, Value::Vec(vec!["location", "stationName", "O3"])),
                    );
                    inner.insert("area", ParseResult::new(PH_POS, Value::String("Area1")));
                    sagc::parser::Blocks { blocks: HashMap::<&str, ParseResult<'_>>::from(inner) }
                }),
            ),
        );
        let geomap = GeoMap::new(&mut blocks, block_name);

        assert!(geomap.is_ok());
        let geomap = geomap.unwrap();

        assert_eq!(geomap.label, "Map Label");
        assert_eq!(geomap.source, "source_data");
        assert_eq!(geomap.data, vec!["location", "stationName", "O3"]);
        assert_eq!(geomap.area, Some("Area1"));
    }

    #[test]
    fn test_geomap_no_label() {
        let mut blocks = Blocks::new();

        // Adding a block without a label
        let block_name = "Map";
        blocks.insert(
            block_name,
            ParseResult::new(
                PH_POS,
                Value::Block({
                    let mut inner = HashMap::new();
                    inner.insert("type", ParseResult::new(PH_POS, Value::String("geomap")));
                    inner.insert("source", ParseResult::new(PH_POS, Value::String("source_data")));
                    inner.insert(
                        "data",
                        ParseResult::new(PH_POS, Value::Vec(vec!["location", "stationName", "O3"])),
                    );
                    inner.insert("area", ParseResult::new(PH_POS, Value::String("Area1")));
                    sagc::parser::Blocks { blocks: HashMap::<&str, ParseResult<'_>>::from(inner) }
                }),
            ),
        );

        let geomap = GeoMap::new(&mut blocks, block_name);

        assert!(geomap.is_err());
        let errors = geomap.unwrap_err();
        assert!(errors.iter().any(|e| matches!(e, SagError::LanguageError(_))));
    }

    #[test]
    fn test_geomap_new_wrong_type() {
        let mut blocks = Blocks::new();

        let block_name = "Map";
        blocks.insert(
            block_name,
            ParseResult::new(
                PH_POS,
                Value::Block({
                    let mut inner = HashMap::new();
                    inner.insert("label", ParseResult::new(PH_POS, Value::String("Map Label")));
                    inner.insert("type", ParseResult::new(PH_POS, Value::String("invalid_type")));
                    inner.insert("source", ParseResult::new(PH_POS, Value::String("source_data")));
                    inner.insert(
                        "data",
                        ParseResult::new(PH_POS, Value::Vec(vec!["location", "stationName", "O3"])),
                    );
                    inner.insert("area", ParseResult::new(PH_POS, Value::String("Area1")));
                    sagc::parser::Blocks { blocks: HashMap::<&str, ParseResult<'_>>::from(inner) }
                }),
            ),
        );

        let geomap = GeoMap::new(&mut blocks, block_name);

        assert!(geomap.is_err());
        let errors = geomap.unwrap_err();
        assert!(errors.iter().any(|e| matches!(e, SagError::LanguageError(_))));
    }

    #[test]
    fn test_geomap_no_area() {
        let mut blocks = Blocks::new();

        let block_name = "Map";
        blocks.insert(
            block_name,
            ParseResult::new(
                PH_POS,
                Value::Block({
                    let mut inner = HashMap::new();
                    inner.insert("label", ParseResult::new(PH_POS, Value::String("Map Label")));
                    inner.insert("type", ParseResult::new(PH_POS, Value::String("geomap")));
                    inner.insert("source", ParseResult::new(PH_POS, Value::String("source_data")));
                    inner.insert(
                        "data",
                        ParseResult::new(PH_POS, Value::Vec(vec!["location", "stationName", "O3"])),
                    );
                    sagc::parser::Blocks { blocks: HashMap::<&str, ParseResult<'_>>::from(inner) }
                }),
            ),
        );

        let geomap = GeoMap::new(&mut blocks, block_name);

        assert!(geomap.is_ok());
        let geomap = geomap.unwrap();

        assert_eq!(geomap.label, "Map Label");
        assert_eq!(geomap.source, "source_data");
        assert_eq!(geomap.data, vec!["location", "stationName", "O3"]);
        assert_eq!(geomap.area, None);
    }
    //=====TimeSeries=====
    #[test]
    fn test_timeseries_new_correct() {
        let mut blocks = Blocks::new();

        let block_name = "TimeSeries";
        blocks.insert(
            block_name,
            ParseResult::new(
                PH_POS,
                Value::Block({
                    let mut inner = HashMap::new();
                    inner.insert("label", ParseResult::new(PH_POS, Value::String("Time Series Label")));
                    inner.insert("type", ParseResult::new(PH_POS, Value::String("timeseries")));
                    inner.insert("source", ParseResult::new(PH_POS, Value::String("source_data")));
                    inner.insert(
                        "traces",
                        ParseResult::new(PH_POS, Value::Vec(vec!["trace1", "trace2", "trace3"])),
                    );
                    sagc::parser::Blocks { blocks: HashMap::<&str, ParseResult<'_>>::from(inner) }
                }),
            ),
        );

        let timeseries = TimeSeries::new(&mut blocks, block_name);

        assert!(timeseries.is_ok());
        let timeseries = timeseries.unwrap();

        assert_eq!(timeseries.label, "Time Series Label");
        assert_eq!(timeseries.source, "source_data");
        assert_eq!(timeseries.traces, vec!["trace1", "trace2", "trace3"]);
    }

    #[test]
    fn test_timeseries_no_label() {
        let mut blocks = Blocks::new();

        let block_name = "TimeSeries";
        blocks.insert(
            block_name,
            ParseResult::new(
                PH_POS,
                Value::Block({
                    let mut inner = HashMap::new();
                    inner.insert("type", ParseResult::new(PH_POS, Value::String("timeseries")));
                    inner.insert("source", ParseResult::new(PH_POS, Value::String("source_data")));
                    inner.insert(
                        "traces",
                        ParseResult::new(PH_POS, Value::Vec(vec!["trace1", "trace2", "trace3"])),
                    );
                    sagc::parser::Blocks { blocks: HashMap::<&str, ParseResult<'_>>::from(inner) }
                }),
            ),
        );

        let timeseries = TimeSeries::new(&mut blocks, block_name);

        assert!(timeseries.is_err());
        let errors = timeseries.unwrap_err();
        assert!(errors.iter().any(|e| matches!(e, SagError::LanguageError(_))));
    }

    #[test]
    fn test_timeseries_wrong_type() {
        let mut blocks = Blocks::new();

        let block_name = "TimeSeries";
        blocks.insert(
            block_name,
            ParseResult::new(
                PH_POS,
                Value::Block({
                    let mut inner = HashMap::new();
                    inner.insert("label", ParseResult::new(PH_POS, Value::String("Time Series Label")));
                    inner.insert("type", ParseResult::new(PH_POS, Value::String("invalid_type")));
                    inner.insert("source", ParseResult::new(PH_POS, Value::String("source_data")));
                    inner.insert(
                        "traces",
                        ParseResult::new(PH_POS, Value::Vec(vec!["trace1", "trace2", "trace3"])),
                    );
                    sagc::parser::Blocks { blocks: HashMap::<&str, ParseResult<'_>>::from(inner) }
                }),
            ),
        );

        let timeseries = TimeSeries::new(&mut blocks, block_name);

        assert!(timeseries.is_err());
        let errors = timeseries.unwrap_err();
        assert!(errors.iter().any(|e| matches!(e, SagError::LanguageError(_))));
    }

    #[test]
    fn test_timeseries_no_traces() {
        let mut blocks = Blocks::new();

        let block_name = "TimeSeries";
        blocks.insert(
            block_name,
            ParseResult::new(
                PH_POS,
                Value::Block({
                    let mut inner = HashMap::new();
                    inner.insert("label", ParseResult::new(PH_POS, Value::String("Time Series Label")));
                    inner.insert("type", ParseResult::new(PH_POS, Value::String("timeseries")));
                    inner.insert("source", ParseResult::new(PH_POS, Value::String("source_data")));
                    sagc::parser::Blocks { blocks: HashMap::<&str, ParseResult<'_>>::from(inner) }
                }),
            ),
        );

        let timeseries = TimeSeries::new(&mut blocks, block_name);

        assert!(timeseries.is_err());
        let errors = timeseries.unwrap_err();
        assert!(errors.iter().any(|e| matches!(e, SagError::LanguageError(_))));
    }

    //=====GrafanaMap=====
    #[test]
    fn test_grafanamap_new_correct() {
        let mut blocks = Blocks::new();

        let block_name = "GrafanaMap";
        blocks.insert(
            block_name,
            ParseResult::new(
                PH_POS,
                Value::Block({
                    let mut inner = HashMap::new();
                    inner.insert("type", ParseResult::new(PH_POS, Value::String("smartcomm-map-panel")));
                    inner.insert("source", ParseResult::new(PH_POS, Value::String("source_data")));
                    inner.insert(
                        "traces",
                        ParseResult::new(PH_POS, Value::Vec(vec!["trace1", "trace2", "trace3"])),
                    );
                    sagc::parser::Blocks { blocks: HashMap::<&str, ParseResult<'_>>::from(inner) }
                }),
            ),
        );

        let grafanamap = GrafanaMap::new(&mut blocks, block_name);

        assert!(grafanamap.is_ok());
        let grafanamap = grafanamap.unwrap();

        assert_eq!(grafanamap.source, "source_data");
        assert_eq!(grafanamap.traces, vec!["trace1", "trace2", "trace3"]);
    }

    #[test]
    fn test_grafanamap_missing_type() {
        let mut blocks = Blocks::new();

        let block_name = "GrafanaMap";
        blocks.insert(
            block_name,
            ParseResult::new(
                PH_POS,
                Value::Block({
                    let mut inner = HashMap::new();
                    inner.insert("source", ParseResult::new(PH_POS, Value::String("source_data")));
                    inner.insert(
                        "traces",
                        ParseResult::new(PH_POS, Value::Vec(vec!["trace1", "trace2", "trace3"])),
                    );
                    sagc::parser::Blocks { blocks: HashMap::<&str, ParseResult<'_>>::from(inner) }
                }),
            ),
        );

        let grafanamap = GrafanaMap::new(&mut blocks, block_name);

        assert!(grafanamap.is_err());
        let errors = grafanamap.unwrap_err();
        assert!(errors.iter().any(|e| matches!(e, SagError::LanguageError(_))));
    }

    #[test]
    fn test_grafanamap_invalid_type() {
        let mut blocks = Blocks::new();

        let block_name = "GrafanaMap";
        blocks.insert(
            block_name,
            ParseResult::new(
                PH_POS,
                Value::Block({
                    let mut inner = HashMap::new();
                    inner.insert("type", ParseResult::new(PH_POS, Value::String("invalid_type")));
                    inner.insert("source", ParseResult::new(PH_POS, Value::String("source_data")));
                    inner.insert(
                        "traces",
                        ParseResult::new(PH_POS, Value::Vec(vec!["trace1", "trace2", "trace3"])),
                    );
                    sagc::parser::Blocks { blocks: HashMap::<&str, ParseResult<'_>>::from(inner) }
                }),
            ),
        );

        let grafanamap = GrafanaMap::new(&mut blocks, block_name);

        assert!(grafanamap.is_err());
        let errors = grafanamap.unwrap_err();
        assert!(errors.iter().any(|e| matches!(e, SagError::LanguageError(_))));
    }

    #[test]
    fn test_grafanamap_missing_source() {
        let mut blocks = Blocks::new();

        let block_name = "GrafanaMap";
        blocks.insert(
            block_name,
            ParseResult::new(
                PH_POS,
                Value::Block({
                    let mut inner = HashMap::new();
                    inner.insert("type", ParseResult::new(PH_POS, Value::String("smartcomm-map-panel")));
                    inner.insert(
                        "traces",
                        ParseResult::new(PH_POS, Value::Vec(vec!["trace1", "trace2", "trace3"])),
                    );
                    sagc::parser::Blocks { blocks: HashMap::<&str, ParseResult<'_>>::from(inner) }
                }),
            ),
        );

        let grafanamap = GrafanaMap::new(&mut blocks, block_name);

        assert!(grafanamap.is_err());
        let errors = grafanamap.unwrap_err();
        assert!(errors.iter().any(|e| matches!(e, SagError::LanguageError(_))));
    }

    #[test]
    fn test_grafanamap_missing_traces() {
        let mut blocks = Blocks::new();

        let block_name = "GrafanaMap";
        blocks.insert(
            block_name,
            ParseResult::new(
                PH_POS,
                Value::Block({
                    let mut inner = HashMap::new();
                    inner.insert("type", ParseResult::new(PH_POS, Value::String("smartcomm-map-panel")));
                    inner.insert("source", ParseResult::new(PH_POS, Value::String("source_data")));
                    sagc::parser::Blocks { blocks: HashMap::<&str, ParseResult<'_>>::from(inner) }
                }),
            ),
        );

        let grafanamap = GrafanaMap::new(&mut blocks, block_name);

        assert!(grafanamap.is_err());
        let errors = grafanamap.unwrap_err();
        assert!(errors.iter().any(|e| matches!(e, SagError::LanguageError(_))));
    }
    //=====PieChart=====
    #[test]
    fn test_piechart_new_correct() {
        let mut blocks = Blocks::new();

        let block_name = "PieChart";
        blocks.insert(
            block_name,
            ParseResult::new(
                PH_POS,
                Value::Block({
                    let mut inner = HashMap::new();
                    inner.insert("label", ParseResult::new(PH_POS, Value::String("Pie Chart Label")));
                    inner.insert("type", ParseResult::new(PH_POS, Value::String("pie_chart")));
                    inner.insert("source", ParseResult::new(PH_POS, Value::String("source_data")));
                    inner.insert(
                        "traces",
                        ParseResult::new(PH_POS, Value::Vec(vec!["trace1", "trace2", "trace3"])),
                    );
                    inner.insert(
                        "pie_chart_type",
                        ParseResult::new(PH_POS, Value::String("donut")),
                    );
                    sagc::parser::Blocks { blocks: HashMap::<&str, ParseResult<'_>>::from(inner) }
                }),
            ),
        );

        let piechart = PieChart::new(&mut blocks, block_name);

        assert!(piechart.is_ok());
        let piechart = piechart.unwrap();

        assert_eq!(piechart.label, "Pie Chart Label");
        assert_eq!(piechart.source, "source_data");
        assert_eq!(piechart.traces, vec!["trace1", "trace2", "trace3"]);
    }

    #[test]
    fn test_piechart_missing_pie_chart_type() {
        let mut blocks = Blocks::new();

        let block_name = "PieChart";
        blocks.insert(
            block_name,
            ParseResult::new(
                PH_POS,
                Value::Block({
                    let mut inner = HashMap::new();
                    inner.insert("label", ParseResult::new(PH_POS, Value::String("Pie Chart Label")));
                    inner.insert("type", ParseResult::new(PH_POS, Value::String("pie_chart")));
                    inner.insert("source", ParseResult::new(PH_POS, Value::String("source_data")));
                    inner.insert(
                        "traces",
                        ParseResult::new(PH_POS, Value::Vec(vec!["trace1", "trace2", "trace3"])),
                    );
                    sagc::parser::Blocks { blocks: HashMap::<&str, ParseResult<'_>>::from(inner) }
                }),
            ),
        );

        let piechart = PieChart::new(&mut blocks, block_name);

        assert!(piechart.is_ok());
        let piechart = piechart.unwrap();

        assert_eq!(piechart.label, "Pie Chart Label");
        assert_eq!(piechart.source, "source_data");
        assert_eq!(piechart.traces, vec!["trace1", "trace2", "trace3"]);
    }

    #[test]
    fn test_piechart_invalid_pie_chart_type() {
        let mut blocks = Blocks::new();

        let block_name = "PieChart";
        blocks.insert(
            block_name,
            ParseResult::new(
                PH_POS,
                Value::Block({
                    let mut inner = HashMap::new();
                    inner.insert("label", ParseResult::new(PH_POS, Value::String("Pie Chart Label")));
                    inner.insert("type", ParseResult::new(PH_POS, Value::String("cake_chart")));
                    inner.insert("source", ParseResult::new(PH_POS, Value::String("source_data")));
                    inner.insert(
                        "traces",
                        ParseResult::new(PH_POS, Value::Vec(vec!["trace1", "trace2", "trace3"])),
                    );
                    inner.insert(
                        "pie_chart_type",
                        ParseResult::new(PH_POS, Value::String("invalid_type")),
                    );
                    sagc::parser::Blocks { blocks: HashMap::<&str, ParseResult<'_>>::from(inner) }
                }),
            ),
        );

        let piechart = PieChart::new(&mut blocks, block_name);

        assert!(piechart.is_err());
        let errors = piechart.unwrap_err();
        assert!(errors.iter().any(|e| matches!(e, SagError::LanguageError(_))));
    }

    #[test]
    fn test_piechart_missing_label() {
        let mut blocks = Blocks::new();

        let block_name = "PieChart";
        blocks.insert(
            block_name,
            ParseResult::new(
                PH_POS,
                Value::Block({
                    let mut inner = HashMap::new();
                    inner.insert("type", ParseResult::new(PH_POS, Value::String("pie_chart")));
                    inner.insert("source", ParseResult::new(PH_POS, Value::String("source_data")));
                    inner.insert(
                        "traces",
                        ParseResult::new(PH_POS, Value::Vec(vec!["trace1", "trace2", "trace3"])),
                    );
                    inner.insert(
                        "pie_chart_type",
                        ParseResult::new(PH_POS, Value::String("pie")),
                    );
                    sagc::parser::Blocks { blocks: HashMap::<&str, ParseResult<'_>>::from(inner) }
                }),
            ),
        );

        let piechart = PieChart::new(&mut blocks, block_name);

        assert!(piechart.is_err());
        let errors = piechart.unwrap_err();
        assert!(errors.iter().any(|e| matches!(e, SagError::LanguageError(_))));
    }

    #[test]
    fn test_piechart_missing_traces() {
        let mut blocks = Blocks::new();

        let block_name = "PieChart";
        blocks.insert(
            block_name,
            ParseResult::new(
                PH_POS,
                Value::Block({
                    let mut inner = HashMap::new();
                    inner.insert("label", ParseResult::new(PH_POS, Value::String("Pie Chart Label")));
                    inner.insert("type", ParseResult::new(PH_POS, Value::String("pie_chart")));
                    inner.insert("source", ParseResult::new(PH_POS, Value::String("source_data")));
                    inner.insert(
                        "pie_chart_type",
                        ParseResult::new(PH_POS, Value::String("pie")),
                    );
                    sagc::parser::Blocks { blocks: HashMap::<&str, ParseResult<'_>>::from(inner) }
                }),
            ),
        );

        let piechart = PieChart::new(&mut blocks, block_name);

        assert!(piechart.is_err());
        let errors = piechart.unwrap_err();
        assert!(errors.iter().any(|e| matches!(e, SagError::LanguageError(_))));
    }
    //=====BarChart=====
    #[test]
    fn test_barchart_new_correct() {
        let mut blocks = Blocks::new();

        let block_name = "BarChart";
        blocks.insert(
            block_name,
            ParseResult::new(
                PH_POS,
                Value::Block({
                    let mut inner = HashMap::new();
                    inner.insert("label", ParseResult::new(PH_POS, Value::String("Bar Chart Label")));
                    inner.insert("type", ParseResult::new(PH_POS, Value::String("bar_chart")));
                    inner.insert("source", ParseResult::new(PH_POS, Value::String("source_data")));
                    inner.insert(
                        "traces",
                        ParseResult::new(PH_POS, Value::Vec(vec!["trace1", "trace2", "trace3"])),
                    );
                    sagc::parser::Blocks { blocks: HashMap::<&str, ParseResult<'_>>::from(inner) }
                }),
            ),
        );

        let barchart = BarChart::new(&mut blocks, block_name);

        assert!(barchart.is_ok());
        let barchart = barchart.unwrap();

        assert_eq!(barchart.label, "Bar Chart Label");
        assert_eq!(barchart.source, "source_data");
        assert_eq!(barchart.traces, vec!["trace1", "trace2", "trace3"]);
    }

    #[test]
    fn test_barchart_missing_label() {
        let mut blocks = Blocks::new();

        let block_name = "BarChart";
        blocks.insert(
            block_name,
            ParseResult::new(
                PH_POS,
                Value::Block({
                    let mut inner = HashMap::new();
                    inner.insert("type", ParseResult::new(PH_POS, Value::String("bar_chart")));
                    inner.insert("source", ParseResult::new(PH_POS, Value::String("source_data")));
                    inner.insert(
                        "traces",
                        ParseResult::new(PH_POS, Value::Vec(vec!["trace1", "trace2", "trace3"])),
                    );
                    sagc::parser::Blocks { blocks: HashMap::<&str, ParseResult<'_>>::from(inner) }
                }),
            ),
        );

        let barchart = BarChart::new(&mut blocks, block_name);

        assert!(barchart.is_err());
        let errors = barchart.unwrap_err();
        assert!(errors.iter().any(|e| matches!(e, SagError::LanguageError(_))));
    }

    #[test]
    fn test_barchart_invalid_type() {
        let mut blocks = Blocks::new();

        let block_name = "BarChart";
        blocks.insert(
            block_name,
            ParseResult::new(
                PH_POS,
                Value::Block({
                    let mut inner = HashMap::new();
                    inner.insert("label", ParseResult::new(PH_POS, Value::String("Bar Chart Label")));
                    inner.insert("type", ParseResult::new(PH_POS, Value::String("invalid_type")));
                    inner.insert("source", ParseResult::new(PH_POS, Value::String("source_data")));
                    inner.insert(
                        "traces",
                        ParseResult::new(PH_POS, Value::Vec(vec!["trace1", "trace2", "trace3"])),
                    );
                    sagc::parser::Blocks { blocks: HashMap::<&str, ParseResult<'_>>::from(inner) }
                }),
            ),
        );

        let barchart = BarChart::new(&mut blocks, block_name);

        assert!(barchart.is_err());
        let errors = barchart.unwrap_err();
        assert!(errors.iter().any(|e| matches!(e, SagError::LanguageError(_))));
    }

    #[test]
    fn test_barchart_missing_source() {
        let mut blocks = Blocks::new();

        let block_name = "BarChart";
        blocks.insert(
            block_name,
            ParseResult::new(
                PH_POS,
                Value::Block({
                    let mut inner = HashMap::new();
                    inner.insert("label", ParseResult::new(PH_POS, Value::String("Bar Chart Label")));
                    inner.insert("type", ParseResult::new(PH_POS, Value::String("bar_chart")));
                    inner.insert(
                        "traces",
                        ParseResult::new(PH_POS, Value::Vec(vec!["trace1", "trace2", "trace3"])),
                    );
                    sagc::parser::Blocks { blocks: HashMap::<&str, ParseResult<'_>>::from(inner) }
                }),
            ),
        );

        let barchart = BarChart::new(&mut blocks, block_name);

        assert!(barchart.is_err());
        let errors = barchart.unwrap_err();
        assert!(errors.iter().any(|e| matches!(e, SagError::LanguageError(_))));
    }

    #[test]
    fn test_barchart_missing_traces() {
        let mut blocks = Blocks::new();

        let block_name = "BarChart";
        blocks.insert(
            block_name,
            ParseResult::new(
                PH_POS,
                Value::Block({
                    let mut inner = HashMap::new();
                    inner.insert("label", ParseResult::new(PH_POS, Value::String("Bar Chart Label")));
                    inner.insert("type", ParseResult::new(PH_POS, Value::String("bar_chart")));
                    inner.insert("source", ParseResult::new(PH_POS, Value::String("source_data")));
                    sagc::parser::Blocks { blocks: HashMap::<&str, ParseResult<'_>>::from(inner) }
                }),
            ),
        );

        let barchart = BarChart::new(&mut blocks, block_name);

        assert!(barchart.is_err());
        let errors = barchart.unwrap_err();
        assert!(errors.iter().any(|e| matches!(e, SagError::LanguageError(_))));
    }
    //=====XYChart=====
    #[test]
    fn test_xychart_new_correct() {
        let mut blocks = Blocks::new();

        let block_name = "XYChart";
        blocks.insert(
            block_name,
            ParseResult::new(
                PH_POS,
                Value::Block({
                    let mut inner = HashMap::new();
                    inner.insert("label", ParseResult::new(PH_POS, Value::String("XY Chart Label")));
                    inner.insert("type", ParseResult::new(PH_POS, Value::String("xy_chart")));
                    inner.insert("source", ParseResult::new(PH_POS, Value::String("source_data")));
                    inner.insert(
                        "traces",
                        ParseResult::new(PH_POS, Value::Vec(vec!["trace1", "trace2", "trace3"])),
                    );
                    sagc::parser::Blocks { blocks: HashMap::<&str, ParseResult<'_>>::from(inner) }
                }),
            ),
        );

        let xychart = XYChart::new(&mut blocks, block_name);

        assert!(xychart.is_ok());
        let xychart = xychart.unwrap();

        assert_eq!(xychart.label, "XY Chart Label");
        assert_eq!(xychart.source, "source_data");
        assert_eq!(xychart.traces, vec!["trace1", "trace2", "trace3"]);
    }

    #[test]
    fn test_xychart_missing_label() {
        let mut blocks = Blocks::new();

        let block_name = "XYChart";
        blocks.insert(
            block_name,
            ParseResult::new(
                PH_POS,
                Value::Block({
                    let mut inner = HashMap::new();
                    inner.insert("type", ParseResult::new(PH_POS, Value::String("xy_chart")));
                    inner.insert("source", ParseResult::new(PH_POS, Value::String("source_data")));
                    inner.insert(
                        "traces",
                        ParseResult::new(PH_POS, Value::Vec(vec!["trace1", "trace2", "trace3"])),
                    );
                    sagc::parser::Blocks { blocks: HashMap::<&str, ParseResult<'_>>::from(inner) }
                }),
            ),
        );

        let xychart = XYChart::new(&mut blocks, block_name);

        assert!(xychart.is_err());
        let errors = xychart.unwrap_err();
        assert!(errors.iter().any(|e| matches!(e, SagError::LanguageError(_))));
    }

    #[test]
    fn test_xychart_invalid_type() {
        let mut blocks = Blocks::new();

        let block_name = "XYChart";
        blocks.insert(
            block_name,
            ParseResult::new(
                PH_POS,
                Value::Block({
                    let mut inner = HashMap::new();
                    inner.insert("label", ParseResult::new(PH_POS, Value::String("XY Chart Label")));
                    inner.insert("type", ParseResult::new(PH_POS, Value::String("invalid_type")));
                    inner.insert("source", ParseResult::new(PH_POS, Value::String("source_data")));
                    inner.insert(
                        "traces",
                        ParseResult::new(PH_POS, Value::Vec(vec!["trace1", "trace2", "trace3"])),
                    );
                    sagc::parser::Blocks { blocks: HashMap::<&str, ParseResult<'_>>::from(inner) }
                }),
            ),
        );

        let xychart = XYChart::new(&mut blocks, block_name);

        assert!(xychart.is_err());
        let errors = xychart.unwrap_err();
        assert!(errors.iter().any(|e| matches!(e, SagError::LanguageError(_))));
    }

    #[test]
    fn test_xychart_missing_source() {
        let mut blocks = Blocks::new();

        let block_name = "XYChart";
        blocks.insert(
            block_name,
            ParseResult::new(
                PH_POS,
                Value::Block({
                    let mut inner = HashMap::new();
                    inner.insert("label", ParseResult::new(PH_POS, Value::String("XY Chart Label")));
                    inner.insert("type", ParseResult::new(PH_POS, Value::String("xy_chart")));
                    inner.insert(
                        "traces",
                        ParseResult::new(PH_POS, Value::Vec(vec!["trace1", "trace2", "trace3"])),
                    );
                    sagc::parser::Blocks { blocks: HashMap::<&str, ParseResult<'_>>::from(inner) }
                }),
            ),
        );

        let xychart = XYChart::new(&mut blocks, block_name);

        assert!(xychart.is_err());
        let errors = xychart.unwrap_err();
        assert!(errors.iter().any(|e| matches!(e, SagError::LanguageError(_))));
    }

    #[test]
    fn test_xychart_missing_traces() {
        let mut blocks = Blocks::new();

        let block_name = "XYChart";
        blocks.insert(
            block_name,
            ParseResult::new(
                PH_POS,
                Value::Block({
                    let mut inner = HashMap::new();
                    inner.insert("label", ParseResult::new(PH_POS, Value::String("XY Chart Label")));
                    inner.insert("type", ParseResult::new(PH_POS, Value::String("xy_chart")));
                    inner.insert("source", ParseResult::new(PH_POS, Value::String("source_data")));
                    sagc::parser::Blocks { blocks: HashMap::<&str, ParseResult<'_>>::from(inner) }
                }),
            ),
        );

        let xychart = XYChart::new(&mut blocks, block_name);

        assert!(xychart.is_err());
        let errors = xychart.unwrap_err();
        assert!(errors.iter().any(|e| matches!(e, SagError::LanguageError(_))));
    }
    //=====GrafanaSingleLine=====
    #[test]
    fn test_grafanasingleline_new_correct() {
        let mut blocks = Blocks::new();

        let block_name = "GrafanaSingleLine";
        blocks.insert(
            block_name,
            ParseResult::new(
                PH_POS,
                Value::Block({
                    let mut inner = HashMap::new();
                    inner.insert("type", ParseResult::new(PH_POS, Value::String("smartcomm-simpleline-panel")));
                    inner.insert("source", ParseResult::new(PH_POS, Value::String("source_data")));
                    inner.insert(
                        "traces",
                        ParseResult::new(PH_POS, Value::Vec(vec!["trace1", "trace2", "trace3"])),
                    );
                    sagc::parser::Blocks { blocks: HashMap::<&str, ParseResult<'_>>::from(inner) }
                }),
            ),
        );

        let grafanasingleline = GrafanaSingleLine::new(&mut blocks, block_name);

        assert!(grafanasingleline.is_ok());
        let grafanasingleline = grafanasingleline.unwrap();

        assert_eq!(grafanasingleline.source, "source_data");
        assert_eq!(grafanasingleline.traces, vec!["trace1", "trace2", "trace3"]);
    }

    #[test]
    fn test_grafanasingleline_missing_type() {
        let mut blocks = Blocks::new();

        let block_name = "GrafanaSingleLine";
        blocks.insert(
            block_name,
            ParseResult::new(
                PH_POS,
                Value::Block({
                    let mut inner = HashMap::new();
                    inner.insert("source", ParseResult::new(PH_POS, Value::String("source_data")));
                    inner.insert(
                        "traces",
                        ParseResult::new(PH_POS, Value::Vec(vec!["trace1", "trace2", "trace3"])),
                    );
                    sagc::parser::Blocks { blocks: HashMap::<&str, ParseResult<'_>>::from(inner) }
                }),
            ),
        );

        let grafanasingleline = GrafanaSingleLine::new(&mut blocks, block_name);

        assert!(grafanasingleline.is_err());
        let errors = grafanasingleline.unwrap_err();
        assert!(errors.iter().any(|e| matches!(e, SagError::LanguageError(_))));
    }

    #[test]
    fn test_grafanasingleline_invalid_type() {
        let mut blocks = Blocks::new();

        let block_name = "GrafanaSingleLine";
        blocks.insert(
            block_name,
            ParseResult::new(
                PH_POS,
                Value::Block({
                    let mut inner = HashMap::new();
                    inner.insert("type", ParseResult::new(PH_POS, Value::String("invalid_type")));
                    inner.insert("source", ParseResult::new(PH_POS, Value::String("source_data")));
                    inner.insert(
                        "traces",
                        ParseResult::new(PH_POS, Value::Vec(vec!["trace1", "trace2", "trace3"])),
                    );
                    sagc::parser::Blocks { blocks: HashMap::<&str, ParseResult<'_>>::from(inner) }
                }),
            ),
        );

        let grafanasingleline = GrafanaSingleLine::new(&mut blocks, block_name);

        assert!(grafanasingleline.is_err());
        let errors = grafanasingleline.unwrap_err();
        assert!(errors.iter().any(|e| matches!(e, SagError::LanguageError(_))));
    }

    #[test]
    fn test_grafanasingleline_missing_source() {
        let mut blocks = Blocks::new();

        let block_name = "GrafanaSingleLine";
        blocks.insert(
            block_name,
            ParseResult::new(
                PH_POS,
                Value::Block({
                    let mut inner = HashMap::new();
                    inner.insert("type", ParseResult::new(PH_POS, Value::String("smartcomm-simpleline-panel")));
                    inner.insert(
                        "traces",
                        ParseResult::new(PH_POS, Value::Vec(vec!["trace1", "trace2", "trace3"])),
                    );
                    sagc::parser::Blocks { blocks: HashMap::<&str, ParseResult<'_>>::from(inner) }
                }),
            ),
        );

        let grafanasingleline = GrafanaSingleLine::new(&mut blocks, block_name);

        assert!(grafanasingleline.is_err());
        let errors = grafanasingleline.unwrap_err();
        assert!(errors.iter().any(|e| matches!(e, SagError::LanguageError(_))));
    }

    #[test]
    fn test_grafanasingleline_missing_traces() {
        let mut blocks = Blocks::new();

        let block_name = "GrafanaSingleLine";
        blocks.insert(
            block_name,
            ParseResult::new(
                PH_POS,
                Value::Block({
                    let mut inner = HashMap::new();
                    inner.insert("type", ParseResult::new(PH_POS, Value::String("smartcomm-simpleline-panel")));
                    inner.insert("source", ParseResult::new(PH_POS, Value::String("source_data")));
                    sagc::parser::Blocks { blocks: HashMap::<&str, ParseResult<'_>>::from(inner) }
                }),
            ),
        );

        let grafanasingleline = GrafanaSingleLine::new(&mut blocks, block_name);

        assert!(grafanasingleline.is_err());
        let errors = grafanasingleline.unwrap_err();
        assert!(errors.iter().any(|e| matches!(e, SagError::LanguageError(_))));
    }

    //=====GrafanaMultiLine=====
    #[test]
    fn test_grafanamulitline_new_correct() {
        let mut blocks = Blocks::new();

        let block_name = "GrafanaMultiLine";
        blocks.insert(
            block_name,
            ParseResult::new(
                PH_POS,
                Value::Block({
                    let mut inner = HashMap::new();
                    inner.insert("type", ParseResult::new(PH_POS, Value::String("smartcomm-multiplelinechart-panel")));
                    inner.insert("source", ParseResult::new(PH_POS, Value::String("source_data")));
                    inner.insert(
                        "locations",
                        ParseResult::new(PH_POS, Value::Vec(vec!["location1", "location2"])),
                    );
                    inner.insert(
                        "traces",
                        ParseResult::new(PH_POS, Value::Vec(vec!["trace1", "trace2", "trace3"])),
                    );
                    sagc::parser::Blocks { blocks: HashMap::<&str, ParseResult<'_>>::from(inner) }
                }),
            ),
        );

        let grafanamulitline = GrafanaMultiLine::new(&mut blocks, block_name);

        assert!(grafanamulitline.is_ok());
        let grafanamulitline = grafanamulitline.unwrap();

        assert_eq!(grafanamulitline.source, "source_data");
        assert_eq!(grafanamulitline.locations, vec!["location1", "location2"]);
        assert_eq!(grafanamulitline.traces, vec!["trace1", "trace2", "trace3"]);
    }

    #[test]
    fn test_grafanamulitline_missing_type() {
        let mut blocks = Blocks::new();

        let block_name = "GrafanaMultiLine";
        blocks.insert(
            block_name,
            ParseResult::new(
                PH_POS,
                Value::Block({
                    let mut inner = HashMap::new();
                    inner.insert("source", ParseResult::new(PH_POS, Value::String("source_data")));
                    inner.insert(
                        "locations",
                        ParseResult::new(PH_POS, Value::Vec(vec!["location1", "location2"])),
                    );
                    inner.insert(
                        "traces",
                        ParseResult::new(PH_POS, Value::Vec(vec!["trace1", "trace2", "trace3"])),
                    );
                    sagc::parser::Blocks { blocks: HashMap::<&str, ParseResult<'_>>::from(inner) }
                }),
            ),
        );

        let grafanamulitline = GrafanaMultiLine::new(&mut blocks, block_name);

        assert!(grafanamulitline.is_err());
        let errors = grafanamulitline.unwrap_err();
        assert!(errors.iter().any(|e| matches!(e, SagError::LanguageError(_))));
    }

    #[test]
    fn test_grafanamulitline_invalid_type() {
        let mut blocks = Blocks::new();

        let block_name = "GrafanaMultiLine";
        blocks.insert(
            block_name,
            ParseResult::new(
                PH_POS,
                Value::Block({
                    let mut inner = HashMap::new();
                    inner.insert("type", ParseResult::new(PH_POS, Value::String("invalid_type")));
                    inner.insert("source", ParseResult::new(PH_POS, Value::String("source_data")));
                    inner.insert(
                        "locations",
                        ParseResult::new(PH_POS, Value::Vec(vec!["location1", "location2"])),
                    );
                    inner.insert(
                        "traces",
                        ParseResult::new(PH_POS, Value::Vec(vec!["trace1", "trace2", "trace3"])),
                    );
                    sagc::parser::Blocks { blocks: HashMap::<&str, ParseResult<'_>>::from(inner) }
                }),
            ),
        );

        let grafanamulitline = GrafanaMultiLine::new(&mut blocks, block_name);

        assert!(grafanamulitline.is_err());
        let errors = grafanamulitline.unwrap_err();
        assert!(errors.iter().any(|e| matches!(e, SagError::LanguageError(_))));
    }

    #[test]
    fn test_grafanamulitline_missing_source() {
        let mut blocks = Blocks::new();

        let block_name = "GrafanaMultiLine";
        blocks.insert(
            block_name,
            ParseResult::new(
                PH_POS,
                Value::Block({
                    let mut inner = HashMap::new();
                    inner.insert("type", ParseResult::new(PH_POS, Value::String("smartcomm-multiplelinechart-panel")));
                    inner.insert(
                        "locations",
                        ParseResult::new(PH_POS, Value::Vec(vec!["location1", "location2"])),
                    );
                    inner.insert(
                        "traces",
                        ParseResult::new(PH_POS, Value::Vec(vec!["trace1", "trace2", "trace3"])),
                    );
                    sagc::parser::Blocks { blocks: HashMap::<&str, ParseResult<'_>>::from(inner) }
                }),
            ),
        );

        let grafanamulitline = GrafanaMultiLine::new(&mut blocks, block_name);

        assert!(grafanamulitline.is_err());
        let errors = grafanamulitline.unwrap_err();
        assert!(errors.iter().any(|e| matches!(e, SagError::LanguageError(_))));
    }

    #[test]
    fn test_grafanamulitline_missing_locations() {
        let mut blocks = Blocks::new();

        let block_name = "GrafanaMultiLine";
        blocks.insert(
            block_name,
            ParseResult::new(
                PH_POS,
                Value::Block({
                    let mut inner = HashMap::new();
                    inner.insert("type", ParseResult::new(PH_POS, Value::String("smartcomm-multiplelinechart-panel")));
                    inner.insert("source", ParseResult::new(PH_POS, Value::String("source_data")));
                    inner.insert(
                        "traces",
                        ParseResult::new(PH_POS, Value::Vec(vec!["trace1", "trace2", "trace3"])),
                    );
                    sagc::parser::Blocks { blocks: HashMap::<&str, ParseResult<'_>>::from(inner) }
                }),
            ),
        );

        let grafanamulitline = GrafanaMultiLine::new(&mut blocks, block_name);

        assert!(grafanamulitline.is_err());
        let errors = grafanamulitline.unwrap_err();
        assert!(errors.iter().any(|e| matches!(e, SagError::LanguageError(_))));
    }

    #[test]
    fn test_grafanamulitline_missing_traces() {
        let mut blocks = Blocks::new();

        let block_name = "GrafanaMultiLine";
        blocks.insert(
            block_name,
            ParseResult::new(
                PH_POS,
                Value::Block({
                    let mut inner = HashMap::new();
                    inner.insert("type", ParseResult::new(PH_POS, Value::String("smartcomm-multiplelinechart-panel")));
                    inner.insert("source", ParseResult::new(PH_POS, Value::String("source_data")));
                    inner.insert(
                        "locations",
                        ParseResult::new(PH_POS, Value::Vec(vec!["location1", "location2"])),
                    );
                    sagc::parser::Blocks { blocks: HashMap::<&str, ParseResult<'_>>::from(inner) }
                }),
            ),
        );

        let grafanamulitline = GrafanaMultiLine::new(&mut blocks, block_name);

        assert!(grafanamulitline.is_err());
        let errors = grafanamulitline.unwrap_err();
        assert!(errors.iter().any(|e| matches!(e, SagError::LanguageError(_))));
    }
    //=====GrafanaExtValues=====
    #[test]
    fn test_grafanaextvalues_new_correct() {
        let mut blocks = Blocks::new();

        let block_name = "GrafanaExtValues";
        blocks.insert(
            block_name,
            ParseResult::new(
                PH_POS,
                Value::Block({
                    let mut inner = HashMap::new();
                    inner.insert("type", ParseResult::new(PH_POS, Value::String("smartcomm-extremevalues-panel")));
                    inner.insert("source", ParseResult::new(PH_POS, Value::String("source_data")));
                    inner.insert(
                        "locations",
                        ParseResult::new(PH_POS, Value::Vec(vec!["location1", "location2"])),
                    );
                    inner.insert(
                        "traces",
                        ParseResult::new(PH_POS, Value::Vec(vec!["trace1", "trace2", "trace3"])),
                    );
                    sagc::parser::Blocks { blocks: HashMap::<&str, ParseResult<'_>>::from(inner) }
                }),
            ),
        );

        let grafanaextvalues = GrafanaExtValues::new(&mut blocks, block_name);

        assert!(grafanaextvalues.is_ok());
        let grafanaextvalues = grafanaextvalues.unwrap();

        assert_eq!(grafanaextvalues.source, "source_data");
        assert_eq!(grafanaextvalues.locations, vec!["location1", "location2"]);
        assert_eq!(grafanaextvalues.traces, vec!["trace1", "trace2", "trace3"]);
    }

    #[test]
    fn test_grafanaextvalues_missing_type() {
        let mut blocks = Blocks::new();

        let block_name = "GrafanaExtValues";
        blocks.insert(
            block_name,
            ParseResult::new(
                PH_POS,
                Value::Block({
                    let mut inner = HashMap::new();
                    inner.insert("source", ParseResult::new(PH_POS, Value::String("source_data")));
                    inner.insert(
                        "locations",
                        ParseResult::new(PH_POS, Value::Vec(vec!["location1", "location2"])),
                    );
                    inner.insert(
                        "traces",
                        ParseResult::new(PH_POS, Value::Vec(vec!["trace1", "trace2", "trace3"])),
                    );
                    sagc::parser::Blocks { blocks: HashMap::<&str, ParseResult<'_>>::from(inner) }
                }),
            ),
        );

        let grafanaextvalues = GrafanaExtValues::new(&mut blocks, block_name);

        assert!(grafanaextvalues.is_err());
        let errors = grafanaextvalues.unwrap_err();
        assert!(errors.iter().any(|e| matches!(e, SagError::LanguageError(_))));
    }

    #[test]
    fn test_grafanaextvalues_invalid_type() {
        let mut blocks = Blocks::new();

        let block_name = "GrafanaExtValues";
        blocks.insert(
            block_name,
            ParseResult::new(
                PH_POS,
                Value::Block({
                    let mut inner = HashMap::new();
                    inner.insert("type", ParseResult::new(PH_POS, Value::String("invalid_type")));
                    inner.insert("source", ParseResult::new(PH_POS, Value::String("source_data")));
                    inner.insert(
                        "locations",
                        ParseResult::new(PH_POS, Value::Vec(vec!["location1", "location2"])),
                    );
                    inner.insert(
                        "traces",
                        ParseResult::new(PH_POS, Value::Vec(vec!["trace1", "trace2", "trace3"])),
                    );
                    sagc::parser::Blocks { blocks: HashMap::<&str, ParseResult<'_>>::from(inner) }
                }),
            ),
        );

        let grafanaextvalues = GrafanaExtValues::new(&mut blocks, block_name);

        assert!(grafanaextvalues.is_err());
        let errors = grafanaextvalues.unwrap_err();
        assert!(errors.iter().any(|e| matches!(e, SagError::LanguageError(_))));
    }

    #[test]
    fn test_grafanaextvalues_missing_source() {
        let mut blocks = Blocks::new();

        let block_name = "GrafanaExtValues";
        blocks.insert(
            block_name,
            ParseResult::new(
                PH_POS,
                Value::Block({
                    let mut inner = HashMap::new();
                    inner.insert("type", ParseResult::new(PH_POS, Value::String("smartcomm-extremevalues-panel")));
                    inner.insert(
                        "locations",
                        ParseResult::new(PH_POS, Value::Vec(vec!["location1", "location2"])),
                    );
                    inner.insert(
                        "traces",
                        ParseResult::new(PH_POS, Value::Vec(vec!["trace1", "trace2", "trace3"])),
                    );
                    sagc::parser::Blocks { blocks: HashMap::<&str, ParseResult<'_>>::from(inner) }
                }),
            ),
        );

        let grafanaextvalues = GrafanaExtValues::new(&mut blocks, block_name);

        assert!(grafanaextvalues.is_err());
        let errors = grafanaextvalues.unwrap_err();
        assert!(errors.iter().any(|e| matches!(e, SagError::LanguageError(_))));
    }

    #[test]
    fn test_grafanaextvalues_missing_locations() {
        let mut blocks = Blocks::new();

        let block_name = "GrafanaExtValues";
        blocks.insert(
            block_name,
            ParseResult::new(
                PH_POS,
                Value::Block({
                    let mut inner = HashMap::new();
                    inner.insert("type", ParseResult::new(PH_POS, Value::String("smartcomm-extremevalues-panel")));
                    inner.insert("source", ParseResult::new(PH_POS, Value::String("source_data")));
                    inner.insert(
                        "traces",
                        ParseResult::new(PH_POS, Value::Vec(vec!["trace1", "trace2", "trace3"])),
                    );
                    sagc::parser::Blocks { blocks: HashMap::<&str, ParseResult<'_>>::from(inner) }
                }),
            ),
        );

        let grafanaextvalues = GrafanaExtValues::new(&mut blocks, block_name);

        assert!(grafanaextvalues.is_err());
        let errors = grafanaextvalues.unwrap_err();
        assert!(errors.iter().any(|e| matches!(e, SagError::LanguageError(_))));
    }

    #[test]
    fn test_grafanaextvalues_missing_traces() {
        let mut blocks = Blocks::new();

        let block_name = "GrafanaExtValues";
        blocks.insert(
            block_name,
            ParseResult::new(
                PH_POS,
                Value::Block({
                    let mut inner = HashMap::new();
                    inner.insert("type", ParseResult::new(PH_POS, Value::String("smartcomm-extremevalues-panel")));
                    inner.insert("source", ParseResult::new(PH_POS, Value::String("source_data")));
                    inner.insert(
                        "locations",
                        ParseResult::new(PH_POS, Value::Vec(vec!["location1", "location2"])),
                    );
                    sagc::parser::Blocks { blocks: HashMap::<&str, ParseResult<'_>>::from(inner) }
                }),
            ),
        );

        let grafanaextvalues = GrafanaExtValues::new(&mut blocks, block_name);

        assert!(grafanaextvalues.is_err());
        let errors = grafanaextvalues.unwrap_err();
        assert!(errors.iter().any(|e| matches!(e, SagError::LanguageError(_))));
    }

    //=====GrafanaBnB=====
    #[test]
    fn test_grafanabnb_new_correct() {
        let mut blocks = Blocks::new();

        let block_name = "GrafanaBnB";
        blocks.insert(
            block_name,
            ParseResult::new(
                PH_POS,
                Value::Block({
                    let mut inner = HashMap::new();
                    inner.insert("type", ParseResult::new(PH_POS, Value::String("smartcomm-minmaxbarchart-panel")));
                    inner.insert("source", ParseResult::new(PH_POS, Value::String("source_data")));
                    inner.insert(
                        "locations",
                        ParseResult::new(PH_POS, Value::Vec(vec!["location1", "location2"])),
                    );
                    inner.insert(
                        "traces",
                        ParseResult::new(PH_POS, Value::Vec(vec!["trace1", "trace2", "trace3"])),
                    );
                    sagc::parser::Blocks { blocks: HashMap::<&str, ParseResult<'_>>::from(inner) }
                }),
            ),
        );

        let grafanabnb = GrafanaBnB::new(&mut blocks, block_name);

        assert!(grafanabnb.is_ok());
        let grafanabnb = grafanabnb.unwrap();

        assert_eq!(grafanabnb.source, "source_data");
        assert_eq!(grafanabnb.locations, vec!["location1", "location2"]);
        assert_eq!(grafanabnb.traces, vec!["trace1", "trace2", "trace3"]);
    }

    #[test]
    fn test_grafanabnb_missing_type() {
        let mut blocks = Blocks::new();

        let block_name = "GrafanaBnB";
        blocks.insert(
            block_name,
            ParseResult::new(
                PH_POS,
                Value::Block({
                    let mut inner = HashMap::new();
                    inner.insert("source", ParseResult::new(PH_POS, Value::String("source_data")));
                    inner.insert(
                        "locations",
                        ParseResult::new(PH_POS, Value::Vec(vec!["location1", "location2"])),
                    );
                    inner.insert(
                        "traces",
                        ParseResult::new(PH_POS, Value::Vec(vec!["trace1", "trace2", "trace3"])),
                    );
                    sagc::parser::Blocks { blocks: HashMap::<&str, ParseResult<'_>>::from(inner) }
                }),
            ),
        );

        let grafanabnb = GrafanaBnB::new(&mut blocks, block_name);

        assert!(grafanabnb.is_err());
        let errors = grafanabnb.unwrap_err();
        assert!(errors.iter().any(|e| matches!(e, SagError::LanguageError(_))));
    }
    //=====GrafanaBulletGraph=====
    #[test]
    fn test_grafanabulletgraph_new_correct() {
        let mut blocks = Blocks::new();

        let block_name = "GrafanaBulletGraph";
        blocks.insert(
            block_name,
            ParseResult::new(
                PH_POS,
                Value::Block({
                    let mut inner = HashMap::new();
                    inner.insert("type", ParseResult::new(PH_POS, Value::String("smartcomm-bulletgraph-panel")));
                    inner.insert("source", ParseResult::new(PH_POS, Value::String("source_data")));
                    inner.insert(
                        "locations",
                        ParseResult::new(PH_POS, Value::Vec(vec!["location1", "location2"])),
                    );
                    inner.insert(
                        "traces",
                        ParseResult::new(PH_POS, Value::Vec(vec!["trace1", "trace2", "trace3"])),
                    );
                    sagc::parser::Blocks { blocks: HashMap::<&str, ParseResult<'_>>::from(inner) }
                }),
            ),
        );

        let grafanabulletgraph = GrafanaBulletGraph::new(&mut blocks, block_name);

        assert!(grafanabulletgraph.is_ok());
        let grafanabulletgraph = grafanabulletgraph.unwrap();

        assert_eq!(grafanabulletgraph.source, "source_data");
        assert_eq!(grafanabulletgraph.locations, vec!["location1", "location2"]);
        assert_eq!(grafanabulletgraph.traces, vec!["trace1", "trace2", "trace3"]);
    }

    #[test]
    fn test_grafanabulletgraph_missing_type() {
        let mut blocks = Blocks::new();

        let block_name = "GrafanaBulletGraph";
        blocks.insert(
            block_name,
            ParseResult::new(
                PH_POS,
                Value::Block({
                    let mut inner = HashMap::new();
                    inner.insert("source", ParseResult::new(PH_POS, Value::String("source_data")));
                    inner.insert(
                        "locations",
                        ParseResult::new(PH_POS, Value::Vec(vec!["location1", "location2"])),
                    );
                    inner.insert(
                        "traces",
                        ParseResult::new(PH_POS, Value::Vec(vec!["trace1", "trace2", "trace3"])),
                    );
                    sagc::parser::Blocks { blocks: HashMap::<&str, ParseResult<'_>>::from(inner) }
                }),
            ),
        );

        let grafanabulletgraph = GrafanaBulletGraph::new(&mut blocks, block_name);

        assert!(grafanabulletgraph.is_err());
        let errors = grafanabulletgraph.unwrap_err();
        assert!(errors.iter().any(|e| matches!(e, SagError::LanguageError(_))));
    }
    //=====GrafanaCalendar=====
    #[test]
    fn test_grafanacalendar_new_correct() {
        let mut blocks = Blocks::new();

        let block_name = "GrafanaCalendar";
        blocks.insert(
            block_name,
            ParseResult::new(
                PH_POS,
                Value::Block({
                    let mut inner = HashMap::new();
                    inner.insert("type", ParseResult::new(PH_POS, Value::String("smartcomm-calendar-panel")));
                    inner.insert("source", ParseResult::new(PH_POS, Value::String("source_data")));
                    inner.insert(
                        "locations",
                        ParseResult::new(PH_POS, Value::Vec(vec!["location1", "location2"])),
                    );
                    inner.insert(
                        "traces",
                        ParseResult::new(PH_POS, Value::Vec(vec!["trace1", "trace2", "trace3"])),
                    );
                    sagc::parser::Blocks { blocks: HashMap::<&str, ParseResult<'_>>::from(inner) }
                }),
            ),
        );

        let grafanacalendar = GrafanaCalendar::new(&mut blocks, block_name);

        assert!(grafanacalendar.is_ok());
        let grafanacalendar = grafanacalendar.unwrap();

        assert_eq!(grafanacalendar.source, "source_data");
        assert_eq!(grafanacalendar.locations, vec!["location1", "location2"]);
        assert_eq!(grafanacalendar.traces, vec!["trace1", "trace2", "trace3"]);
    }

    #[test]
    fn test_grafanacalendar_missing_type() {
        let mut blocks = Blocks::new();

        let block_name = "GrafanaCalendar";
        blocks.insert(
            block_name,
            ParseResult::new(
                PH_POS,
                Value::Block({
                    let mut inner = HashMap::new();
                    inner.insert("source", ParseResult::new(PH_POS, Value::String("source_data")));
                    inner.insert(
                        "locations",
                        ParseResult::new(PH_POS, Value::Vec(vec!["location1", "location2"])),
                    );
                    inner.insert(
                        "traces",
                        ParseResult::new(PH_POS, Value::Vec(vec!["trace1", "trace2", "trace3"])),
                    );
                    sagc::parser::Blocks { blocks: HashMap::<&str, ParseResult<'_>>::from(inner) }
                }),
            ),
        );

        let grafanacalendar = GrafanaCalendar::new(&mut blocks, block_name);

        assert!(grafanacalendar.is_err());
        let errors = grafanacalendar.unwrap_err();
        assert!(errors.iter().any(|e| matches!(e, SagError::LanguageError(_))));
    }

    #[test]
    fn test_grafanacalendar_invalid_type() {
        let mut blocks = Blocks::new();

        let block_name = "GrafanaCalendar";
        blocks.insert(
            block_name,
            ParseResult::new(
                PH_POS,
                Value::Block({
                    let mut inner = HashMap::new();
                    inner.insert("type", ParseResult::new(PH_POS, Value::String("invalid_type")));
                    inner.insert("source", ParseResult::new(PH_POS, Value::String("source_data")));
                    inner.insert(
                        "locations",
                        ParseResult::new(PH_POS, Value::Vec(vec!["location1", "location2"])),
                    );
                    inner.insert(
                        "traces",
                        ParseResult::new(PH_POS, Value::Vec(vec!["trace1", "trace2", "trace3"])),
                    );
                    sagc::parser::Blocks { blocks: HashMap::<&str, ParseResult<'_>>::from(inner) }
                }),
            ),
        );

        let grafanacalendar = GrafanaCalendar::new(&mut blocks, block_name);

        assert!(grafanacalendar.is_err());
        let errors = grafanacalendar.unwrap_err();
        assert!(errors.iter().any(|e| matches!(e, SagError::LanguageError(_))));
    }

    #[test]
    fn test_grafanacalendar_missing_source() {
        let mut blocks = Blocks::new();

        let block_name = "GrafanaCalendar";
        blocks.insert(
            block_name,
            ParseResult::new(
                PH_POS,
                Value::Block({
                    let mut inner = HashMap::new();
                    inner.insert("type", ParseResult::new(PH_POS, Value::String("smartcomm-calendar-panel")));
                    inner.insert(
                        "locations",
                        ParseResult::new(PH_POS, Value::Vec(vec!["location1", "location2"])),
                    );
                    inner.insert(
                        "traces",
                        ParseResult::new(PH_POS, Value::Vec(vec!["trace1", "trace2", "trace3"])),
                    );
                    sagc::parser::Blocks { blocks: HashMap::<&str, ParseResult<'_>>::from(inner) }
                }),
            ),
        );

        let grafanacalendar = GrafanaCalendar::new(&mut blocks, block_name);

        assert!(grafanacalendar.is_err());
        let errors = grafanacalendar.unwrap_err();
        assert!(errors.iter().any(|e| matches!(e, SagError::LanguageError(_))));
    }

    #[test]
    fn test_grafanacalendar_missing_locations() {
        let mut blocks = Blocks::new();

        let block_name = "GrafanaCalendar";
        blocks.insert(
            block_name,
            ParseResult::new(
                PH_POS,
                Value::Block({
                    let mut inner = HashMap::new();
                    inner.insert("type", ParseResult::new(PH_POS, Value::String("smartcomm-calendar-panel")));
                    inner.insert("source", ParseResult::new(PH_POS, Value::String("source_data")));
                    inner.insert(
                        "traces",
                        ParseResult::new(PH_POS, Value::Vec(vec!["trace1", "trace2", "trace3"])),
                    );
                    sagc::parser::Blocks { blocks: HashMap::<&str, ParseResult<'_>>::from(inner) }
                }),
            ),
        );

        let grafanacalendar = GrafanaCalendar::new(&mut blocks, block_name);

        assert!(grafanacalendar.is_err());
        let errors = grafanacalendar.unwrap_err();
        assert!(errors.iter().any(|e| matches!(e, SagError::LanguageError(_))));
    }

    #[test]
    fn test_grafanacalendar_missing_traces() {
        let mut blocks = Blocks::new();

        let block_name = "GrafanaCalendar";
        blocks.insert(
            block_name,
            ParseResult::new(
                PH_POS,
                Value::Block({
                    let mut inner = HashMap::new();
                    inner.insert("type", ParseResult::new(PH_POS, Value::String("smartcomm-calendar-panel")));
                    inner.insert("source", ParseResult::new(PH_POS, Value::String("source_data")));
                    inner.insert(
                        "locations",
                        ParseResult::new(PH_POS, Value::Vec(vec!["location1", "location2"])),
                    );
                    sagc::parser::Blocks { blocks: HashMap::<&str, ParseResult<'_>>::from(inner) }
                }),
            ),
        );

        let grafanacalendar = GrafanaCalendar::new(&mut blocks, block_name);

        assert!(grafanacalendar.is_err());
        let errors = grafanacalendar.unwrap_err();
        assert!(errors.iter().any(|e| matches!(e, SagError::LanguageError(_))));
    }
}
