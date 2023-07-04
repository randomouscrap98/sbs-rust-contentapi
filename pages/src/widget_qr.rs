use std::io::Write;

use common::*;
use common::prefab::get_fullpage_by_hash;
use common::render::layout::*;
use common::response::*;
use flate2::write::ZlibEncoder;
use maud::*;
use qrcode::QrCode;
use qrcode::render::svg;
use qrcode::types::QrError;
use serde::{Serialize, Deserialize};

use base64::{Engine as _, engine::general_purpose};

//use bbscope::BBCode;


// This widget is special: i'm worried about the memory usage, so I'm ensuring everything is 
// done in each loop iteration rather than precomputing everything. Or at least, I'm setting it
// up so it can be done that way (I'm not actually, but that's the excuse I'm using for why
// there's no "render()" like usual)

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct PtcData {
    pub base64: String,
    pub name: String,
    pub description: Option<String>
}

pub async fn get_render(mut context: PageContext, hash: &str, high_density: bool) -> Result<Response, Error>
{
    //First, go lookup the page
    let page = get_fullpage_by_hash(&mut context.api_context, hash).await?;
    let qrlink = context.layout_data.links.qr_generator(&page.main);

    Ok(Response::Render(
        //Eventually, this'll be a real widget. Until then, render normal page
        //basic_skeleton(&context.layout_data, html! {
        //    title { (opt_s!(page.main.name)) " QR Codes" }
        //    meta name="description" content="SmileBASIC Source QR code generator (for Petit Computer)";
        //    (context.layout_data.links.style("/forpage/qrwidget.css"))
        //}, html! {
        layout(&context.layout_data, html!{
            (context.layout_data.links.style("/forpage/qrwidget.css"))
            (context.layout_data.links.script("/forpage/qrwidget.js"))
            section {
                h1 { a."flatlink" href=(context.layout_data.links.forum_thread(&page.main)) { (opt_s!(page.main.name)) } }
                div."controls mediumseparate" {
                    @if high_density {
                        a href=(qrlink) { "Normal density" }
                        span { "High density (current)"}
                    }
                    @else {
                        span { "Normal density (current)"}
                        a href={(qrlink)"?high_density=true"} { "High density" }
                    }
                }
                @if let Some(ptc_files) = page.ptc {
                    @if let Some(ptc_data) = ptc_files.text {
                        @let parsed_data = serde_json::de::from_str::<Vec<PtcData>>(&ptc_data)?;
                        @for ptc_file in parsed_data {
                            hr;
                            h3 { (ptc_file.name) }
                            @if let Some(ref description) = ptc_file.description {
                                p { (description)}
                            }
                            @let qr_codes = generate_qr_svgs(ptc_file, if high_density { QrConfig::high_density() } else { QrConfig::default() })?; 
                            div."qrcodes" {
                                @for (i, qr) in qr_codes.iter().enumerate()
                                {
                                    div."qr" {
                                        (PreEscaped(qr))
                                        div."tracking" {
                                            span { ({i + 1}) } " / " span { (qr_codes.len())}
                                        }
                                    }
                                }
                            }
                        }
                    }
                    @else {
                        p."error" { "Something went seriously wrong! No text in ptc content!" }
                    }
                }
                @else {
                    p."error" { "This page doesn't have any petit computer files!!" }
                }
            }
        }).into_string()))
}

pub struct QrConfig {
    pub bytes_per_qr : i32,
    pub qr_version : i16,
    pub error_level : qrcode::EcLevel,
    pub min_size : u32,
    pub dark_color: String,
    pub light_color: String
}

impl Default for QrConfig {
    fn default() -> Self {
        Self { 
            bytes_per_qr: 630,  //Doc says 630
            qr_version : 20,    //Doc says 20 
            error_level: qrcode::EcLevel::M,
            min_size: 200,
            dark_color: String::from("#000000"),
            light_color: String::from("#ffffff")
        }
    }
}

impl QrConfig {
    pub fn high_density() -> Self {
        Self {
            bytes_per_qr: 1237, //'spec' says 1273 (minus 36 = 1237)
            qr_version : 25, //This the max from PTCUtilities
            error_level: qrcode::EcLevel::L,
            min_size: 250,
            dark_color: String::from("#000000"),
            light_color: String::from("#ffffff")
        }
    }
}

pub fn generate_qr_svgs(ptc_file: PtcData, config : QrConfig) -> Result<Vec<String>, Error>
{
    let raw = general_purpose::STANDARD.decode(&ptc_file.base64).map_err(|e| Error::Other(e.to_string()))?;
    let rawlength = raw.len() as u32;
    let ftype = &raw[8..12]; //The 4 char code that describes the type
    println!("raw length: {}\nftype: {}", rawlength, std::str::from_utf8(ftype).unwrap());

    let mut enc = ZlibEncoder::new(Vec::new(), flate2::Compression::best());
    enc.write_all(&raw).map_err(|e| Error::Other(e.to_string()))?;
    let zlibdata = enc.finish().map_err(|e| Error::Other(e.to_string()))?;

    let mut result : Vec<u8> = Vec::new();
    result.extend_from_slice(ptc_file.name.as_bytes());
    //slow but ugh i'm tired. this pads the name
    while result.len() < 8 { result.push(0); }
    result.extend_from_slice(ftype);
    result.extend((zlibdata.len() as u32).to_le_bytes());
    result.extend(rawlength.to_le_bytes());
    result.extend(zlibdata);

    let resultmd5 : [u8;16] = md5::compute(&result).into();
    let qrcount = (result.len() as f32 / config.bytes_per_qr as f32).ceil() as u8;
    println!("QR codes: {}", qrcount);

    let mut qrcodes : Vec<String> = Vec::new();
    for qrnum in 0u8..qrcount 
    {
        let start = (config.bytes_per_qr * qrnum as i32) as usize;
        let end = std::cmp::min((start + config.bytes_per_qr as usize) as usize, result.len());
        let resultslice = &result[start..end];
        let slicemd5 : [u8;16] = md5::compute(resultslice).into();

        let mut qrdata : Vec<u8> = vec![0x50u8, 0x54u8, qrnum + 1, qrcount];
        qrdata.extend(slicemd5);
        qrdata.extend_from_slice(&resultmd5);
        qrdata.extend_from_slice(resultslice);
        //println!("QR {} size: {}", qrnum + 1, qrdata.len());

        let code = get_qr_code(&qrdata, &config)
            .map_err(|e| Error::Other(e.to_string()))?;
        let image = code.render()
            .min_dimensions(config.min_size, config.min_size)
            .dark_color(svg::Color(&config.dark_color))
            .light_color(svg::Color(&config.light_color))
            .build();

        qrcodes.push(image);
    }
    Ok(qrcodes)
}

/// Retrieve a 'qrcode::QrCode' object for the given qrdata. Apparently this takes some setup
/// because the library can't automatically do it...
pub fn get_qr_code(qrdata: &[u8], config: &QrConfig) -> Result<QrCode, QrError>
{
    let mut qrbits = qrcode::bits::Bits::new(qrcode::Version::Normal(config.qr_version));
    qrbits.push_byte_data(&qrdata)?;
    qrbits.push_terminator(config.error_level)?;
    let code = QrCode::with_bits(qrbits, config.error_level)?;
    Ok(code)
}