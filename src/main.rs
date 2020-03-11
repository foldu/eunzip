use crate::mmap::{FileMmap, SharedFileMmap};
use encoding::{label::encoding_from_whatwg_label, DecoderTrap, EncodingRef};
use rayon::prelude::*;
use std::fs::{self, File};
use std::io::{self, BufWriter};
use std::path::{Path, PathBuf};
use structopt::StructOpt;
use zip::{read::ZipFile, ZipArchive};

mod mmap;

// TODO: handle SIGBUS

fn run() -> Result<(), anyhow::Error> {
    match Opt::from_args() {
        Opt::Try { zips } => traverse_all_files_in_zips(zips.iter(), |file| {
            try_all_encodings(file.name_raw());
            Ok(())
        }),

        Opt::Unzip { from, output, zips } => {
            for path in zips {
                let archive = open_zip(path)?;

                (0..archive.len())
                    .into_par_iter()
                    .map(|i| {
                        let mut archive = archive.clone();
                        let mut file = archive.by_index(i)?;
                        let decoded = decode_zip_filename(from, &file)?;
                        extract_zip_entry_to(&output, &decoded.into(), &mut file)
                    })
                    .collect::<Result<_, anyhow::Error>>()?;
            }
            Ok(())
        }

        Opt::PrintAll => {
            print_all_encodings();
            Ok(())
        }

        Opt::List { from, zips } => traverse_all_files_in_zips(zips.iter(), |file| {
            let decoded = decode_zip_filename(from, &file)?;
            println!("{}", decoded);
            Ok(())
        }),
    }
}

fn traverse_all_files_in_zips<P, I, F>(paths: I, f: F) -> Result<(), anyhow::Error>
where
    P: AsRef<Path>,
    I: IntoIterator<Item = P>,
    F: Fn(&mut ZipFile) -> Result<(), anyhow::Error>,
{
    traverse_zip_archives_with(paths, |a| {
        traverse_zip_files_with(a, |mut file| f(&mut file))
    })
}

fn traverse_zip_files_with<F>(archive: &mut ZipFileReader, f: F) -> Result<(), anyhow::Error>
where
    F: Fn(&mut ZipFile) -> Result<(), anyhow::Error>,
{
    (0..archive.len())
        .map(|i| {
            let mut file = archive.by_index(i)?;
            f(&mut file)
        })
        .collect()
}

fn traverse_zip_archives_with<I, P, F>(paths: I, f: F) -> Result<(), anyhow::Error>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
    F: Fn(&mut ZipFileReader) -> Result<(), anyhow::Error>,
{
    paths
        .into_iter()
        .map(|path| open_zip(path).and_then(|mut z| f(&mut z)))
        .collect()
}

type ZipFileReader = ZipArchive<io::Cursor<SharedFileMmap>>;

fn open_zip<P>(path: P) -> Result<ZipFileReader, anyhow::Error>
where
    P: AsRef<Path>,
{
    let path = path.as_ref();
    let mmap =
        FileMmap::open(&path).map_err(|e| anyhow::format_err!("Can't open {:?}: {}", path, e))?;
    let cur = io::Cursor::new(mmap.make_shared());

    ZipArchive::new(cur).map_err(|e| anyhow::format_err!("Can't open {:?} as zip: {}", path, e))
}

#[derive(Debug, Clone)]
struct SanitizedZipPath {
    cont: String,
    is_dir: bool,
}

impl SanitizedZipPath {
    fn is_dir(&self) -> bool {
        self.is_dir
    }
}

impl From<String> for SanitizedZipPath {
    fn from(mut s: String) -> Self {
        // mostly stolen from https://github.com/mvdnes/zip-rs/blob/master/src/types.rs
        let is_dir = s.ends_with('/');

        if let Some(index) = s.find('\0') {
            s.truncate(index);
        }

        // zip files can contain both / and \ as separators regardless of the OS
        // and as we want to return a sanitized PathBuf that only supports the
        // OS separator let's convert incompatible separators to compatible ones
        let separator = ::std::path::MAIN_SEPARATOR;
        let opposite_separator = match separator {
            '/' => '\\',
            '\\' | _ => '/',
        };
        let filename = s.replace(&opposite_separator.to_string(), &separator.to_string());

        let ret = Path::new(&filename)
            .components()
            .filter(|component| match *component {
                ::std::path::Component::Normal(..) => true,
                _ => false,
            })
            .fold(PathBuf::new(), |mut path, ref cur| {
                path.push(cur.as_os_str());
                path
            });

        SanitizedZipPath {
            cont: ret.to_str().unwrap().to_string(),
            is_dir,
        }
    }
}

pub fn try_all_encodings(buf: &[u8]) {
    for enc in encoding::all::encodings() {
        if enc.whatwg_name().is_some() {
            if let Ok(s) = enc.decode(buf, DecoderTrap::Strict) {
                println!("{}: {}", enc.whatwg_name().unwrap_or_else(|| enc.name()), s);
            }
        }
    }
}

pub fn print_all_encodings() {
    for enc in encoding::all::encodings() {
        if let Some(name) = enc.whatwg_name() {
            println!("{}", name);
        }
    }
}

impl AsRef<str> for SanitizedZipPath {
    fn as_ref(&self) -> &str {
        &self.cont
    }
}

fn extract_zip_entry_to<P>(
    root: P,
    decoded: &SanitizedZipPath,
    mut zip: &mut ZipFile,
) -> Result<(), anyhow::Error>
where
    P: AsRef<Path>,
{
    let dest = root.as_ref().join(decoded.as_ref());
    if decoded.is_dir() {
        if !dest.is_dir() {
            fs::create_dir_all(&dest)?;
        }
    } else {
        if let Some(p) = dest.parent() {
            if !p.is_dir() {
                fs::create_dir_all(p)?;
            }
        }

        let mut out = File::create(&dest)
            .map(BufWriter::new)
            .map_err(|e| anyhow::format_err!("Can't create output file {:?}: {}", &dest, e))?;

        io::copy(&mut zip, &mut out)?;
    }
    println!("{:?}", dest);

    Ok(())
}

fn decode_zip_filename(enc: EncodingRef, file: &ZipFile) -> Result<String, anyhow::Error> {
    enc.decode(file.name_raw(), DecoderTrap::Strict)
        .map_err(|e| {
            anyhow::format_err!(
                "Encoding {} doesn't work for file {}: {}",
                enc.whatwg_name().unwrap(),
                file.name(),
                e
            )
        })
}

fn parse_encoding(s: &str) -> Result<EncodingRef, anyhow::Error> {
    encoding_from_whatwg_label(&s).ok_or_else(|| anyhow::format_err!("Unknown encoding: {}", s))
}

#[derive(StructOpt)]
enum Opt {
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

fn main() {
    if let Err(e) = run() {
        eprintln!("{}", e);
        ::std::process::exit(1);
    }
}
