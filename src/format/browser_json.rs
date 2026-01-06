use std::io;
use std::string::FromUtf8Error;
use serde::de::DeserializeOwned;
use serde::Serialize;
use thiserror::Error;
use wasm_bindgen_futures::wasm_bindgen::JsValue;
use web_sys::js_sys;
use crate::format::{Format, FormatInfo};

const CONTENT_TYPE: & str = "application/json";

#[derive(Debug, Copy, Clone)]
/// [JavaScript Object Notation](https://www.json.org/)
pub struct BrowserJson;

impl<Read, Write> Format<Read, Write> for BrowserJson
where Read: DeserializeOwned, Write: Serialize
{
    const INFO: &'static FormatInfo = &FormatInfo {
        http_content_type: CONTENT_TYPE,
    };
    type ReadError = Error;
    type WriteError = Error;

    fn read(&self, mut reader: impl io::Read) -> Result<Read, Self::ReadError> {
        let mut json = Vec::new();
        reader.read_to_end(&mut json)?;
        let json = String::from_utf8(json)?;
        let value = js_sys::JSON::parse(&json).map_err(Error::Parse)?;
        Ok(serde_wasm_bindgen::from_value(value)?)
    }

    fn write(&self, value: Write, mut writer: impl io::Write) -> Result<(), Self::WriteError> {
        let value = serde_wasm_bindgen::to_value(&value)?;
        let json = js_sys::JSON::stringify(&value).map_err(Error::Parse)?;
        writer.write_all(json.as_string().expect("failed to decode json string").as_bytes())?;
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to parse json: {0:?}")]
    Parse(JsValue),
    #[error("failed to serialize json: {0}")]
    Serde(#[from] serde_wasm_bindgen::Error),
    #[error(transparent)]
    IO(#[from] io::Error),
    #[error("failed to decode string: {0}")]
    Decode(#[from] FromUtf8Error)
}
