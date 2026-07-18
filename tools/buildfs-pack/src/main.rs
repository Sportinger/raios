use buildfs_pack::pack_directory;
use raios_core::parse_sha256_ref;
use std::env;
use std::ffi::OsString;
use std::path::PathBuf;
use std::process::ExitCode;

const USAGE: &str =
    "usage: buildfs-pack <input-dir> --output <out-dir> [--expect-manifest-sha256 <hex>]";

struct Arguments {
    input: PathBuf,
    output: PathBuf,
    expected_manifest_sha256: Option<[u8; 32]>,
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("buildfs-pack: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let arguments = parse_arguments(env::args_os().skip(1))?;
    pack_directory(
        &arguments.input,
        &arguments.output,
        arguments.expected_manifest_sha256,
    )
    .map(|_| ())
    .map_err(|error| error.to_string())
}

fn parse_arguments(arguments: impl IntoIterator<Item = OsString>) -> Result<Arguments, String> {
    let mut arguments = arguments.into_iter();
    let input = arguments
        .next()
        .filter(|argument| argument != "--output" && argument != "--expect-manifest-sha256")
        .map(PathBuf::from)
        .ok_or_else(|| USAGE.to_owned())?;
    let mut output = None;
    let mut expected_manifest_sha256 = None;

    while let Some(argument) = arguments.next() {
        if argument == "--output" {
            if output.is_some() {
                return Err("--output may be specified only once".to_owned());
            }
            output = Some(PathBuf::from(
                arguments
                    .next()
                    .ok_or_else(|| "--output requires a value".to_owned())?,
            ));
        } else if argument == "--expect-manifest-sha256" {
            if expected_manifest_sha256.is_some() {
                return Err("--expect-manifest-sha256 may be specified only once".to_owned());
            }
            let value = arguments
                .next()
                .ok_or_else(|| "--expect-manifest-sha256 requires a value".to_owned())?
                .into_string()
                .map_err(|_| "--expect-manifest-sha256 must be valid UTF-8".to_owned())?;
            if value.len() != 64 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
                return Err(
                    "--expect-manifest-sha256 must be exactly 64 hexadecimal digits".to_owned(),
                );
            }
            expected_manifest_sha256 = Some(parse_sha256_ref(&value).ok_or_else(|| {
                "--expect-manifest-sha256 must be exactly 64 hexadecimal digits".to_owned()
            })?);
        } else {
            return Err(format!("unknown argument {argument:?}; {USAGE}"));
        }
    }

    Ok(Arguments {
        input,
        output: output.ok_or_else(|| format!("--output is required; {USAGE}"))?,
        expected_manifest_sha256,
    })
}
