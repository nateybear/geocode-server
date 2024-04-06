use optfield::optfield;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub(crate) enum OutFormat {
    Json,
    Csv,
}

impl OutFormat {
    pub(crate) fn print<T: serde::Serialize>(
        &self,
        out: Vec<T>,
    ) -> Result<String, crate::AppError> {
        match self {
            OutFormat::Json => Ok(serde_json::to_string(&out)?),
            OutFormat::Csv => {
                let mut wtr = csv::Writer::from_writer(vec![]);
                for a in out {
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
