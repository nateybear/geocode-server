use anyhow::anyhow;
use axum::extract::{Json, Query, State};
use futures::stream::{FuturesOrdered, StreamExt};
use serde::Serialize;
use sqlx::{Encode, PgPool};
use std::sync::Arc;
use tracing::{instrument, warn};

use crate::error::AppError;
use crate::options::*;

#[derive(Encode, Serialize, Default, Debug, Clone)]
pub(crate) struct GeocodeOutput {
    rating: Option<i32>,
    lon: Option<f64>,
    lat: Option<f64>,
    stno: Option<i32>,
    street: Option<String>,
    styp: Option<String>,
    city: Option<String>,
    state: Option<String>,
    zip: Option<String>,
}

#[instrument(skip(pool, results))]
async fn geocode_query(
    pool: &PgPool,
    address: &str,
    results: &i32,
) -> anyhow::Result<Vec<GeocodeOutput>> {
    Ok(sqlx::query_as!(
        GeocodeOutput,
        "select g.rating, 
            st_x(g.geomout) as lon, 
            st_y(g.geomout) as lat, 
            (addy).address as stno, 
            (addy).streetname as street, 
            (addy).streettypeabbrev as styp, 
            (addy).location as city, 
            (addy).stateabbrev as state,
            (addy).zip 
        from geocode($1, $2) as g",
        address,
        results
    )
    .fetch_all(pool)
    .await?)
}

#[instrument(skip_all)]
pub(crate) async fn geocode(
    State(pool): State<Arc<PgPool>>,
    Query(options_opt): Query<OptionsOpt>,
    Json(address): Json<String>,
) -> Result<String, AppError> {
    let Options { format: f, results } = merge(options_opt);

    let query_out = geocode_query(&pool, &address, &results).await?;

    Ok(f.print(query_out)?)
}

#[derive(Serialize, Default, Debug)]
pub(crate) struct GeocodeBatchOutput {
    address: String,
    #[serde(flatten)]
    output: GeocodeOutput,
}

#[derive(Serialize, Debug)]
pub(crate) struct GeocodeBatchResults {
    pub(crate) successes: Vec<GeocodeBatchOutput>,
    pub(crate) errors: Vec<(String, String)>,
}

#[instrument(skip_all)]
pub(crate) async fn geocode_batch(
    State(pool): State<Arc<PgPool>>,
    Query(options_opt): Query<OptionsOpt>,
    Json(addresses): Json<Vec<String>>,
) -> Result<String, AppError> {
    let Options { format: f, results } = merge(options_opt);

    let mut query_out = FuturesOrdered::new();
    for address in addresses.iter() {
        let pool = pool.clone();
        query_out.push_back(async move { geocode_query(&pool, address, &results).await });
    }

    let query_out = query_out.collect::<Vec<_>>().await;

    let out = query_out.into_iter().zip(addresses.into_iter());

    let mut errors = vec![];

    let successes = out
        .filter_map(|(query_out, address)| {
            if let Err(e) = &query_out {
                warn!(?address, ?e, "Failed to geocode address");
                errors.push((address, e.to_string()));
                return None;
            }
            query_out.ok().map(|results| {
                results
                    .iter()
                    .map(|r| GeocodeBatchOutput {
                        address: address.clone(),
                        output: r.clone(),
                    })
                    .collect::<Vec<_>>()
            })
        })
        .flatten()
        .collect::<Vec<GeocodeBatchOutput>>();

    let results = GeocodeBatchResults { successes, errors };

    Ok(f.print(results)?)
}
