use anyhow::anyhow;
use axum::extract::{Json, Query, State};
use futures::stream::{FuturesOrdered, StreamExt};
use log::warn;
use serde::Serialize;
use sqlx::{Encode, PgPool};
use std::sync::Arc;

use crate::error::AppError;
use crate::options::*;

#[derive(Encode, Serialize, Default, Debug)]
struct GeocodeOutput {
    rating: Option<i32>,
    lon: Option<f64>,
    lat: Option<f64>,
}

async fn geocode_query(
    pool: &PgPool,
    address: &str,
    results: &i32,
) -> anyhow::Result<Vec<GeocodeOutput>> {
    Ok(sqlx::query_as!(
        GeocodeOutput,
        "select g.rating, st_x(g.geomout) as lon, st_y(g.geomout) as lat from geocode($1, $2) as g",
        address,
        results
    )
    .fetch_all(pool)
    .await?)
}

pub(crate) async fn geocode(
    State(pool): State<Arc<PgPool>>,
    Query(options_opt): Query<OptionsOpt>,
    Json(address): Json<String>,
) -> Result<String, AppError> {
    let Options { format: f, results } = merge(options_opt);

    let query_out = geocode_query(&pool, &address, &results).await?;

    if query_out.is_empty() {
        return Err(anyhow!("No results found").into());
    }

    f.print(query_out)
}

#[derive(Encode, Serialize, Default, Debug)]
struct GeocodeBatchOutput {
    address: String,
    rating: Option<i32>,
    lon: Option<f64>,
    lat: Option<f64>,
}

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

    let out = query_out.iter().zip(addresses.iter());

    out.clone().filter(|(o, _)| o.is_err()).for_each(|(o, a)| {
        warn!("Failed to geocode address {:?}: {:?}", a, o);
    });

    let successes = out
        .filter_map(|(o, a)| {
            o.as_ref().ok().map(|t| {
                t.iter()
                    .map(|r| GeocodeBatchOutput {
                        address: a.clone(),
                        rating: r.rating,
                        lon: r.lon,
                        lat: r.lat,
                    })
                    .collect::<Vec<_>>()
            })
        })
        .flatten()
        .collect::<Vec<GeocodeBatchOutput>>();

    f.print(successes)
}
