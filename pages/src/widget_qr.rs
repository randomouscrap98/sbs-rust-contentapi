
//use bbscope::BBCode;

use std::io::Write;

//use base64::alphabet;
//use base64::engine::{general_purpose, self};
use common::*;
use common::prefab::{get_fullpage_by_hash};
use common::render::layout::*;
use flate2::write::ZlibEncoder;
use maud::*;
use serde::{Serialize, Deserialize};

use base64::{Engine as _, engine::general_purpose};


// This widget is special: i'm worried about the memory usage, so I'm ensuring everything is 
// done in each loop iteration rather than precomputing everything.

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct PtcData {
    pub base64: String,
    pub name: String,
    pub description: Option<String>
}

pub async fn get_render(mut context: PageContext, hash: &str) -> Result<Response, Error>
{
    //First, go lookup the page
    let page = get_fullpage_by_hash(&mut context.api_context, hash).await?;

    Ok(Response::Render(
        //Eventually, this'll be a real widget. Until then, render normal page
        //basic_skeleton(&context.layout_data, html! {
        //    title { (opt_s!(page.main.name)) " QR Codes" }
        //    meta name="description" content="SmileBASIC Source QR code generator (for Petit Computer)";
        //    (context.layout_data.links.style("/forpage/qrwidget.css"))
        //}, html! {
        layout(&context.layout_data, html!{
            section {
                h1 { a."flatlink" href=(context.layout_data.links.forum_thread(&page.main)) { (opt_s!(page.main.name)) } }
                @if let Some(ptc_files) = page.ptc {
                    @if let Some(ptc_data) = ptc_files.text {
                        @let parsed_data = serde_json::de::from_str::<Vec<PtcData>>(&ptc_data)?;
                        @for ptc_file in parsed_data {
                            h3 { (ptc_file.name) }
                            @if let Some(description) = ptc_file.description {
                                p { (description)}
                            }
                            ({
                                //This is normal code, can do whatever in here
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
                                result.extend_from_slice(ftype); //.append(other)//copy_from_slice(ftype);
                                result.extend((zlibdata.len() as u32).to_le_bytes());
                                result.extend(rawlength.to_le_bytes());
                                result.extend(zlibdata);
                                ""
                            })
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

//pub fn get_qr_element() -> Markup
//{
//
//}