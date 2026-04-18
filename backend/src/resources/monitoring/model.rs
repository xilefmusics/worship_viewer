use chrono::{DateTime, Datelike, Duration, NaiveDate, Utc};
use serde::Serialize;
use surrealdb::sql::{Datetime, Thing};
use utoipa::{IntoParams, ToSchema};

use crate::database::record_id_string;

/// Maximum length of `[start, end)` for metrics queries (avoids unbounded table scans).
pub const METRICS_MAX_WINDOW_DAYS: i64 = 90;

/// When computing new-user activation, refuse if more than this many users were created in the window.
pub const METRICS_ACTIVATION_NEW_USER_CAP: usize = 10_000;

#[derive(Debug, Clone, serde::Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct MonitoringMetricsQuery {
    /// Inclusive lower bound (UTC, RFC 3339).
    pub start: DateTime<Utc>,
    /// Exclusive upper bound (UTC, RFC 3339).
    pub end: DateTime<Utc>,
}

impl MonitoringMetricsQuery {
    pub fn validate(self) -> Result<Self, String> {
        if self.start >= self.end {
            return Err("start must be before end".into());
        }
        let max = Duration::days(METRICS_MAX_WINDOW_DAYS);
        if self.end.signed_duration_since(self.start) > max {
            return Err(format!(
                "window must be at most {METRICS_MAX_WINDOW_DAYS} days"
            ));
        }
        Ok(self)
    }

