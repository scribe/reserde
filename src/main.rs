use std::io::Read;
use {
    argh::FromArgs,
    serde_detach::detach,
    serde_object::Object,
    std::{
        fs::File,
        io::{stdin, stdout, Write as _},
        path::PathBuf,
    },
    strum::EnumString,
    wyz::Pipe as _,
};

#[derive(Debug, FromArgs)]
/// Transcode one self-describing format into another.
///
/// Currently supports CBOR, JSON (--pretty), TAML (--in only), XML, x-www-form-urlencoded (as urlencoded) and YAML.
/// All names are lowercase.
struct Args {
    #[argh(option, long = "if")]
    /// where to read input from. Defaults to stdin
    in_file: Option<PathBuf>,

    #[argh(option, long = "of")]
    /// where to write output to. Defaults to stdout
    out_file: Option<PathBuf>,

    //TODO: List In variant.
    #[argh(option, short = 'i', long = "in")]
    /// what to read
    in_format: In,

    //TODO: List Out variant.
    #[argh(option, short = 'o', long = "out")]
    /// what to write
    out_format: Out,

    #[argh(switch, short = 'p')]
    /// pretty-print (where supported)
    pretty: bool,
}

#[derive(Debug, EnumString)]
enum In {
    #[cfg(feature = "de-cbor")]
    #[strum(serialize = "cbor")]
    Cbor,

    #[cfg(feature = "de-json")]
    #[strum(serialize = "json")]
    Json,

    #[cfg(feature = "de-taml")]
    #[strum(serialize = "taml")]
    Taml,

    #[cfg(feature = "de-urlencoded")]
    #[strum(serialize = "urlencoded")]
    Urlencoded,

    #[cfg(feature = "de-xml")]
    #[strum(serialize = "xml")]
    Xml,

    #[cfg(feature = "de-yaml")]
    #[strum(serialize = "yaml")]
    Yaml,
}

#[derive(Debug, EnumString)]
enum Out {
    #[cfg(feature = "ser-cbor")]
    #[strum(serialize = "cbor")]
    Cbor,

    #[cfg(feature = "ser-json")]
    #[strum(serialize = "json")]
    Json,

    #[cfg(feature = "ser-urlencoded")]
    #[strum(serialize = "urlencoded")]
    Urlencoded,

    #[cfg(feature = "ser-xml")]
    #[strum(serialize = "xml")]
    Xml,

    #[cfg(feature = "ser-yaml")]
    #[strum(serialize = "yaml")]
    Yaml,
}

fn main() {
    let args: Args = argh::from_env();

    //TODO: Avoid leaking.

    let object: Object = match args.in_format {
        #[cfg(feature = "de-cbor")]
        In::Cbor => if let Some(path) = args.in_file {
            File::open(path)
                .unwrap()
                .pipe(serde_cbor::from_reader)
                .map(detach)
        } else {
            stdin().pipe(serde_cbor::from_reader).map(detach)
        }
        .unwrap(),

        #[cfg(feature = "de-json")]
        In::Json => {
            let mut text = String::new();
            if let Some(path) = args.in_file {
                File::open(path).unwrap().read_to_string(&mut text).unwrap();
            } else {
                stdin().read_to_string(&mut text).unwrap();
            }
            serde_json::from_str(&text).map(detach).unwrap()
        }

        #[cfg(feature = "de-taml")]
        In::Taml => {
            let diagnostics = vec![];
            let diagnostics = Box::new(diagnostics);
            let diagnostics = Box::leak(diagnostics);
            let mut text = String::new();
            if let Some(path) = args.in_file {
                File::open(path).unwrap().read_to_string(&mut text).unwrap();
            } else {
                stdin().read_to_string(&mut text).unwrap();
            }
            taml::deserializer::from_str(&text, diagnostics)
                .map(detach)
                .unwrap()
        }

        #[cfg(feature = "de-urlencoded")]
        In::Urlencoded => if let Some(path) = args.in_file {
            File::open(path)
                .unwrap()
                .pipe(serde_urlencoded::from_reader)
                .map(detach)
        } else {
            stdin().pipe(serde_urlencoded::from_reader).map(detach)
        }
        .unwrap(),

        #[cfg(feature = "de-xml")]
        In::Xml => {
            let mut text = String::new();
            if let Some(path) = args.in_file {
                File::open(path).unwrap().read_to_string(&mut text).unwrap();
            } else {
                stdin().read_to_string(&mut text).unwrap();
            }
            quick_xml::de::from_str(&text).map(detach).unwrap()
        }

        #[cfg(feature = "de-yaml")]
        In::Yaml => {
            let mut text = String::new();
            if let Some(path) = args.in_file {
                File::open(path).unwrap().read_to_string(&mut text).unwrap();
            } else {
                stdin().read_to_string(&mut text).unwrap();
            }
            serde_yaml::from_str(&text).map(detach).unwrap()
        }
    };

    let pretty = args.pretty;
    match args.out_format {
        #[cfg(feature = "ser-cbor")]
        Out::Cbor => {
            if let Some(path) = args.out_file {
                serde_cbor::to_writer(File::create(path).unwrap(), &object).unwrap()
            } else {
                serde_cbor::to_writer(stdout(), &object).unwrap()
            }
        }

        #[cfg(feature = "ser-json")]
        Out::Json => {
            if let Some(path) = args.out_file {
                let file = File::create(path).unwrap();
                if pretty {
                    serde_json::to_writer_pretty(file, &object).unwrap()
                } else {
                    serde_json::to_writer(file, &object).unwrap()
                }
            } else if pretty {
                serde_json::to_writer_pretty(stdout(), &object).unwrap()
            } else {
                serde_json::to_writer(stdout(), &object).unwrap()
            }
        }

        #[cfg(feature = "ser-urlencoded")]
        Out::Urlencoded => {
            let text = serde_urlencoded::to_string(&object).unwrap();

            if let Some(path) = args.out_file {
                write!(File::create(path).unwrap(), "{}", text).unwrap();
            } else {
                print!("{}", text)
            }
        }

        #[cfg(feature = "ser-xml")]
        Out::Xml => {
            if let Some(path) = args.out_file {
                let file = File::create(path).unwrap();
                quick_xml::se::to_writer(file, &object).unwrap()
            } else {
                quick_xml::se::to_writer(stdout(), &object).unwrap()
            }
        }

        #[cfg(feature = "ser-yaml")]
        Out::Yaml => {
            if let Some(path) = args.out_file {
                let file = File::create(path).unwrap();
                serde_yaml::to_writer(file, &object).unwrap()
            } else {
                serde_yaml::to_writer(stdout(), &object).unwrap()
            }
        }
    };

    stdout().flush().unwrap()
}
