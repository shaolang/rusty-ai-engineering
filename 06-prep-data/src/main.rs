use std::path::PathBuf;

use futures_util::StreamExt;
use tokio::io::AsyncWriteExt;
use transmutation::{Converter, OutputFormat};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let args: Args = argh::from_env();

    let fpath = download_file(&args.src).await?;
    convert_to_markdown(fpath.to_str().unwrap(), &args.output).await?;
    println!("Converted {} to {}", args.src, args.output);
    Ok(())
}

type Error = Box<dyn std::error::Error>;

async fn convert_to_markdown(src: &str, dest: &str) -> Result<(), Error> {
    let converter = Converter::new()?;
    let output_format = OutputFormat::Markdown {
        split_pages: true,
        optimize_for_llm: true,
    };
    let result = converter.convert(src).to(output_format).execute().await?;

    result.save(dest).await?;
    Ok(())
}

async fn download_file(src_url: &str) -> Result<PathBuf, Error> {
    let fpath = {
        let temp = temp_file::TempFile::new()?;
        temp.path().to_owned()
    };

    let mut file = tokio::fs::File::create(&fpath).await?;

    let mut stream = reqwest::get(src_url).await?.bytes_stream();
    while let Some(chunk) = stream.next().await {
        if let Ok(raw) = chunk {
            file.write_all(&raw).await?;
        }
    }

    Ok(fpath)
}

/// Document converting demo; currently only supports convert PDF files to Markdown
#[derive(Debug, argh::FromArgs)]
struct Args {
    /// source file
    #[argh(positional)]
    src: String,

    /// output file
    #[argh(positional)]
    output: String,
}
