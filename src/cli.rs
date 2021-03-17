use encoding::{label::encoding_from_whatwg_label, EncodingRef};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt)]
pub enum Opt {
    #[structopt(name = "print-encodings", visible_alias = "p")]
    /// Print all available encodings
    PrintAll,

    #[structopt(name = "test", visible_alias = "t")]
    /// Just try all encodings on every single file of the zip and print the working ones
    Try {
        /// Your zips
        #[structopt(parse(from_os_str))]
        zips: Vec<PathBuf>,
    },

    #[structopt(name = "unzip", visible_alias = "x")]
    /// Unzip mode
    Unzip {
        #[structopt(
            short,
            long,
            parse(try_from_str = parse_encoding),
            default_value = "utf-8"
        )]
        /// Encoding of file names in zip file
        from: EncodingRef,

        #[structopt(short, long, parse(from_os_str), default_value = ".")]
        /// Output dir for extracted files
        output: PathBuf,

        #[structopt(parse(from_os_str))]
        /// The zips you want to extract
        zips: Vec<PathBuf>,
    },

    #[structopt(visible_alias = "l")]
    /// List all files in zip
    List {
        /// Encoding of file names in zip file
        #[structopt(
            short = "f",
            long = "from",
            parse(try_from_str = parse_encoding),
            default_value = "utf-8"
        )]
        from: EncodingRef,

        #[structopt(parse(from_os_str))]
        /// Zips
        zips: Vec<PathBuf>,
    },
}

fn parse_encoding(s: &str) -> Result<EncodingRef, anyhow::Error> {
    encoding_from_whatwg_label(&s).ok_or_else(|| anyhow::format_err!("Unknown encoding: {}", s))
}
