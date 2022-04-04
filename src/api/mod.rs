mod api;
pub mod aes;

use std::borrow::Borrow;
use crate::pb::*;
use aes::*;
use std::env;
use std::sync::Mutex;
use once_cell::sync::OnceCell;
use reqwest::{Client, Error, get, Proxy};
use base64::{decode, encode};
use anyhow::{anyhow, Result};
use prost::Message;
use serde::Deserialize;


static HOST: &str = "https://bone.saltedfish.fun/_api/paste";
static PREFIX: &str = "##PasteMe##";
#[derive(Debug)]
pub enum DeleteType {
    NoDelete,
    DeleteToken(String),
}
#[derive(Debug)]
pub enum PasteType {
    Once,
    OneDay,
    SevenDays,
    Forever,
}

#[derive(Deserialize, Debug)]
pub struct MessageWrapper {
    id: String,
    status: i32,
    message: String,

}

impl From<i32> for PasteType {
    fn from(i: i32) -> Self {
        match i {
            0 => PasteType::Once,
            1 => PasteType::OneDay,
            2 => PasteType::SevenDays,
            3 => PasteType::Forever,
            _ => PasteType::Once,
        }
    }
}

#[derive(Debug)]
pub struct PasteConfig {
    pub(crate) delete_config: DeleteType,
    pub(crate) paste_type: PasteType,
}

impl Default for PasteConfig {
    fn default() -> Self {
        PasteConfig {
            delete_config: DeleteType::NoDelete,
            paste_type: PasteType::Once,
        }
    }
}

fn get_client() -> &'static reqwest::Client {
    static CLIENT: OnceCell<reqwest::Client> = OnceCell::new();
    CLIENT.get_or_init(|| {
        Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/80.0.3987.149 Safari/537.36")
            .build().expect("❌ Client Init Error!")
    })
}


async fn get_token() -> Result<String> {
    let token = get_client().request(reqwest::Method::GET, HOST.to_owned() + "/?method=beat").send().await?.text().await?;
    let buf = decode(token.as_bytes())?;
    let token: Paste = Paste::decode(buf.as_slice())?;
    // println!("token: {:?}", token.token_encryption);
    Ok(token.token_encryption.to_owned())
}

fn encrypt(content: String, password: &str) -> Result<String> {
    if password.is_empty() {
        Ok(content)
    } else {
        Ok(aes_encrypt(&format!("{}{}", PREFIX, content), password)?)
    }
}


pub async fn create(content: String, language: String, password: String, config: PasteConfig) -> Result<Paste> {
    let  client = get_client();
    let token = get_token().await?;
    // println!("token: {}", token);
    let url = match config.paste_type {
        PasteType::Once => "/once",
        PasteType::SevenDays => "/",
        _ => "/once",
    };
    let mut paste = NewPaste {
        xtoken: base64::encode(token.as_bytes()).to_string().replace("=", "!"),
        lang: language,
        content: content.to_owned(),
        timeline: config.paste_type as i32,
        ..Default::default()
    };
    if let DeleteType::DeleteToken(token) = config.delete_config {
        paste.token = token.clone();
        paste.token_encryption = encrypt(token, &password)?;
    }
    // println!("{:?}", password);
    if !password.is_empty() {
        paste.password = true;
        paste.content = encrypt(paste.content, &password)?;
    }
    // println!("{:?}",paste);


    let res = client.request(reqwest::Method::POST, HOST.to_owned() + url).body(paste.to_base64()).send().await?;
    let text = res.text().await?;
    // println!("{:?}", text);

    let res: Paste = Paste::from_base64(text)?;
    // println!("{:?}",res );
    if res.status!=201 {
        return Err(anyhow!(res.message));
    }
    Ok(res)
}

pub fn decrypt(content: &str, password: &str) -> Result<String> {
    if password.is_empty() {
         Ok(content.to_owned())
    } else {
        let content = aes_decrypt(content, password)?;
        // println!("{}", content);
        if content.starts_with(PREFIX) {
            Ok(content.split_at(PREFIX.len()).1.to_owned())
        } else {
            Err(anyhow!("Decrypt Error!"))
        }
    }
}
impl Paste {
    pub fn decrypt(&self, password: &str) -> Option<String> {
        if password.is_empty() {
            return None;
        }
        // Some(decrypt(&self.content, password).unwrap())
        decrypt(&self.content, password).ok()
    }
}

pub async fn get_paste(id: String) -> Result<Paste> {
    let mut client = get_client();
    let url = format!("/{}?json=true", id);
    let res = client.request(reqwest::Method::GET, HOST.to_owned() + &url).send().await?;
    let text = res.text().await?;
    let res: Paste = Paste::from_base64(text)?;
    if res.status!=200&&res.status!=401 {
        return Err(anyhow!(res.message));
    }
    // println!("{:?}",res );
    Ok(res)
}

pub async fn delete_paste(paste:&Paste,password:&str) -> Result<()> {
    let token=decrypt(&paste.token_encryption, password)?;
    let mut client = get_client();
    let url = format!("/{}?token={}", paste.id, token);
    let res = client.request(reqwest::Method::DELETE, HOST.to_owned() + &url).send().await?;
    let resp: MessageWrapper = res.json().await?;
    if resp.status != 200&&resp.status != 404 {
        return Err(anyhow!("HTTP Error: {}",resp.message));
    }
    // println!("{:?}",res );
    Ok(())
}