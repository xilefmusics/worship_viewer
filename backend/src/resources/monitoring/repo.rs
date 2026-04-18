use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Duration, Utc};
use serde::Deserialize;
use surrealdb::sql::{Datetime, Thing};

use shared::api::ListQuery;

use crate::database::Database;
use crate::database::record_id_string;
use crate::error::AppError;

use super::model::{
    ActivityCalendarMetrics, AdminMonitoringMetrics, EngagementMetrics, FamilyErrorRates,
    FamilyLatency, FeatureFamilyMetrics, HttpAuditLog, HttpAuditRecord, IdLike404Metrics,
    LatencyMetrics, METRICS_ACTIVATION_NEW_USER_CAP, MetricsWindow, MetricsWindowWire,
    MonitoringMetricsResponse, MutationHealthMetrics, NewUserActivationMetrics, ReliabilityMetrics,
    RouteFamily, TopFailingRoute, TrafficMetrics, TrafficMixEntry,
};

pub struct MonitoringRepo;

fn surreal_query_err(ctx: &'static str, err: surrealdb::Error) -> AppError {
    crate::observability::log_error_chain(ctx, &err);
    AppError::database(err.to_string())
}

#[derive(Deserialize)]
struct CountRow {
    count: i64,
}

#[derive(Deserialize)]
struct PctRow {
    p95: Option<serde_json::Value>,
    p99: Option<serde_json::Value>,
}

#[derive(Deserialize)]
struct FailPathRow {
    path: String,
    error_count: i64,
}

#[derive(Deserialize)]
struct MethodOnlyRow {
    method: String,
}

#[derive(Deserialize)]
struct NewUserRow {
    id: Thing,
    created_at: Datetime,
}

#[derive(Deserialize)]
struct AuditActivationRow {
    user: Thing,
    #[allow(dead_code)]
    path: String,
    created_at: Datetime,
}

impl MonitoringRepo {
    pub async fn count_http_audit_logs(db: &Database) -> Result<u64, AppError> {
        #[derive(Deserialize)]
        struct CountResult {
            count: u64,
        }
        let mut response = db
            .db
            .query("SELECT count() FROM http_request_audit GROUP ALL")
            .await
            .map_err(|e| surreal_query_err("http_audit.count", e))?;
        Ok(response
            .take::<Vec<CountResult>>(0)
            .map_err(|e| surreal_query_err("http_audit.count.take", e))?
            .into_iter()
            .next()
            .map(|r| r.count)
            .unwrap_or(0))
    }

    pub async fn list_http_audit_logs(
        db: &Database,
        query: ListQuery,
    ) -> Result<Vec<HttpAuditLog>, AppError> {
        let (offset, limit) = query.effective_offset_limit();
        let mut response = db
            .db
            .query(
                "SELECT * FROM http_request_audit ORDER BY created_at DESC LIMIT $limit START $start",
            )
            .bind(("limit", limit))
            .bind(("start", offset))
            .await
            .map_err(|e| surreal_query_err("http_audit.list", e))?;
        let rows: Vec<HttpAuditRecord> = response
            .take(0)
            .map_err(|e| surreal_query_err("http_audit.list.take", e))?;
        Ok(rows.into_iter().map(|r| r.into_wire()).collect())
    }

