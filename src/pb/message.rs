#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Paste {
    #[prost(string, tag="1")]
    pub id: ::prost::alloc::string::String,
    #[prost(int32, tag="2")]
    pub status: i32,
    #[prost(string, tag="3")]
    pub content: ::prost::alloc::string::String,
    #[prost(string, tag="4")]
    pub lang: ::prost::alloc::string::String,
    #[prost(bool, tag="5")]
    pub password: bool,
    #[prost(string, tag="6")]
    pub error: ::prost::alloc::string::String,
    #[prost(string, tag="7")]
    pub message: ::prost::alloc::string::String,
    #[prost(string, tag="8")]
    pub token_encryption: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NewPaste {
    #[prost(string, tag="1")]
    pub content: ::prost::alloc::string::String,
    #[prost(string, tag="2")]
    pub lang: ::prost::alloc::string::String,
    #[prost(bool, tag="3")]
    pub password: bool,
    #[prost(bool, tag="4")]
    pub once: bool,
    #[prost(string, tag="5")]
    pub token_encryption: ::prost::alloc::string::String,
    #[prost(string, tag="6")]
    pub token: ::prost::alloc::string::String,
    #[prost(string, tag="7")]
    pub xtoken: ::prost::alloc::string::String,
    #[prost(int32, tag="8")]
    pub timeline: i32,
}
