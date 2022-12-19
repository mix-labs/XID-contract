use candid::{CandidType};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, CandidType)]
pub enum VerifyError {
    SigDecoErr,
    VerifyErr,
    MsgDecodeErr,
    IcPrincipalErr,
    IDExist,
    XidNotExist,
    XidCNoNameErr,
    ReplayErr,
}

#[derive(Serialize, Deserialize, Debug, Clone, CandidType)]
pub struct Verification {
    pub message: Vec<u8>,
    pub signature: Vec<u8>,
    pub public_key: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone, CandidType, Default)]
pub struct Payload {
    pub action : String,  // 行为
    pub created_at : String, // 创建时间
    pub identity : String, // 地址 推特Id
    pub persona : String, // 待删除
    pub platform : String, // 平台
    pub uuid : String,
}

#[derive(Serialize, Deserialize, Debug, Clone, CandidType)]
pub struct MsgIn {
    pub msg : String,
    pub sig : String,
}