    pub async fn fetch_metrics(
        db: &Database,
        window: MetricsWindow,
    ) -> Result<MonitoringMetricsResponse, AppError> {
        let start = Datetime::from(window.start);
        let end = Datetime::from(window.end);
        let (month_start, month_end, mau_month) = window.month_of_end_bounds();
        let month_start = Datetime::from(month_start);
        let month_end = Datetime::from(month_end);
        let (day_start, day_end) = window.day_of_end_bounds();
        let day_start = Datetime::from(day_start);
        let day_end = Datetime::from(day_end);
        let wau_start = Datetime::from(window.wau_start());

        let days_spanned = window.utc_days_spanned();

        let total_requests = count_audit(
            db,
            "metrics.total",
            "SELECT count() AS count FROM http_request_audit \
             WHERE created_at >= $start AND created_at < $end GROUP ALL",
            &start,
            &end,
        )
        .await?;
        let total_f = total_requests as f64;

        let error_count_all =
            count_audit_where(db, "metrics.err_all", "status_code >= 400", &start, &end).await?;
        let error_count_api_v1 = count_audit_where(
            db,
            "metrics.err_v1",
            "status_code >= 400 AND string::starts_with(path, '/api/v1/')",
            &start,
            &end,
        )
        .await?;
        let count_5xx_all =
            count_audit_where(db, "metrics.5xx", "status_code >= 500", &start, &end).await?;
        let count_401_all =
            count_audit_where(db, "metrics.401", "status_code = 401", &start, &end).await?;
        let count_403_all =
            count_audit_where(db, "metrics.403", "status_code = 403", &start, &end).await?;
        let count_401_api_v1 = count_audit_where(
            db,
            "metrics.401v1",
            "status_code = 401 AND string::starts_with(path, '/api/v1/')",
            &start,
            &end,
        )
        .await?;
        let count_429_all =
            count_audit_where(db, "metrics.429", "status_code = 429", &start, &end).await?;
        let count_authed =
            count_audit_where(db, "metrics.authed", "user IS NOT NONE", &start, &end).await?;
        let count_api_v1 = count_audit_where(
            db,
            "metrics.mix_v1",
            "string::starts_with(path, '/api/v1/')",
            &start,
            &end,
        )
        .await?;
        let count_auth_paths = count_audit_where(
            db,
            "metrics.mix_auth",
            "string::starts_with(path, '/auth/')",
            &start,
            &end,
        )
        .await?;
        let count_docs_paths = count_audit_where(
            db,
            "metrics.mix_docs",
            "string::starts_with(path, '/api/docs')",
            &start,
            &end,
        )
        .await?;
        let count_mau = distinct_users_product(db, "metrics.mau", &month_start, &month_end).await?;
        let count_dau = distinct_users_product(db, "metrics.dau", &day_start, &day_end).await?;
        let count_wau = distinct_users_product(db, "metrics.wau", &wau_start, &end).await?;
        let product_requests_with_user = count_audit_where(
            db,
            "metrics.prod_req",
            "user IS NOT NONE AND string::starts_with(path, '/api/v1/') AND NOT (string::starts_with(path, '/api/v1/monitoring/'))",
            &start,
            &end,
        )
        .await?;
        let distinct_product_users =
            distinct_users_product(db, "metrics.prod_users", &start, &end).await?;
        let distinct_sessions_end_day =
            distinct_sessions_in_range(db, "metrics.sess_day", &day_start, &day_end, false).await?;
        let sessioned_requests =
            count_audit_where(db, "metrics.sess_req", "session IS NOT NONE", &start, &end).await?;
        let distinct_sessions_window =
            distinct_sessions_in_range(db, "metrics.sess_win", &start, &end, false).await?;
        let monitoring_reqs = count_audit_where(
            db,
            "metrics.mon",
            "string::starts_with(path, '/api/v1/monitoring/')",
            &start,
            &end,
        )
        .await?;
        let distinct_admins = distinct_admins_monitoring(db, &start, &end).await?;
        let total_404 =
            count_audit_where(db, "metrics.404", "status_code = 404", &start, &end).await?;
        let (id_like_404, top_id404) = id_like_404_metrics(db, &start, &end).await?;
        let mutations_total = count_audit_where(
            db,
            "metrics.mut",
            "method IN ['POST', 'PUT', 'PATCH', 'DELETE']",
            &start,
            &end,
        )
        .await?;
        let mut_2xx = count_audit_where(
            db,
            "metrics.mut2",
            "method IN ['POST', 'PUT', 'PATCH', 'DELETE'] AND status_code >= 200 AND status_code <= 299",
            &start,
            &end,
        )
        .await?;
        let mut_4xx = count_audit_where(
            db,
            "metrics.mut4",
            "method IN ['POST', 'PUT', 'PATCH', 'DELETE'] AND status_code >= 400 AND status_code <= 499",
            &start,
            &end,
        )
        .await?;
        let mut_5xx = count_audit_where(
            db,
            "metrics.mut5",
            "method IN ['POST', 'PUT', 'PATCH', 'DELETE'] AND status_code >= 500",
            &start,
            &end,
        )
        .await?;

        let c5_api = count_audit_where(
            db,
            "metrics.5xx_v1",
            "status_code >= 500 AND string::starts_with(path, '/api/v1/')",
            &start,
            &end,
        )
        .await?;
        let c5_auth = count_audit_where(
            db,
            "metrics.5xx_auth",
            "status_code >= 500 AND string::starts_with(path, '/auth/')",
            &start,
            &end,
        )
        .await?;
        let c5_docs = count_audit_where(
            db,
            "metrics.5xx_docs",
            "status_code >= 500 AND string::starts_with(path, '/api/docs')",
            &start,
            &end,
        )
        .await?;
        let c5_other = count_audit_where(
            db,
            "metrics.5xx_other",
            "status_code >= 500 AND NOT (string::starts_with(path, '/api/v1/') OR string::starts_with(path, '/auth/') OR string::starts_with(path, '/api/docs'))",
            &start,
            &end,
        )
        .await?;
        let c_all_api = count_audit_where(
            db,
            "metrics.all_v1",
            "string::starts_with(path, '/api/v1/')",
            &start,
            &end,
        )
        .await?;
        let c_all_auth = count_audit_where(
            db,
            "metrics.all_auth",
            "string::starts_with(path, '/auth/')",
            &start,
            &end,
        )
        .await?;
        let c_all_docs = count_audit_where(
            db,
            "metrics.all_docs",
            "string::starts_with(path, '/api/docs')",
            &start,
            &end,
        )
        .await?;

        let count_other =
            total_requests.saturating_sub(count_api_v1 + count_auth_paths + count_docs_paths);

        let p_all = percentiles_global(db, "metrics.p_all", "", &start, &end).await?;
        let p_v1 = percentiles_global(
            db,
            "metrics.p_v1",
            "AND string::starts_with(path, '/api/v1/')",
            &start,
            &end,
        )
        .await?;
        let p_auth = percentiles_global(
            db,
            "metrics.p_auth",
            "AND string::starts_with(path, '/auth/')",
            &start,
            &end,
        )
        .await?;
        let p_docs = percentiles_global(
            db,
            "metrics.p_docs",
            "AND string::starts_with(path, '/api/docs')",
            &start,
            &end,
        )
        .await?;
        let p_other = percentiles_global(
            db,
            "metrics.p_other",
            "AND NOT (string::starts_with(path, '/api/v1/') OR string::starts_with(path, '/auth/') OR string::starts_with(path, '/api/docs'))",
            &start,
            &end,
        )
        .await?;
        let by_method = percentiles_by_method(db, &start, &end).await?;

        let top_failing = top_failing_paths(db, &start, &end).await?;

        let feature_usage =
            feature_families(db, &start, &end, &month_start, &month_end, count_mau).await?;

        let (activation, activation_skipped_reason) = compute_activation(db, &start, &end).await?;

        let error_rate_all = ratio(error_count_all, total_requests);
        let error_rate_api_v1 = ratio(error_count_api_v1, count_api_v1);
        let rate_5xx_all = ratio(count_5xx_all, total_requests);
        let rate_401_all = ratio(count_401_all, total_requests);
        let rate_403_all = ratio(count_403_all, total_requests);
        let rate_401_api_v1 = ratio(count_401_api_v1, count_api_v1);
        let rate_429_all = ratio(count_429_all, total_requests);
        let authenticated_share = ratio(count_authed, total_requests);

        let dau_over_mau = if count_mau > 0 {
            Some(count_dau as f64 / count_mau as f64)
        } else {
            None
        };

        let requests_per_day = total_requests as f64 / days_spanned as f64;

        let five_xx_by_route_family = vec![
            family_rates(RouteFamily::ApiV1, c_all_api, c5_api),
            family_rates(RouteFamily::Auth, c_all_auth, c5_auth),
            family_rates(RouteFamily::Docs, c_all_docs, c5_docs),
            family_rates(RouteFamily::Other, count_other, c5_other),
        ];

        let mix = vec![
            mix_entry(RouteFamily::ApiV1, count_api_v1, total_f),
            mix_entry(RouteFamily::Auth, count_auth_paths, total_f),
            mix_entry(RouteFamily::Docs, count_docs_paths, total_f),
            mix_entry(RouteFamily::Other, count_other, total_f),
        ];

        let dau_date = window.end.date_naive().format("%Y-%m-%d").to_string();

        Ok(MonitoringMetricsResponse {
            window: MetricsWindowWire {
                start: window.start,
                end: window.end,
                days_spanned,
                total_requests,
                requests_per_day,
            },
            reliability: ReliabilityMetrics {
                error_rate_all,
                error_count_all,
                error_rate_api_v1,
                error_count_api_v1,
                rate_5xx_all,
                count_5xx_all,
                rate_401_all,
                count_401_all,
                rate_403_all,
                count_403_all,
                rate_401_api_v1,
                count_401_api_v1,
                rate_429_all,
                count_429_all,
                five_xx_by_route_family,
                authenticated_share,
            },
            latency: LatencyMetrics {
                p95_ms_all: p_all.0,
                p99_ms_all: p_all.1,
                by_route_family: vec![
                    FamilyLatency {
                        family: RouteFamily::ApiV1,
                        p95_ms: p_v1.0,
                        p99_ms: p_v1.1,
                    },
                    FamilyLatency {
                        family: RouteFamily::Auth,
                        p95_ms: p_auth.0,
                        p99_ms: p_auth.1,
                    },
                    FamilyLatency {
                        family: RouteFamily::Docs,
                        p95_ms: p_docs.0,
                        p99_ms: p_docs.1,
                    },
                    FamilyLatency {
                        family: RouteFamily::Other,
                        p95_ms: p_other.0,
                        p99_ms: p_other.1,
                    },
                ],
                by_method,
            },
            activity_calendar: ActivityCalendarMetrics {
                mau_month,
                mau_users: count_mau,
                dau_date,
                dau_users: count_dau,
                dau_over_mau,
                wau_rolling_7d_users: count_wau,
            },
            traffic: TrafficMetrics { mix },
            top_failing_routes: top_failing,
            feature_usage,
            mutations: MutationHealthMetrics {
                total_mutations: mutations_total,
                mutations_2xx: mut_2xx,
                mutations_4xx: mut_4xx,
                mutations_5xx: mut_5xx,
            },
            engagement: EngagementMetrics {
                requests_per_active_user_product: ratio_opt(
                    product_requests_with_user,
                    distinct_product_users,
                ),
                product_requests_with_user,
                distinct_active_users_product: distinct_product_users,
                distinct_sessions_on_end_day: distinct_sessions_end_day,
                requests_per_session: ratio_opt(sessioned_requests, distinct_sessions_window),
                sessioned_requests_in_window: sessioned_requests,
                distinct_sessions_in_window: distinct_sessions_window,
            },
            admin_monitoring: AdminMonitoringMetrics {
                request_count: monitoring_reqs,
                distinct_admin_users: distinct_admins,
            },
            new_user_activation: activation,
            activation_skipped_reason,
            probing: IdLike404Metrics {
                id_like_404_count: id_like_404,
                total_404_count: total_404,
                id_like_404_share_of_404: ratio(id_like_404, total_404),
                top_id_like_404_paths: top_id404,
            },
        })
    }
}

