use std::fs;
use std::path::Path;

use clap::Parser;
use lodepng::FilterStrategy;
use rgb::ComponentBytes;
use sha2::{Digest, Sha256};

#[derive(Parser, Debug)]
#[command(arg_required_else_help(true))]
struct Cli {
    input_file_path: String,
    #[arg(short, long)]
    output_dir: Option<String>,
}

#[derive(Debug)]
struct ImageWithHashes {
    /// hash of the png data, so not the file but the rgb(a) values
    pub src_data_hash: Box<[u8]>,
    /// the png as would be present on the minecraft servers
    pub minecraft_png: Box<[u8]>,
    /// the hash of the png as would be present on the minecraft servers
    pub minecraft_hash: Box<[u8]>,
}

fn main() {
    let args = Cli::parse();

    let input_content =
        fs::read(args.input_file_path)
            .expect("failed to read image from given location!");

    let decoded = decode_image(input_content.as_slice());
    let src_png_hash = sha256_of(input_content.as_slice());
    let result = encode_image_minecraft(decoded.0.as_slice(), decoded.1, decoded.2);

    println!("minecraft hash: {:}", write_hex(result.minecraft_hash.as_ref()));
    println!("image file hash: {:}", write_hex(src_png_hash.as_ref()));
    println!("image data hash: {:}", write_hex(result.src_data_hash.as_ref()));

    if let Some(output_dir) = args.output_dir {
        let file_name = write_hex(result.minecraft_hash.as_ref());

        let output_file = Path::new(&output_dir).join(format!("converted-{:}.png", file_name));
        fs::write(&output_file, result.minecraft_png.as_ref())
            .expect("could not write image to given file");

        println!("minecraft image has been exported to {:}", output_file.to_str().unwrap());
    }
}

fn decode_image(bytes: &[u8]) -> (Vec<u8>, usize, usize) {
    let png = lodepng::decode32(bytes).expect("failed to decode image");
    (Vec::from(png.buffer.as_bytes()), png.width, png.height)
}

fn encode_image_minecraft(src_data: &[u8], width: usize, height: usize) -> ImageWithHashes {
    // encode images like Minecraft does
    let mut encoder = lodepng::Encoder::new();
    encoder.set_auto_convert(false);
    encoder.info_png_mut().interlace_method = 0; // should be 0 but just to be sure

    let mut encoder_settings = encoder.settings_mut();
    encoder_settings.zlibsettings.set_level(4);
    encoder_settings.filter_strategy = FilterStrategy::ZERO;

    let minecraft_png = encoder.encode(src_data, width, height).unwrap();

    let mut hasher = Sha256::new();

    hasher.update(&minecraft_png);
    let minecraft_hash = hasher.finalize_reset();

    hasher.update(src_data);
    let src_data_hash = hasher.finalize();

    ImageWithHashes {
        src_data_hash: Box::from(src_data_hash.as_slice()),
        minecraft_png: Box::from(minecraft_png.as_slice()),
        minecraft_hash: Box::from(minecraft_hash.as_slice()),
    }
}

fn sha256_of(bytes: &[u8]) -> Box<[u8]> {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let hash = hasher.finalize();
    Box::from(hash.as_slice())
}

fn write_hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(2 * bytes.len());
    for byte in bytes {
        core::fmt::write(&mut s, format_args!("{:02X}", byte)).unwrap();
    }
    s
}
