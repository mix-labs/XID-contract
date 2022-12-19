use candid::{CandidType, Principal};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub enum XidError {
    DataNotExist,
    UuidRepeat,
    UuidNotExist,
    IDNotExist,
    MainIdBan,
    FieldOutOfRange,
    XidNotExist,
    XidCNoNameErr,
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub enum XidResponse {
    VerifyOk,
    StoreOk,
    DeleteOk,
    MintOk,
    ChangeIdOk,
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub enum XidCenterError {
    Invalid_Operation,
    Invalid_Platform,
    NotXidOwner,
    XidNotExist,
    IDExist,
    IDNotExist,
}

#[derive(Default, Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct ID {
    pub platform : String,
    pub identity : String,
    pub bind_time : String,
}
impl Eq for ID {}
impl PartialEq<Self> for ID {
    fn eq(&self, other: &Self) -> bool {
        (self.platform.clone() + &self.identity) == (other.platform.clone() + &other.identity)
    }
}
impl PartialOrd<Self> for ID {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some((self.platform.clone() + &self.identity).cmp(&(other.platform.clone() + &other.identity)))
    }
}
impl Ord for ID {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.platform.clone() + &self.identity).cmp(&(other.platform.clone() + &other.identity))
    }
}

#[derive(Default, Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct Xid {
    pub pub_key : String,
    pub name : String,
    pub main_id : ID,
    pub ids : Vec<ID>,
    pub avatar_url : String,
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct StoreArg {
    pub uuid : String,
    pub d_platform : String,
    pub content : Contents,
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct Storage {
    pub owner : String,
    pub uuid : String,
    pub content : Contents,
    pub d_platform : String,
    pub is_minted : bool,
    pub mint_time : String,
    pub upload_time : String,
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct TwitterStorage {
    pub owner : String,
    pub uuid : String,
    pub twitter_content : TwitterContent,
    pub d_platform : String,
    pub is_minted : bool,
    pub mint_time : String,
    pub upload_time : String,
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct OffStorage {
    pub owner : String,
    pub uuid : String,
    pub off_content : OffChainContent,
    pub d_platform : String,
    pub is_minted : bool,
    pub mint_time : String,
    pub upload_time : String,
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct ContentUuid {
    pub content_type: ContentType,
    pub uuid : String,
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub enum ContentType {
    Twitter,
    OffChain,
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub enum Contents {
    TwitterContent(TwitterContent),
    OffChainContent(OffChainContent),
}

#[derive(Default, Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct TwitterContent {
    pub url : String,
    pub text_content : String,
    pub text_url : String,
    pub image_urls : Vec<String>,
    pub video_url : String,
    pub post_time : String, // 推文发布时间
}

#[derive(Default, Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct OffChainContent {
    pub local_content_type : String,
    pub file_type: String,
    pub text_content : String,
    pub url : String
}

#[derive(Default, Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct SimpleId {
    pub platform : String,
    pub identity : String,
}

#[derive(Default, Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct XidArgs {
    pub name : Option<String>,
    pub avatar_url : Option<String>,
}

#[derive(Default, Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct Avatar {
    pub image_data : Vec<u8>,
    pub image_type : String,
}

#[derive(Default, Deserialize, Serialize, CandidType, Clone)]
pub struct State {
    pub pub_key : RefCell<String>,
    pub name : RefCell<String>,
    pub main_id : RefCell<ID>,
    pub ids : RefCell<BTreeSet<ID>>,
    pub ic_verify : RefCell<String>,
    pub avatar_url : RefCell<String>,
    pub avatar : RefCell<Avatar>,
    pub twitter_store : RefCell<BTreeMap<String, TwitterStorage>>,
    pub off_store : RefCell<BTreeMap<String, OffStorage>>,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct StableState {
    pub pub_key : String,
    pub name : String,
    pub main_id : ID,
    pub ids : BTreeSet<ID>,
    pub ic_verify : String,
    pub avatar_url : String,
    pub avatar : Avatar,
    pub twitter_store : BTreeMap<String, TwitterStorage>,
    pub off_store : BTreeMap<String, OffStorage>,
}