fn ratio(num: u64, den: u64) -> f64 {
    if den == 0 {
        0.0
    } else {
        num as f64 / den as f64
    }
}

fn ratio_opt(num: u64, den: u64) -> Option<f64> {
    if den == 0 {
        None
    } else {
        Some(num as f64 / den as f64)
    }
}

fn mix_entry(family: RouteFamily, count: u64, total: f64) -> TrafficMixEntry {
    TrafficMixEntry {
        family,
        count,
        share: if total <= 0.0 {
            0.0
        } else {
            count as f64 / total
        },
    }
}

fn family_rates(family: RouteFamily, total: u64, five: u64) -> FamilyErrorRates {
    FamilyErrorRates {
        family,
        total_requests: total,
        count_5xx: five,
        rate_5xx: ratio(five, total),
    }
}

async fn count_audit(
    db: &Database,
    ctx: &'static str,
    q: &str,
    start: &Datetime,
    end: &Datetime,
) -> Result<u64, AppError> {
    let mut response = db
        .db
        .query(q)
        .bind(("start", start.clone()))
        .bind(("end", end.clone()))
        .await
        .map_err(|e| surreal_query_err(ctx, e))?;
    let rows: Vec<CountRow> = response
        .take(0)
        .map_err(|e| surreal_query_err("metrics.surreal.take", e))?;
    Ok(rows
        .into_iter()
        .next()
        .map(|r| r.count.max(0) as u64)
        .unwrap_or(0))
}

