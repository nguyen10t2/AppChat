use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use serde::Serialize;

const LATENCY_BUCKET_10_MS: u64 = 10;
const LATENCY_BUCKET_25_MS: u64 = 25;
const LATENCY_BUCKET_50_MS: u64 = 50;
const LATENCY_BUCKET_100_MS: u64 = 100;
const LATENCY_BUCKET_250_MS: u64 = 250;

#[derive(Debug, Clone)]
pub struct RequestContext {
    pub request_id: String,
}

pub struct AppMetrics {
    http_requests_total: AtomicU64,
    ws_reconnect_total: AtomicU64,
    ws_disconnect_total: AtomicU64,
    ws_close_client_total: AtomicU64,
    ws_close_timeout_total: AtomicU64,
    ws_close_protocol_error_total: AtomicU64,
    ws_close_ping_failure_total: AtomicU64,
    ws_close_stream_end_total: AtomicU64,
    message_send_total: AtomicU64,
    message_send_total_ms: AtomicU64,
    message_send_max_ms: AtomicU64,
    message_send_latency_le_10_ms: AtomicU64,
    message_send_latency_le_25_ms: AtomicU64,
    message_send_latency_le_50_ms: AtomicU64,
    message_send_latency_le_100_ms: AtomicU64,
    message_send_latency_le_250_ms: AtomicU64,
    message_send_latency_inf_ms: AtomicU64,
    upload_attempt_total: AtomicU64,
    upload_failure_total: AtomicU64,
}

