use clap::{ArgGroup, Parser};

use crate::filter::{FsMessageFilter, FsMessageFilterMode};

#[derive(Parser, Debug)]
#[command(author, version, about)]
#[command(group(
    ArgGroup::new("filter_mode")
        .args(["filter_optout", "filter_optin"])
        .required(false)
        .multiple(false)
))]
pub struct Cli {
    #[arg(long)]
    pub path: String,

    #[arg(long, value_delimiter = ',')]
    pub filter_optout: Option<Vec<String>>,

    #[arg(long, value_delimiter = ',')]
    pub filter_optin: Option<Vec<String>>,
}

pub fn build_filter(
    filter_optout: Option<Vec<String>>,
    filter_optin: Option<Vec<String>>,
) -> Result<Option<FsMessageFilter>, globset::Error> {
    if let Some(filter_optout) = filter_optout {
        let filter = FsMessageFilter::create(FsMessageFilterMode::OptOut, filter_optout)?;

        return Ok(Some(filter));
    }

    if let Some(filter_optin) = filter_optin {
        let filter = FsMessageFilter::create(FsMessageFilterMode::OptIn, filter_optin)?;

        return Ok(Some(filter));
    }

    Ok(None)
}