async fn count_audit_where(
    db: &Database,
    ctx: &'static str,
    cond: &str,
    start: &Datetime,
    end: &Datetime,
) -> Result<u64, AppError> {
    let q = format!(
        "SELECT count() AS count FROM http_request_audit \
         WHERE created_at >= $start AND created_at < $end AND {cond} GROUP ALL"
    );
    count_audit(db, ctx, &q, start, end).await
}

async fn distinct_users_product(
    db: &Database,
    ctx: &'static str,
    range_start: &Datetime,
    range_end: &Datetime,
) -> Result<u64, AppError> {
    let q = "SELECT count() AS count FROM (SELECT user FROM http_request_audit \
             WHERE created_at >= $start AND created_at < $end \
             AND user IS NOT NONE \
             AND string::starts_with(path, '/api/v1/') AND NOT (string::starts_with(path, '/api/v1/monitoring/')) \
             GROUP BY user)";
    let mut response = db
        .db
        .query(q)
        .bind(("start", range_start.clone()))
        .bind(("end", range_end.clone()))
        .await
        .map_err(|e| surreal_query_err(ctx, e))?;
    let rows: Vec<CountRow> = response
        .take(0)
        .map_err(|e| surreal_query_err("metrics.surreal.take", e))?;
    Ok(rows
        .into_iter()
        .next()
        .map(|r| r.count.max(0) as u64)
        .unwrap_or(0))
}

