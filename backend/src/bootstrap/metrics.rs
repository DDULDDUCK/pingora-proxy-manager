use metrics_exporter_prometheus::PrometheusBuilder;

pub fn init_metrics() -> Result<metrics_exporter_prometheus::PrometheusHandle, Box<dyn std::error::Error>> {
    let recorder_handle = PrometheusBuilder::new()
        .install_recorder()
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
    Ok(recorder_handle)
}
