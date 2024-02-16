use std::time::Instant;

use zxing_rs_no_cmake::*;

fn main() -> anyhow::Result<()> {
    let now: Instant = Instant::now();

    let filename = std::env::args()
        .nth(1)
        .expect("no image file name provided");
    let formats = std::env::args().nth(2);

    let image = image::open(&filename)?;

    #[cfg(not(feature = "image"))]
    let lum_img = image.into_luma8();
    #[cfg(not(feature = "image"))]
    let iv = ImageView::from_slice(
        &lum_img,
        lum_img.width(),
        lum_img.height(),
        ImageFormat::Lum,
    )?;

    let formats = barcode_formats_from_string(formats.unwrap_or_default())?;
    let opts = ReaderOptions::default()
        .formats(formats)
        .try_harder(true)
        .try_invert(false)
        .try_rotate(false)
        .try_downscale(false);

    #[cfg(feature = "image")]
    let barcodes = read_barcodes(&image, &opts)?;
    #[cfg(not(feature = "image"))]
    let barcodes = read_barcodes(iv, opts)?;

    if barcodes.is_empty() {
        println!("No barcode found.");
    } else {
        for barcode in barcodes {
            println!("Text:       {}", barcode.text());
            println!("Bytes:      {:?}", barcode.bytes());
            println!("Format:     {}", barcode.format());
            println!("Content:    {}", barcode.content_type());
            println!("Identifier: {}", barcode.symbology_identifier());
            println!("EC Level:   {}", barcode.ec_level());
            println!("Error:      {}", barcode.error_message());
            println!("Rotation:   {}", barcode.orientation());
            println!("Position:   {}", barcode.position());
            println!();
        }
    }

    println!("running zxing-cpp took: {}ms", now.elapsed().as_millis());

    Ok(())
}