async fn distinct_sessions_in_range(
    db: &Database,
    ctx: &'static str,
    range_start: &Datetime,
    range_end: &Datetime,
    product_only: bool,
) -> Result<u64, AppError> {
    let extra = if product_only {
        " AND string::starts_with(path, '/api/v1/') AND NOT (string::starts_with(path, '/api/v1/monitoring/'))"
    } else {
        ""
    };
    let q = format!(
        "SELECT count() AS count FROM (SELECT session FROM http_request_audit \
         WHERE created_at >= $start AND created_at < $end \
         AND session IS NOT NONE {extra} \
         GROUP BY session)"
    );
    let mut response = db
        .db
        .query(q)
        .bind(("start", range_start.clone()))
        .bind(("end", range_end.clone()))
        .await
        .map_err(|e| surreal_query_err(ctx, e))?;
    let rows: Vec<CountRow> = response
        .take(0)
        .map_err(|e| surreal_query_err("metrics.surreal.take", e))?;
    Ok(rows
        .into_iter()
        .next()
        .map(|r| r.count.max(0) as u64)
        .unwrap_or(0))
}

async fn distinct_admins_monitoring(
    db: &Database,
    start: &Datetime,
    end: &Datetime,
) -> Result<u64, AppError> {
    let q = "SELECT count() AS count FROM (SELECT user FROM http_request_audit \
             WHERE created_at >= $start AND created_at < $end \
             AND string::starts_with(path, '/api/v1/monitoring/') \
             AND user IS NOT NONE \
             AND user IN (SELECT id FROM user WHERE role = 'admin') \
             GROUP BY user)";
    let mut response = db
        .db
        .query(q)
        .bind(("start", start.clone()))
        .bind(("end", end.clone()))
        .await
        .map_err(|e| surreal_query_err("metrics.adm_dist", e))?;
    let rows: Vec<CountRow> = response
        .take(0)
        .map_err(|e| surreal_query_err("metrics.adm_dist.take", e))?;
    Ok(rows
        .into_iter()
        .next()
        .map(|r| r.count.max(0) as u64)
        .unwrap_or(0))
}

fn json_to_f64(v: &serde_json::Value) -> Option<f64> {
    match v {
        serde_json::Value::Number(n) => n.as_f64().or_else(|| n.as_i64().map(|i| i as f64)),
        _ => None,
    }
}

