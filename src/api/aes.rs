use crypto::aes::{KeySize, ecb_decryptor, ecb_encryptor};
// use rustc_serialize::hex::ToHex;
use crypto::blockmodes::PkcsPadding;
use crypto::buffer::{RefReadBuffer, RefWriteBuffer, BufferResult, WriteBuffer, ReadBuffer};
use evpkdf::evpkdf;
// use hex::{encode, decode};
// use hex_literal::hex;
use md5::Md5;
use anyhow::{Error, Result};

const KEY_SIZE: usize = 256;
const IV_SIZE: usize = 128;
const SALT: &'static str = "fishbone";

// pub fn test_aes() {
//     let key = "myPassword";
//
//     let mut output = [0; (KEY_SIZE + IV_SIZE) / 8];
//
//     evpkdf::<Md5>(key.as_bytes(), b"fishbone", 1, &mut output);
//
//     let (key, iv) = output.split_at(KEY_SIZE / 8);
//     println!("Key: {:x?}", encode(&key[..]));
//     assert_eq!(key, hex!("2707b28406c1588efd892eed07e28d56bb119bcf9bc41a77d0fcbcddbf8c53a9"));
//
//     let enc = base64::decode("T8s339n7QKtTDTiYVcO2q3Vct9sOG8aI8Q6NnGCyiddZMawZg0IYUKTpWqm3Isj+".as_bytes()).expect("Failed to decode base64");
//     let mut decryption = ecb_decryptor(KeySize::KeySize256, key, PkcsPadding);
//     let mut buffer = vec![0; enc.len()];
//
//     let mut ref_write_buffer = RefWriteBuffer::new(&mut buffer);
//     decryption.decrypt(&mut RefReadBuffer::new(&enc[..]), &mut ref_write_buffer, true).expect("Failed to decrypt");
//     println!("Decrypted: {}", String::from_utf8(buffer).expect("").trim_matches(char::from(0)));
// }

pub fn aes_decrypt(encrypted: &str, password: &str) -> Result<String> {
    let mut outputs = [0; (KEY_SIZE + IV_SIZE) / 8];
    evpkdf::<Md5>(password.as_bytes(), SALT.as_bytes(), 1, &mut outputs);
    let (key, _iv) = outputs.split_at(KEY_SIZE / 8);

    let enc = base64::decode(encrypted.as_bytes())?;
    // println!("Decrypting: {:?}", enc);

    let mut decryption = ecb_decryptor(KeySize::KeySize256, key, PkcsPadding);
    let mut buffer = vec![0; enc.len()];
    let mut ref_write_buffer = RefWriteBuffer::new(&mut buffer);
    decryption.decrypt(&mut RefReadBuffer::new(&enc[..]), &mut ref_write_buffer, true).map_err(|e| {
        // println!("{:?}",e);
        Error::msg("SymmetricCipherError")
    })?;
    Ok(String::from_utf8(buffer)?.trim_end_matches(char::from(0)).to_string())
}

pub fn aes_encrypt(content: &str, password: &str) -> Result<String> {
    let mut outputs = [0; (KEY_SIZE + IV_SIZE) / 8];
    // println!("Encrypting: {}", content);
    evpkdf::<Md5>(password.as_bytes(), SALT.as_bytes(), 1, &mut outputs);
    let (key, _iv) = outputs.split_at(KEY_SIZE / 8);
    // println!("key: {:?}",key);

    let mut encryption = ecb_encryptor(KeySize::KeySize256, key, PkcsPadding);
    let mut final_result = Vec::<u8>::new();
    let mut buffer = vec![0; 9152];

    let mut ref_read_buffer = RefReadBuffer::new(content.as_bytes());
    let mut ref_write_buffer = RefWriteBuffer::new(&mut buffer);
    loop {
        let result = encryption.encrypt(&mut ref_read_buffer, &mut ref_write_buffer, true).map_err(|e| {
            // println!("{:?}",e);
            Error::msg("SymmetricCipherError")
        })?;

        // "write_buffer.take_read_buffer().take_remaining()" means:
        // from the writable buffer, create a new readable buffer which
        // contains all data that has been written, and then access all
        // of that data as a slice.
        final_result.extend(ref_write_buffer.take_read_buffer().take_remaining().iter().map(|&i| i));

        match result {
            BufferResult::BufferUnderflow => break,
            BufferResult::BufferOverflow => {}
        }
    }
    // // println!("Decrypting: {:?}", final_result);
    //
    Ok(base64::encode(&final_result))
    // Ok(String::new())
}

#[test]
fn test_aes_decrypt() {
    let encrypted = aes_encrypt("https://www.titanesmedellin.com/", "myPassword").expect("Failed to encrypt");
    let password = "myPassword";
    println!("encrypted: {}", encrypted);
    let decrypted = aes_decrypt(&encrypted, password).expect("Failed to decrypt");
    assert_eq!(decrypted, "https://www.titanesmedellin.com/");
}