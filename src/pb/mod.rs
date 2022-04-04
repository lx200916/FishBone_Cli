mod message;
use anyhow::Result;
use base64::decode;
pub use message::*;

// pub struct PasteContent {
//     id: String,
//     content:String,
//     lang:String,
//     password:bool,
// }
use prost::Message;
impl Paste{
    pub fn from_base64(text:String) -> Result<Self> {
        let buf=decode(text.as_bytes())?;
        Ok(Paste::decode(buf.as_slice())?)
    }
    pub fn to_base64(self) -> String {
        let buf=self.encode_to_vec();
        base64::encode(buf.as_slice()).to_string()
    }
}

impl NewPaste{
    pub fn from_base64(text:String) -> Result<Self> {
        let buf=decode(text.as_bytes())?;
        Ok(NewPaste::decode(buf.as_slice())?)
    }
    pub(crate) fn to_base64(self) -> String {
        let buf=self.encode_to_vec();
        base64::encode(buf.as_slice()).to_string()
    }
}