async fn percentiles_global(
    db: &Database,
    ctx: &'static str,
    and_cond: &str,
    start: &Datetime,
    end: &Datetime,
) -> Result<(Option<f64>, Option<f64>), AppError> {
    let n = count_audit_where(
        db,
        "metrics.pct_n",
        &format!("duration_ms >= 0 {and_cond}"),
        start,
        end,
    )
    .await?;
    if n == 0 {
        return Ok((None, None));
    }

    let q = format!(
        "LET $d = (SELECT VALUE duration_ms FROM http_request_audit \
           WHERE created_at >= $start AND created_at < $end {and_cond}); \
         RETURN {{ p95: math::percentile($d, 95), p99: math::percentile($d, 99) }}",
        and_cond = and_cond
    );
    let mut response = db
        .db
        .query(q)
        .bind(("start", start.clone()))
        .bind(("end", end.clone()))
        .await
        .map_err(|e| surreal_query_err(ctx, e))?;
    let rows: Vec<PctRow> = response
        .take(1)
        .map_err(|e| surreal_query_err("metrics.surreal.take", e))?;
    let row = rows.into_iter().next();
    let p95 = row
        .as_ref()
        .and_then(|r| r.p95.as_ref())
        .and_then(json_to_f64);
    let p99 = row
        .as_ref()
        .and_then(|r| r.p99.as_ref())
        .and_then(json_to_f64);
    Ok((p95, p99))
}

async fn percentiles_by_method(
    db: &Database,
    start: &Datetime,
    end: &Datetime,
) -> Result<Vec<super::model::MethodLatency>, AppError> {
    let q_methods = "SELECT method FROM http_request_audit \
         WHERE created_at >= $start AND created_at < $end \
         GROUP BY method";
    let mut response = db
        .db
        .query(q_methods)
        .bind(("start", start.clone()))
        .bind(("end", end.clone()))
        .await
        .map_err(|e| surreal_query_err("metrics.p_method.list", e))?;
    let methods: Vec<MethodOnlyRow> = response
        .take(0)
        .map_err(|e| surreal_query_err("metrics.p_method.list.take", e))?;

    let mut out: Vec<super::model::MethodLatency> = Vec::new();
    for row in methods {
        let n = count_audit_where(
            db,
            "metrics.p_method.n",
            &format!(
                "duration_ms >= 0 AND method = {}",
                sql_string_literal(&row.method)
            ),
            start,
            end,
        )
        .await?;
        if n == 0 {
            continue;
        }
        let q = format!(
            "LET $d = (SELECT VALUE duration_ms FROM http_request_audit \
               WHERE created_at >= $start AND created_at < $end AND method = {}); \
             RETURN {{ p95: math::percentile($d, 95), p99: math::percentile($d, 99) }}",
            sql_string_literal(&row.method)
        );
        let mut response = db
            .db
            .query(q)
            .bind(("start", start.clone()))
            .bind(("end", end.clone()))
            .await
            .map_err(|e| surreal_query_err("metrics.p_method.pct", e))?;
        let pct_rows: Vec<PctRow> = response
            .take(1)
            .map_err(|e| surreal_query_err("metrics.p_method.pct.take", e))?;
        let pct = pct_rows.into_iter().next();
        let p95 = pct
            .as_ref()
            .and_then(|r| r.p95.as_ref())
            .and_then(json_to_f64);
        let p99 = pct
            .as_ref()
            .and_then(|r| r.p99.as_ref())
            .and_then(json_to_f64);
        out.push(super::model::MethodLatency {
            method: row.method,
            p95_ms: p95,
            p99_ms: p99,
        });
    }
    out.sort_by(|a, b| a.method.cmp(&b.method));
    Ok(out)
}

/// Escape a string for safe embedding as a SurrealQL string literal (e.g. `method = 'GET'`).
fn sql_string_literal(s: &str) -> String {
    let escaped = s.replace('\'', "''");
    format!("'{escaped}'")
}

async fn top_failing_paths(
    db: &Database,
    start: &Datetime,
    end: &Datetime,
) -> Result<Vec<TopFailingRoute>, AppError> {
    let q = "SELECT path, count() AS error_count FROM http_request_audit \
             WHERE created_at >= $start AND created_at < $end AND status_code >= 400 \
             GROUP BY path ORDER BY error_count DESC LIMIT 20";
    let mut response = db
        .db
        .query(q)
        .bind(("start", start.clone()))
        .bind(("end", end.clone()))
        .await
        .map_err(|e| surreal_query_err("metrics.fail_paths", e))?;
    let rows: Vec<FailPathRow> = response
        .take(0)
        .map_err(|e| surreal_query_err("metrics.fail_paths.take", e))?;
    Ok(rows
        .into_iter()
        .map(|r| TopFailingRoute {
            path: r.path,
            error_count: r.error_count.max(0) as u64,
        })
        .collect())
}

