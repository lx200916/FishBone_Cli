mod pb;
mod api;
mod tio;

use std::io::Read;
use detect_lang::{from_path, Language};

use api::*;
use crate::api::create;
use anyhow::{anyhow, Result};
use clap::Parser;
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use crate::pb::Paste;
use crate::tio::{copy, print_paste, prompt};

const MAX_SIZE: usize = 512 * 1024;
const MAX_PEEK_SIZE: usize = 1024;

#[derive(Parser)]
#[clap(author, version, about)]
struct Args {
    #[clap(subcommand)]
    action: Action,
}

#[derive(clap::Subcommand)]
enum Action {
    Create(Create),
    Get(Get),
}

#[derive(Parser, Debug)]
struct Create {
    /// 阅后即焚
    #[clap(short, long)]
    burn: bool,
    /// 1天后自动删除
    #[clap(long="OneDay")]
    one_day: bool,
    /// 7天后自动删除
    #[clap(long="OneWeek")]
    one_week: bool,
    #[clap(long="Forever")]
    forever: bool,
    /// 不创建删除Token
    #[clap(long="NoDelete")]
    no_delete: bool,
    /// 将传入内容作为文件路径读取
    #[clap(long, short)]
    file: bool,
    #[clap(long, short)]
    language: Option<String>,
    #[clap(long, short)]
    password: Option<String>,
    /// 复制到剪贴板
    #[clap(short, long)]
    copy: bool,
    /// 内容
    content: String,

}

#[derive(Parser, Debug)]
struct Get {
    /// Paste ID.
    #[clap(parse(try_from_str=id_check))]
    id: String,
    /// 不发送焚毁请求
    #[clap(short, long)]
    keep: bool,
    ///以 Raw 方式输出，不追加高亮和格式化
    #[clap(short, long)]
    raw: bool,
    /// Paste 密码
    #[clap(short, long)]
    password: Option<String>,
    /// 复制到剪贴板
    #[clap(short, long)]
    copy: bool,

}

impl PasteType {
    fn from_args(args: &Create) -> PasteType {
        match args {
            Create { burn: true, .. } => PasteType::Once,
            Create { one_day: true, .. } => PasteType::OneDay,
            Create { one_week: true, .. } => PasteType::SevenDays,
            Create { forever: true, .. } => PasteType::Forever,
            _ => PasteType::Once,
        }
    }
}
fn id_check(id: &str) -> Result<String> {
    let id=id.replace("https://bone.saltedfish.fun/","");
    match id {
         id if !(3..=8).contains(&id.len())  => Err(anyhow!("ID length must be 3-8")),
         id if !id.chars().all(|c| c.is_ascii_alphanumeric()) => Err(anyhow!("ID must be alphanumeric")),
         _=> Ok(id)

    }
}
fn random_string(len: usize) -> String {
    thread_rng().sample_iter(&Alphanumeric).take(len).map(char::from).collect()
}

fn detect_language(args: &Create) -> String {
    match args {
        Create { language: Some(lang), .. } => lang.to_string(),
        Create { file: true, .. } => {
            let path = args.content.clone();
            let lang = from_path(&path);
            lang.map_or("plaintext".to_string(), |lang| lang.id().to_string())
        }
        _ => "plaintext".to_string(),
    }
}

async fn create_subcommand(args: Create) -> Result<()> {
    let mut delete_token = String::new();
    let config = PasteConfig {
        paste_type: PasteType::from_args(&args),
        delete_config: if args.no_delete {
            DeleteType::NoDelete
        } else {
            delete_token = random_string(6);
            DeleteType::DeleteToken(delete_token.clone())
        },
    };
    let lang = detect_language(&args);
    let content = if args.file {
        {
            let f = std::fs::File::open(&args.content)?;
            if f.metadata()?.len() as usize > MAX_SIZE {
                return Err(anyhow!("file too large"));
            }
            let mut buffer: Vec<u8> = vec![];

            f.take(MAX_PEEK_SIZE as u64).read_to_end(&mut buffer)?;


            if content_inspector::inspect(&buffer).is_binary() {
                return Err(anyhow::anyhow!("binary file"));
            }
        }

        std::fs::read_to_string(&args.content)?
    } else {
        args.content
    };
    if content.is_empty() {
        return Err(anyhow!("content is empty"));
    }
    // println!("{:?}", (&config, &lang, &content));
    let paste = create(content, lang, args.password.unwrap_or(String::new()), config).await?;
    println!("https://bone.saltedfish.fun/{}", paste.id);
    if args.copy {
       copy(&format!("https://bone.saltedfish.fun/{}", paste.id));
    }
    if !args.no_delete {
        println!("删除Token:{}", delete_token);
    }
    Ok(())
}
fn check_password(paste: &mut Paste,password:String) -> Result<String> {
    if !paste.password {
        return Ok(String::new());
    }
    let mut password = password;
    while let None = paste.decrypt(&password) {
        if password.is_empty() {
            password = prompt("Paste被加密，请输入密码")?;
        }else {
            password = prompt("密码错误，请重新输入")?;
        }
    }
    paste.content=paste.decrypt(&password).unwrap();


    Ok(password)
}
async fn get_subcommand(args: Get) -> Result<()> {
    let mut paste=get_paste(args.id, ).await?;
    // println!("{:?}",paste);
    let password=check_password(&mut paste,args.password.unwrap_or(String::new()))?;
    print_paste(&paste,args.raw);
    if args.copy{
        copy(&paste.content);
    }
    if !args.keep {

        delete_paste(&paste,&password).await;
    }


    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {

    // println!("Hello, world!");
    let args: Args = Args::parse();
    match args.action {
        Action::Create(create) => {
            create_subcommand(create).await
        }
        Action::Get(ger) => {
            get_subcommand(ger).await
        }
    }
}
