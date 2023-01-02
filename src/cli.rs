use clap::Parser;
use encoding::{label::encoding_from_whatwg_label, EncodingRef};
use std::path::PathBuf;

#[derive(Parser)]
pub enum Opt {
    #[clap(name = "print-encodings", visible_alias = "p")]
    /// Print all available encodings
    PrintAll,

    #[clap(name = "test", visible_alias = "t")]
    /// Just try all encodings on every single file of the zip and print the working ones
    Try {
        /// Your zips
        zips: Vec<PathBuf>,
    },

    #[clap(name = "unzip", visible_alias = "x")]
    /// Unzip mode
    Unzip {
        #[clap(
            short,
            long,
            value_parser = parse_encoding,
            default_value = "utf-8"
        )]
        /// Encoding of file names in zip file
        from: EncodingRef,

        #[clap(short, long, default_value = ".")]
        /// Output dir for extracted files
        output: PathBuf,

        /// The zips you want to extract
        zips: Vec<PathBuf>,
    },

    #[clap(visible_alias = "l")]
    /// List all files in zip
    List {
        /// Encoding of file names in zip file
        #[clap(
            short,
            long,
            value_parser = parse_encoding,
            default_value = "utf-8"
        )]
        from: EncodingRef,

        /// Zips
        zips: Vec<PathBuf>,
    },
}

fn parse_encoding(s: &str) -> Result<EncodingRef, anyhow::Error> {
    encoding_from_whatwg_label(s).ok_or_else(|| anyhow::format_err!("Unknown encoding: {}", s))
}