fn path_looks_like_id_probe_404(path: &str) -> bool {
    let path = path.split('?').next().unwrap_or(path);
    if path.split('/').any(is_uuid_like_segment) {
        return true;
    }
    if string_starts_with(path, "/api/v1/")
        && let Some(seg) = path.rsplit('/').next()
        && !seg.is_empty()
        && seg.chars().all(|c| c.is_ascii_digit())
    {
        return true;
    }
    false
}

fn string_starts_with(hay: &str, needle: &str) -> bool {
    hay.len() >= needle.len() && hay.as_bytes().starts_with(needle.as_bytes())
}

fn is_uuid_like_segment(seg: &str) -> bool {
    if seg.len() != 36 || seg.matches('-').count() != 4 {
        return false;
    }
    seg.chars().all(|c| c.is_ascii_hexdigit() || c == '-')
}

/// Groups 404s by path in the database, then applies [`path_looks_like_id_probe_404`] in Rust.
async fn id_like_404_metrics(
    db: &Database,
    start: &Datetime,
    end: &Datetime,
) -> Result<(u64, Vec<TopFailingRoute>), AppError> {
    let q = "SELECT path, count() AS error_count FROM http_request_audit \
             WHERE created_at >= $start AND created_at < $end AND status_code = 404 \
             GROUP BY path";
    let mut response = db
        .db
        .query(q)
        .bind(("start", start.clone()))
        .bind(("end", end.clone()))
        .await
        .map_err(|e| surreal_query_err("metrics.id404_group", e))?;
    let rows: Vec<FailPathRow> = response
        .take(0)
        .map_err(|e| surreal_query_err("metrics.id404_group.take", e))?;

    let mut id_like: u64 = 0;
    let mut top: Vec<TopFailingRoute> = Vec::new();
    for r in rows {
        let n = r.error_count.max(0) as u64;
        if path_looks_like_id_probe_404(&r.path) {
            id_like += n;
            top.push(TopFailingRoute {
                path: r.path,
                error_count: n,
            });
        }
    }
    top.sort_by_key(|r| std::cmp::Reverse(r.error_count));
    top.truncate(20);
    Ok((id_like, top))
}

const FEATURES: &[(&str, &str)] = &[
    ("songs", "/api/v1/songs"),
    ("collections", "/api/v1/collections"),
    ("setlists", "/api/v1/setlists"),
    ("blobs", "/api/v1/blobs"),
    ("teams", "/api/v1/teams"),
    ("users", "/api/v1/users"),
];

async fn feature_families(
    db: &Database,
    win_start: &Datetime,
    win_end: &Datetime,
    month_start: &Datetime,
    month_end: &Datetime,
    mau_users: u64,
) -> Result<Vec<FeatureFamilyMetrics>, AppError> {
    let mut out = Vec::with_capacity(FEATURES.len());
    for (key, pat) in FEATURES {
        let req_c = count_audit_where(
            db,
            "metrics.feat_req",
            &format!("string::starts_with(path, '{pat}')"),
            win_start,
            win_end,
        )
        .await?;
        let q_dist_win = format!(
            "SELECT count() AS count FROM (SELECT user FROM http_request_audit \
             WHERE created_at >= $start AND created_at < $end \
             AND user IS NOT NONE AND string::starts_with(path, '{pat}') \
             GROUP BY user)"
        );
        let dist_win =
            distinct_from_subquery(db, "metrics.feat_dw", &q_dist_win, win_start, win_end).await?;
        let q_dist_mo = format!(
            "SELECT count() AS count FROM (SELECT user FROM http_request_audit \
             WHERE created_at >= $start AND created_at < $end \
             AND user IS NOT NONE AND string::starts_with(path, '{pat}') \
             GROUP BY user)"
        );
        let dist_mo =
            distinct_from_subquery(db, "metrics.feat_dm", &q_dist_mo, month_start, month_end)
                .await?;
        let pct = if mau_users > 0 {
            dist_mo as f64 / mau_users as f64
        } else {
            0.0
        };
        out.push(FeatureFamilyMetrics {
            key: (*key).into(),
            path_prefix: format!("/api/v1/{key}"),
            request_count: req_c,
            distinct_users_in_window: dist_win,
            distinct_users_in_mau_month: dist_mo,
            pct_of_mau: pct,
        });
    }
    Ok(out)
}

