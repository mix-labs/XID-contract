use crate::rc_bytes::RcBytes;
use serde_bytes::ByteBuf;
use ic_cdk::export::candid::{Func, Nat, CandidType, Deserialize};

// HTTP interface
#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct HttpRequest {
    pub method: String,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: ByteBuf,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct HttpResponse {
    status_code: u16,
    headers: Vec<(String, String)>,
    body: RcBytes,
    streaming_strategy: Option<StreamingStrategy>,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct StreamingCallbackToken {
    pub key: String,
    pub content_encoding: String,
    pub index: Nat,
    // We don't care about the sha, we just want to be backward compatible.
    pub sha256: Option<ByteBuf>,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub enum StreamingStrategy {
    Callback {
        callback: Func,
        token: StreamingCallbackToken,
    },
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct StreamingCallbackHttpResponse {
    body: RcBytes,
    token: Option<StreamingCallbackToken>,
}

pub fn build_202(image_data : Vec<u8>,  image_type: String) -> HttpResponse {
    HttpResponse{
        status_code : 200,
        headers : vec![(String::from("Content-Type"), String::from(image_type + ";charset=utf-8")),
                       (String::from("Cache-Control"), String::from("max-age=680400"))],
        streaming_strategy : None,
        body : RcBytes::from( ByteBuf::from(image_data)),
    }
}

pub fn build_404() -> HttpResponse {
    HttpResponse {
        status_code : 404,
        headers : vec![(String::from("Content-Type"), String::from("text/html"))],
        body : RcBytes::from(ByteBuf::from(String::from(
            "<html> <head> <meta charset=") +
            "UTF-8> <link href='//fonts.googleapis.com/css?family=Lato:100' \
            rel='stylesheet' type='text/css'> <style> body { margin: 0; padding: 0; \
            width: 100%; height: 100%; color: #B0BEC5; display: table; font-weight: 100; font-family: 'Lato'; } \
            .container { text-align: center; display: table-cell; vertical-align: middle; } .content { text-align: \
            center; display: inline-block; } .title { font-size: 42px; margin-bottom: 40px; } \
            </style> </head> <body> <div class=" + "container" + "> <div class=" +"content" + "> \
            <div class=" +"title" + ">404 NOT FOUND</div> </div> </div> </body> </html>")),
        streaming_strategy : None,
    }
}