impl Default for AppMetrics {
    fn default() -> Self {
        Self {
            http_requests_total: AtomicU64::new(0),
            ws_reconnect_total: AtomicU64::new(0),
            ws_disconnect_total: AtomicU64::new(0),
            ws_close_client_total: AtomicU64::new(0),
            ws_close_timeout_total: AtomicU64::new(0),
            ws_close_protocol_error_total: AtomicU64::new(0),
            ws_close_ping_failure_total: AtomicU64::new(0),
            ws_close_stream_end_total: AtomicU64::new(0),
            message_send_total: AtomicU64::new(0),
            message_send_total_ms: AtomicU64::new(0),
            message_send_max_ms: AtomicU64::new(0),
            message_send_latency_le_10_ms: AtomicU64::new(0),
            message_send_latency_le_25_ms: AtomicU64::new(0),
            message_send_latency_le_50_ms: AtomicU64::new(0),
            message_send_latency_le_100_ms: AtomicU64::new(0),
            message_send_latency_le_250_ms: AtomicU64::new(0),
            message_send_latency_inf_ms: AtomicU64::new(0),
            upload_attempt_total: AtomicU64::new(0),
            upload_failure_total: AtomicU64::new(0),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum WsCloseReason {
    ClientClose,
    Timeout,
    ProtocolError,
    PingFailure,
    StreamEnded,
}

#[derive(Debug, Serialize)]
pub struct MetricsSnapshot {
    pub http_requests_total: u64,
    pub ws_reconnect_total: u64,
    pub ws_disconnect_total: u64,
    pub ws_close_client_total: u64,
    pub ws_close_timeout_total: u64,
    pub ws_close_protocol_error_total: u64,
    pub ws_close_ping_failure_total: u64,
    pub ws_close_stream_end_total: u64,
    pub message_send_total: u64,
    pub message_send_avg_ms: f64,
    pub message_send_p50_ms: f64,
    pub message_send_p95_ms: f64,
    pub message_send_p99_ms: f64,
    pub upload_attempt_total: u64,
    pub upload_failure_total: u64,
    pub upload_failure_rate: f64,
}

impl AppMetrics {
    pub fn inc_http_requests(&self) {
        self.http_requests_total.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_ws_reconnect(&self) {
        self.ws_reconnect_total.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_ws_disconnect(&self) {
        self.ws_disconnect_total.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_ws_close_reason(&self, reason: WsCloseReason) {
        match reason {
            WsCloseReason::ClientClose => {
                self.ws_close_client_total.fetch_add(1, Ordering::Relaxed);
            }
            WsCloseReason::Timeout => {
                self.ws_close_timeout_total.fetch_add(1, Ordering::Relaxed);
            }
            WsCloseReason::ProtocolError => {
                self.ws_close_protocol_error_total
                    .fetch_add(1, Ordering::Relaxed);
            }
            WsCloseReason::PingFailure => {
                self.ws_close_ping_failure_total
                    .fetch_add(1, Ordering::Relaxed);
            }
            WsCloseReason::StreamEnded => {
                self.ws_close_stream_end_total.fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    pub fn record_message_send_latency(&self, duration: Duration) {
        let duration_ms = duration.as_millis() as u64;

        self.message_send_total.fetch_add(1, Ordering::Relaxed);
        self.message_send_total_ms
            .fetch_add(duration_ms, Ordering::Relaxed);
        self.message_send_max_ms
            .fetch_max(duration_ms, Ordering::Relaxed);

        match duration_ms {
            ms if ms <= LATENCY_BUCKET_10_MS => {
                self.message_send_latency_le_10_ms
                    .fetch_add(1, Ordering::Relaxed);
            }
            ms if ms <= LATENCY_BUCKET_25_MS => {
                self.message_send_latency_le_25_ms
                    .fetch_add(1, Ordering::Relaxed);
            }
            ms if ms <= LATENCY_BUCKET_50_MS => {
                self.message_send_latency_le_50_ms
                    .fetch_add(1, Ordering::Relaxed);
            }
            ms if ms <= LATENCY_BUCKET_100_MS => {
                self.message_send_latency_le_100_ms
                    .fetch_add(1, Ordering::Relaxed);
            }
            ms if ms <= LATENCY_BUCKET_250_MS => {
                self.message_send_latency_le_250_ms
                    .fetch_add(1, Ordering::Relaxed);
            }
            _ => {
                self.message_send_latency_inf_ms
                    .fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    pub fn inc_upload_attempt(&self) {
        self.upload_attempt_total.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_upload_failure(&self) {
        self.upload_failure_total.fetch_add(1, Ordering::Relaxed);
    }

    pub fn snapshot(&self) -> MetricsSnapshot {
        let message_send_total = self.message_send_total.load(Ordering::Relaxed);
        let message_send_total_ms = self.message_send_total_ms.load(Ordering::Relaxed);
        let message_send_max_ms = self.message_send_max_ms.load(Ordering::Relaxed);
        let upload_attempt_total = self.upload_attempt_total.load(Ordering::Relaxed);
        let upload_failure_total = self.upload_failure_total.load(Ordering::Relaxed);

        let message_send_avg_ms = if message_send_total == 0 {
            0.0
        } else {
            message_send_total_ms as f64 / message_send_total as f64
        };

        let upload_failure_rate = if upload_attempt_total == 0 {
            0.0
        } else {
            upload_failure_total as f64 / upload_attempt_total as f64
        };

        let message_send_p50_ms = self.latency_percentile(0.50, message_send_max_ms);
        let message_send_p95_ms = self.latency_percentile(0.95, message_send_max_ms);
        let message_send_p99_ms = self.latency_percentile(0.99, message_send_max_ms);

        MetricsSnapshot {
            http_requests_total: self.http_requests_total.load(Ordering::Relaxed),
            ws_reconnect_total: self.ws_reconnect_total.load(Ordering::Relaxed),
            ws_disconnect_total: self.ws_disconnect_total.load(Ordering::Relaxed),
            ws_close_client_total: self.ws_close_client_total.load(Ordering::Relaxed),
            ws_close_timeout_total: self.ws_close_timeout_total.load(Ordering::Relaxed),
            ws_close_protocol_error_total: self.ws_close_protocol_error_total.load(Ordering::Relaxed),
            ws_close_ping_failure_total: self.ws_close_ping_failure_total.load(Ordering::Relaxed),
            ws_close_stream_end_total: self.ws_close_stream_end_total.load(Ordering::Relaxed),
            message_send_total,
            message_send_avg_ms,
            message_send_p50_ms,
            message_send_p95_ms,
            message_send_p99_ms,
            upload_attempt_total,
            upload_failure_total,
            upload_failure_rate,
        }
    }

    fn latency_percentile(&self, percentile: f64, max_ms: u64) -> f64 {
        let total = self.message_send_total.load(Ordering::Relaxed);
        if total == 0 {
            return 0.0;
        }

        let target_rank = (percentile * total as f64).ceil() as u64;

        let b10 = self.message_send_latency_le_10_ms.load(Ordering::Relaxed);
        let b25 = b10 + self.message_send_latency_le_25_ms.load(Ordering::Relaxed);
        let b50 = b25 + self.message_send_latency_le_50_ms.load(Ordering::Relaxed);
        let b100 = b50 + self.message_send_latency_le_100_ms.load(Ordering::Relaxed);
        let b250 = b100 + self.message_send_latency_le_250_ms.load(Ordering::Relaxed);

        if target_rank <= b10 {
            LATENCY_BUCKET_10_MS as f64
        } else if target_rank <= b25 {
            LATENCY_BUCKET_25_MS as f64
        } else if target_rank <= b50 {
            LATENCY_BUCKET_50_MS as f64
        } else if target_rank <= b100 {
            LATENCY_BUCKET_100_MS as f64
        } else if target_rank <= b250 {
            LATENCY_BUCKET_250_MS as f64
        } else {
            max_ms.max(LATENCY_BUCKET_250_MS) as f64
        }
    }

    pub fn prometheus_text(&self) -> String {
        let snapshot = self.snapshot();

        let b10 = self.message_send_latency_le_10_ms.load(Ordering::Relaxed);
        let b25 = b10 + self.message_send_latency_le_25_ms.load(Ordering::Relaxed);
        let b50 = b25 + self.message_send_latency_le_50_ms.load(Ordering::Relaxed);
        let b100 = b50 + self.message_send_latency_le_100_ms.load(Ordering::Relaxed);
        let b250 = b100 + self.message_send_latency_le_250_ms.load(Ordering::Relaxed);
        let binf = b250 + self.message_send_latency_inf_ms.load(Ordering::Relaxed);

        format!(
            "# HELP app_http_requests_total Total HTTP requests\n\
# TYPE app_http_requests_total counter\n\
app_http_requests_total {}\n\
# HELP app_ws_reconnect_total Total reconnects inside reconnect window\n\
# TYPE app_ws_reconnect_total counter\n\
app_ws_reconnect_total {}\n\
# HELP app_ws_disconnect_total Total websocket disconnects\n\
# TYPE app_ws_disconnect_total counter\n\
app_ws_disconnect_total {}\n\
# HELP app_ws_close_total Websocket close events by reason\n\
# TYPE app_ws_close_total counter\n\
app_ws_close_total{{reason=\"client_close\"}} {}\n\
app_ws_close_total{{reason=\"timeout\"}} {}\n\
app_ws_close_total{{reason=\"protocol_error\"}} {}\n\
app_ws_close_total{{reason=\"ping_failure\"}} {}\n\
app_ws_close_total{{reason=\"stream_end\"}} {}\n\
# HELP app_message_send_total Total sent messages\n\
# TYPE app_message_send_total counter\n\
app_message_send_total {}\n\
# HELP app_message_send_latency_ms Message send latency histogram in milliseconds\n\
# TYPE app_message_send_latency_ms histogram\n\
app_message_send_latency_ms_bucket{{le=\"10\"}} {}\n\
app_message_send_latency_ms_bucket{{le=\"25\"}} {}\n\
app_message_send_latency_ms_bucket{{le=\"50\"}} {}\n\
app_message_send_latency_ms_bucket{{le=\"100\"}} {}\n\
app_message_send_latency_ms_bucket{{le=\"250\"}} {}\n\
app_message_send_latency_ms_bucket{{le=\"+Inf\"}} {}\n\
app_message_send_latency_ms_sum {}\n\
app_message_send_latency_ms_count {}\n\
# HELP app_upload_attempt_total Total upload attempts\n\
# TYPE app_upload_attempt_total counter\n\
app_upload_attempt_total {}\n\
# HELP app_upload_failure_total Total upload failures\n\
# TYPE app_upload_failure_total counter\n\
app_upload_failure_total {}\n",
            snapshot.http_requests_total,
            snapshot.ws_reconnect_total,
            snapshot.ws_disconnect_total,
            snapshot.ws_close_client_total,
            snapshot.ws_close_timeout_total,
            snapshot.ws_close_protocol_error_total,
            snapshot.ws_close_ping_failure_total,
            snapshot.ws_close_stream_end_total,
            snapshot.message_send_total,
            b10,
            b25,
            b50,
            b100,
            b250,
            binf,
            self.message_send_total_ms.load(Ordering::Relaxed),
            snapshot.message_send_total,
            snapshot.upload_attempt_total,
            snapshot.upload_failure_total,
        )
    }
}