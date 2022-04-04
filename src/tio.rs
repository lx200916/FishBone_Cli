use std::io::Write;
use anyhow::{anyhow, Result};
use crossterm::style::Color::{AnsiValue, Magenta, Yellow};

use crate::Paste;
use syntect::{
    easy::HighlightLines,
    highlighting::{Style, ThemeSet},
    parsing::SyntaxSet,
    util::{as_24_bit_terminal_escaped, LinesWithEndings},
};
use termimad::*;

fn is_tty(out:bool) ->bool{
    if out {
atty::is(atty::Stream::Stdout)
    } else {
atty::is(atty::Stream::Stdin)
    }
}
pub fn copy(str:&str){
    if is_tty(false) {
        terminal_clipboard::set_string(str);
    }
}

pub fn prompt(info: &str) -> Result<String> {
    if !is_tty(true){
        return Err(anyhow!("密码错误"));
    }
    print!(" 🔑 {}:", info);
    std::io::stdout().flush()?;
    let mut s = String::new();
    std::io::stdin().read_line(&mut s)?;
    Ok(s.trim().to_string())




}
fn make_skin() -> MadSkin {
    let mut skin = MadSkin::default();
    skin.table.align = Alignment::Center;
    skin.set_headers_fg(AnsiValue(178));
    skin.bold.set_fg(Yellow);
    skin.italic.set_fg(Magenta);
    skin.scrollbar.thumb.set_fg(AnsiValue(178));
    skin.code_block.align = Alignment::Center;
    skin
}
pub fn print_paste(paste:&Paste,raw:bool){
    println!("{}",paste.lang.to_ascii_lowercase());
   if raw||!is_tty(true) {
       println!("{}", paste.content);
   }else if paste.lang.to_ascii_lowercase()=="markdown"{
       println!();
       eprintln!("{}", make_skin().term_text(&paste.content));

   }else{
       let ps = SyntaxSet::load_defaults_newlines();
       let ts = ThemeSet::load_defaults();
       let syntax = ps.find_syntax_by_extension(&detect_lang::from_id(&paste.lang).unwrap_or("txt")).unwrap();
       let mut h = HighlightLines::new(syntax, &ts.themes["base16-ocean.dark"]);
       for line in LinesWithEndings::from(&paste.content) {
           let ranges: Vec<(Style, &str)> = h.highlight(line, &ps);
           let escaped = as_24_bit_terminal_escaped(&ranges[..], false);
           print!("{}", escaped);
       }

   }
    println!();


}