async fn distinct_from_subquery(
    db: &Database,
    ctx: &'static str,
    q: &str,
    start: &Datetime,
    end: &Datetime,
) -> Result<u64, AppError> {
    let mut response = db
        .db
        .query(q)
        .bind(("start", start.clone()))
        .bind(("end", end.clone()))
        .await
        .map_err(|e| surreal_query_err(ctx, e))?;
    let rows: Vec<CountRow> = response
        .take(0)
        .map_err(|e| surreal_query_err("metrics.surreal.take", e))?;
    Ok(rows
        .into_iter()
        .next()
        .map(|r| r.count.max(0) as u64)
        .unwrap_or(0))
}

fn is_product_path(path: &str) -> bool {
    path.starts_with("/api/v1/") && !path.starts_with("/api/v1/monitoring/")
}

async fn compute_activation(
    db: &Database,
    start: &Datetime,
    end: &Datetime,
) -> Result<(Option<NewUserActivationMetrics>, Option<String>), AppError> {
    let cap = METRICS_ACTIVATION_NEW_USER_CAP + 1;
    let q = format!(
        "SELECT id, created_at FROM user \
         WHERE created_at >= $start AND created_at < $end \
         LIMIT {cap}"
    );
    let mut response = db
        .db
        .query(q)
        .bind(("start", start.clone()))
        .bind(("end", end.clone()))
        .await
        .map_err(|e| surreal_query_err("metrics.act_users", e))?;
    let rows: Vec<NewUserRow> = response
        .take(0)
        .map_err(|e| surreal_query_err("metrics.act_users.take", e))?;
    if rows.len() > METRICS_ACTIVATION_NEW_USER_CAP {
        return Ok((None, Some("too_many_new_users".into())));
    }
    if rows.is_empty() {
        return Ok((
            Some(NewUserActivationMetrics {
                new_users_in_window: 0,
                activated_within_7d: 0,
                activation_rate: 0.0,
            }),
            None,
        ));
    }

    let mut min_t: Option<DateTime<Utc>> = None;
    let mut max_t: Option<DateTime<Utc>> = None;
    let mut user_created: HashMap<String, DateTime<Utc>> = HashMap::new();
    for r in &rows {
        let id = record_id_string(&r.id);
        let ct: DateTime<Utc> = r.created_at.clone().into();
        user_created.insert(id, ct);
        min_t = Some(match min_t {
            Some(m) => m.min(ct),
            None => ct,
        });
        max_t = Some(match max_t {
            Some(m) => m.max(ct),
            None => ct,
        });
    }
    let audit_min = Datetime::from(min_t.expect("non-empty rows"));
    let audit_max = Datetime::from(max_t.expect("non-empty rows") + Duration::days(7));

    let user_things: Vec<Thing> = rows.iter().map(|r| r.id.clone()).collect();
    let q2 = "SELECT user, path, created_at FROM http_request_audit \
              WHERE user IN $users AND created_at >= $a_start AND created_at < $a_end";
    let mut response = db
        .db
        .query(q2)
        .bind(("users", user_things))
        .bind(("a_start", audit_min))
        .bind(("a_end", audit_max))
        .await
        .map_err(|e| surreal_query_err("metrics.act_audit", e))?;
    let hits: Vec<AuditActivationRow> = response
        .take(0)
        .map_err(|e| surreal_query_err("metrics.act_audit.take", e))?;

    let mut activated: HashSet<String> = HashSet::new();
    for h in hits {
        let uid = record_id_string(&h.user);
        let Some(&signup) = user_created.get(&uid) else {
            continue;
        };
        let at: DateTime<Utc> = h.created_at.clone().into();
        if at < signup || at >= signup + Duration::days(7) {
            continue;
        }
        if is_product_path(&h.path) {
            activated.insert(uid);
        }
    }

    let new_users_in_window = rows.len() as u64;
    let activated_within_7d = activated.len() as u64;
    let activation_rate = ratio(activated_within_7d, new_users_in_window);

    Ok((
        Some(NewUserActivationMetrics {
            new_users_in_window,
            activated_within_7d,
            activation_rate,
        }),
        None,
    ))
}