    pub fn as_window(&self) -> MetricsWindow {
        MetricsWindow {
            start: self.start,
            end: self.end,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct MetricsWindow {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

impl MetricsWindow {
    pub fn utc_days_spanned(self) -> i64 {
        let s = self.start.date_naive();
        let e = self.end.date_naive();
        (e.signed_duration_since(s).num_days() + 1).max(1)
    }

    pub fn wau_start(self) -> DateTime<Utc> {
        self.end - Duration::days(7)
    }

    pub fn day_of_end_bounds(self) -> (DateTime<Utc>, DateTime<Utc>) {
        let d = self.end.date_naive();
        let day_start = d
            .and_hms_opt(0, 0, 0)
            .map(|t| DateTime::<Utc>::from_naive_utc_and_offset(t, Utc))
            .expect("midnight");
        (day_start, day_start + Duration::days(1))
    }

    pub fn month_of_end_bounds(self) -> (DateTime<Utc>, DateTime<Utc>, String) {
        let d = self.end.date_naive();
        let month_start =
            NaiveDate::from_ymd_opt(d.year(), d.month(), 1).expect("valid month start");
        let month_start_dt = month_start
            .and_hms_opt(0, 0, 0)
            .map(|t| DateTime::<Utc>::from_naive_utc_and_offset(t, Utc))
            .expect("midnight");
        let next_month = if d.month() == 12 {
            NaiveDate::from_ymd_opt(d.year() + 1, 1, 1)
        } else {
            NaiveDate::from_ymd_opt(d.year(), d.month() + 1, 1)
        }
        .expect("next month");
        let month_end_dt = next_month
            .and_hms_opt(0, 0, 0)
            .map(|t| DateTime::<Utc>::from_naive_utc_and_offset(t, Utc))
            .expect("midnight");
        let label = format!("{:04}-{:02}", d.year(), d.month());
        (month_start_dt, month_end_dt, label)
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct HttpAuditRecord {
    #[serde(default)]
    pub id: Option<Thing>,
    pub request_id: String,
    pub method: String,
    pub path: String,
    pub status_code: i64,
    pub duration_ms: i64,
    #[serde(default)]
    pub user: Option<Thing>,
    #[serde(default)]
    pub session: Option<Thing>,
    pub created_at: Datetime,
}

impl HttpAuditRecord {
    pub fn into_wire(self) -> HttpAuditLog {
        let id = self.id.as_ref().map(record_id_string).unwrap_or_default();
        HttpAuditLog {
            id,
            request_id: self.request_id,
            method: self.method,
            path: self.path,
            status_code: self.status_code as i32,
            duration_ms: self.duration_ms as i32,
            user_id: self.user.as_ref().map(record_id_string),
            session_id: self.session.as_ref().map(record_id_string),
            created_at: self.created_at.into(),
        }
    }
}

/// One persisted HTTP request audit row (admin monitoring API).
#[derive(Debug, Serialize, ToSchema)]
pub struct HttpAuditLog {
    pub id: String,
    pub request_id: String,
    pub method: String,
    pub path: String,
    pub status_code: i32,
    pub duration_ms: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    pub created_at: DateTime<Utc>,
}

// --- Metrics bundle (GET /monitoring/metrics) ---

#[derive(Debug, Serialize, ToSchema)]
pub struct MonitoringMetricsResponse {
    pub window: MetricsWindowWire,
    pub reliability: ReliabilityMetrics,
    pub latency: LatencyMetrics,
    pub activity_calendar: ActivityCalendarMetrics,
    pub traffic: TrafficMetrics,
    pub top_failing_routes: Vec<TopFailingRoute>,
    pub feature_usage: Vec<FeatureFamilyMetrics>,
    pub mutations: MutationHealthMetrics,
    pub engagement: EngagementMetrics,
    pub admin_monitoring: AdminMonitoringMetrics,
    pub new_user_activation: Option<NewUserActivationMetrics>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub activation_skipped_reason: Option<String>,
    pub probing: IdLike404Metrics,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MetricsWindowWire {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub days_spanned: i64,
    pub total_requests: u64,
    pub requests_per_day: f64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ReliabilityMetrics {
    /// `status_code >= 400` / total in window.
    pub error_rate_all: f64,
    pub error_count_all: u64,
    /// Same, paths under `/api/v1/`.
    pub error_rate_api_v1: f64,
    pub error_count_api_v1: u64,
    pub rate_5xx_all: f64,
    pub count_5xx_all: u64,
    pub rate_401_all: f64,
    pub count_401_all: u64,
    pub rate_403_all: f64,
    pub count_403_all: u64,
    pub rate_401_api_v1: f64,
    pub count_401_api_v1: u64,
    pub rate_429_all: f64,
    pub count_429_all: u64,
    pub five_xx_by_route_family: Vec<FamilyErrorRates>,
    pub authenticated_share: f64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct FamilyErrorRates {
    pub family: RouteFamily,
    pub total_requests: u64,
    pub count_5xx: u64,
    pub rate_5xx: f64,
}

#[derive(Debug, Clone, Copy, Serialize, ToSchema, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum RouteFamily {
    ApiV1,
    Auth,
    Docs,
    Other,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct LatencyMetrics {
    pub p95_ms_all: Option<f64>,
    pub p99_ms_all: Option<f64>,
    pub by_route_family: Vec<FamilyLatency>,
    pub by_method: Vec<MethodLatency>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct FamilyLatency {
    pub family: RouteFamily,
    pub p95_ms: Option<f64>,
    pub p99_ms: Option<f64>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MethodLatency {
    pub method: String,
    pub p95_ms: Option<f64>,
    pub p99_ms: Option<f64>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ActivityCalendarMetrics {
    pub mau_month: String,
    pub mau_users: u64,
    pub dau_date: String,
    pub dau_users: u64,
    pub dau_over_mau: Option<f64>,
    pub wau_rolling_7d_users: u64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TrafficMixEntry {
    pub family: RouteFamily,
    pub count: u64,
    pub share: f64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TrafficMetrics {
    pub mix: Vec<TrafficMixEntry>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TopFailingRoute {
    pub path: String,
    pub error_count: u64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct FeatureFamilyMetrics {
    pub key: String,
    pub path_prefix: String,
    pub request_count: u64,
    pub distinct_users_in_window: u64,
    pub distinct_users_in_mau_month: u64,
    pub pct_of_mau: f64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MutationHealthMetrics {
    pub total_mutations: u64,
    pub mutations_2xx: u64,
    pub mutations_4xx: u64,
    pub mutations_5xx: u64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct EngagementMetrics {
    pub requests_per_active_user_product: Option<f64>,
    pub product_requests_with_user: u64,
    pub distinct_active_users_product: u64,
    pub distinct_sessions_on_end_day: u64,
    pub requests_per_session: Option<f64>,
    pub sessioned_requests_in_window: u64,
    pub distinct_sessions_in_window: u64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AdminMonitoringMetrics {
    pub request_count: u64,
    pub distinct_admin_users: u64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct NewUserActivationMetrics {
    pub new_users_in_window: u64,
    pub activated_within_7d: u64,
    pub activation_rate: f64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct IdLike404Metrics {
    pub id_like_404_count: u64,
    pub total_404_count: u64,
    pub id_like_404_share_of_404: f64,
    pub top_id_like_404_paths: Vec<TopFailingRoute>,
}
