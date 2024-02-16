use std::{env, path::PathBuf};

const ZXING_CPP_SRC: &[&str] = &[
    "BarcodeFormat.cpp",
    "BitArray.cpp",
    "BitMatrix.cpp",
    "BitMatrixIO.cpp",
    "ConcentricFinder.cpp",
    "ECI.cpp",
    "GenericGF.cpp",
    "GenericGFPoly.cpp",
    "GTIN.cpp",
    "Utf.cpp",
    "ZXBigInteger.cpp",
    "BinaryBitmap.cpp",
    "BitSource.cpp",
    "Content.cpp",
    "GlobalHistogramBinarizer.cpp",
    "GridSampler.cpp",
    "HRI.cpp",
    "HybridBinarizer.cpp",
    "MultiFormatReader.cpp",
    "PerspectiveTransform.cpp",
    "ReadBarcode.cpp",
    "ReedSolomonDecoder.cpp",
    "Result.cpp",
    "ResultPoint.cpp",
    "TextDecoder.cpp",
    "WhiteRectDetector.cpp",
    "ZXingC.cpp",

    "aztec/AZDecoder.cpp",
    "aztec/AZDetector.cpp",
    "aztec/AZReader.cpp",
    
    "datamatrix/DMBitLayout.cpp",
    "datamatrix/DMVersion.cpp",
    "datamatrix/DMDataBlock.cpp",
    "datamatrix/DMDecoder.cpp",
    "datamatrix/DMDetector.cpp",
    "datamatrix/DMReader.cpp",
    
    "maxicode/MCBitMatrixParser.cpp",
    "maxicode/MCDecoder.cpp",
    "maxicode/MCReader.cpp",

    "oned/ODUPCEANCommon.cpp",
    "oned/ODCode128Patterns.cpp",
    "oned/ODCodabarReader.cpp",
    "oned/ODCode39Reader.cpp",
    "oned/ODCode93Reader.cpp",
    "oned/ODCode128Reader.cpp",
    "oned/ODDataBarCommon.cpp",
    "oned/ODDataBarReader.cpp",
    "oned/ODDataBarExpandedBitDecoder.cpp",
    "oned/ODDataBarExpandedReader.cpp",
    "oned/ODDXFilmEdgeReader.cpp",
    "oned/ODITFReader.cpp",
    "oned/ODMultiUPCEANReader.cpp",
    "oned/ODReader.cpp",

    "pdf417/PDFBarcodeValue.cpp",
    "pdf417/PDFBoundingBox.cpp",
    "pdf417/PDFCodewordDecoder.cpp",
    "pdf417/PDFDecoder.cpp",
    "pdf417/PDFDetectionResult.cpp",
    "pdf417/PDFDetectionResultColumn.cpp",
    "pdf417/PDFDetector.cpp",
    "pdf417/PDFModulusGF.cpp",
    "pdf417/PDFModulusPoly.cpp",
    "pdf417/PDFReader.cpp",
    "pdf417/PDFScanningDecoder.cpp",

    "qrcode/QRCodecMode.cpp",
    "qrcode/QRErrorCorrectionLevel.cpp",
    "qrcode/QRVersion.cpp",
    "qrcode/QRBitMatrixParser.cpp",
    "qrcode/QRDataBlock.cpp",
    "qrcode/QRDecoder.cpp",
    "qrcode/QRDetector.cpp",
    "qrcode/QRFormatInformation.cpp",
    "qrcode/QRReader.cpp",
    "libzueci/zueci.c",
];

fn base_build() -> cc::Build {
    let mut build = cc::Build::new();

    build
        .warnings(false)
        .extra_warnings(false);

    build
}

fn is_msvc() -> bool {
    env::var("TARGET").unwrap().ends_with("-msvc")
}

fn build_xzing_cpp() {
    let root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let cpp_flags: &[&str] = if is_msvc() {
        &["/std:c++20", "/EHsc"]
    } else {
        &["-std=c++20"]
    };

    let mut build = base_build();
    for flag in cpp_flags {
        build.flag(flag);
    }

    build
        .cpp(true)
        .include(root.join("zxing-cpp").join("core").join("src"))
        .define("NDEBUG", "true")
        .files(
            ZXING_CPP_SRC
                .iter()
                .map(|f| root.join("zxing-cpp").join("core").join("src").join(f))
                .collect::<Vec<_>>(),
        )
        .compile("zxing-cpp");
}
fn main() {
    // build_bindings();
    build_xzing_cpp();
}
