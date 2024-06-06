use optfield::optfield;
use serde::Deserialize;

use crate::{GeocodeBatchResults, GeocodeOutput};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub(crate) enum OutFormat {
    Json,
    Csv,
}

pub(crate) trait Print<T> {
    fn print(&self, out: T) -> anyhow::Result<String>;
}

impl Print<Vec<GeocodeOutput>> for OutFormat {
    fn print(&self, out: Vec<GeocodeOutput>) -> anyhow::Result<String> {
        match self {
            OutFormat::Json => Ok(serde_json::to_string(&out)?),
            OutFormat::Csv => {
                let mut wtr = csv::Writer::from_writer(vec![]);
                wtr.serialize(out)?;
                Ok(String::from_utf8(wtr.into_inner()?)?)
            }
        }
    }
}

impl Print<GeocodeBatchResults> for OutFormat {
    fn print(&self, out: GeocodeBatchResults) -> anyhow::Result<String> {
        match self {
            OutFormat::Json => Ok(serde_json::to_string(&out)?),
            OutFormat::Csv => {
                let mut wtr = csv::Writer::from_writer(vec![]);
                for a in out.successes {
                    wtr.serialize(a)?;
                }
                Ok(String::from_utf8(wtr.into_inner()?)?)
            }
        }
    }
}

#[optfield(pub(crate) OptionsOpt, attrs, merge_fn)]
#[derive(Deserialize, Debug)]
pub(crate) struct Options {
    pub(crate) format: OutFormat,
    pub(crate) results: i32,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            format: OutFormat::Json,
            results: 10,
        }
    }
}

pub(crate) fn merge(a: OptionsOpt) -> Options {
    let mut out = Options::default();
    out.merge_opt(a);
    out
}
