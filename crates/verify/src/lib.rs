use std::cell::RefCell;
use secp256k1::{Message, PublicKey, RecoveryId, Signature, PublicKeyFormat};
use secp256k1::util::{MESSAGE_SIZE, SIGNATURE_SIZE};
use secp256k1::util::{FULL_PUBLIC_KEY_SIZE, RAW_PUBLIC_KEY_SIZE, COMPRESSED_PUBLIC_KEY_SIZE};
use sha3::{Digest, Keccak256};
use candid::{CandidType, candid_method};
use ic_cdk_macros::{query, pre_upgrade, post_upgrade};
use ic_cdk;
use serde::{Deserialize, Serialize};
use serde_json;
use base64;
use std::collections::{BTreeSet};

thread_local! {
    static STATE : State = State::default();
}

pub const RECOVERY_ID_SIZE: usize = 1;

#[derive(Default, Deserialize, Serialize, CandidType, Clone)]
pub struct State {
    pub uuids : RefCell<BTreeSet<String>>,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct StableState {
    pub uuids : BTreeSet<String>,
}

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

#[query(name = "msg_in")]
#[candid_method(query, rename = "msg_in")]
pub fn msg_in(msgin : MsgIn) -> Result<Payload, VerifyError> {
    let res : Payload = match serde_json::from_str(&msgin.msg) {
        Ok(tmp) => tmp,
        Err(_) => return Err(VerifyError::MsgDecodeErr)
    };
    let mut flag = false;
    STATE.with(|s| {
        let mut uuids = s.uuids.borrow_mut();
        if uuids.contains(&res.uuid) {
            flag = true;
        } else {
            uuids.insert(res.uuid.clone());
        };
    });
    if flag {return Err(VerifyError::ReplayErr)};
    let pub_k : Vec<u8> = [2, 142, 36, 253, 150, 84, 241, 44, 121, 61, 61, 55, 108, 21, 247, 171, 229, 62, 15, 189, 83, 120, 132, 163, 169, 141, 16, 210, 220, 109, 81, 59, 78].to_vec();
    let msg_32 = hash_keccak256(msgin.msg);
    let sig_vec = msgin.sig.clone().into_bytes();
    let sig_deco = match base64::decode(sig_vec) {
        Ok(res) => res,
        Err(_) => return Err(VerifyError::SigDecoErr),
    };
    if verify(Verification {
        message : msg_32.to_vec(),
        signature : sig_deco[0..64].to_owned(),
        public_key : pub_k }) {
        Ok(res)
    } else {
        Err(VerifyError::VerifyErr)
    }
}


fn verify(verification: Verification) -> bool {
    if verification.message.len() != MESSAGE_SIZE { return false };
    let msg = match Message::parse_slice(&verification.message) {
        Ok(res) => res,
        Err(_) => return false,
    };

    if verification.signature.len() != SIGNATURE_SIZE { return false };
    let sig = match Signature::parse_standard_slice(&verification.signature) {
        Ok(res) => res,
        Err(_) => return false,
    };

    let pub_key = if verification.public_key.len() == FULL_PUBLIC_KEY_SIZE {
        match PublicKey::parse_slice(&verification.public_key, Some(PublicKeyFormat::Full)) {
            Ok(res) => res,
            Err(_) => return false,
        }
    } else if verification.public_key.len() == RAW_PUBLIC_KEY_SIZE {
        match PublicKey::parse_slice(&verification.public_key, Some(PublicKeyFormat::Raw)) {
            Ok(res) => res,
            Err(_) => return false,
        }
    } else if verification.public_key.len() == COMPRESSED_PUBLIC_KEY_SIZE {
        match PublicKey::parse_slice(&verification.public_key, Some(PublicKeyFormat::Compressed)) {
            Ok(res) => res,
            Err(_) => return false,
        }
    } else if verification.public_key.len() == RECOVERY_ID_SIZE {
        let rec_id = match RecoveryId::parse(verification.public_key[0]) {
            Ok(res) => res,
            Err(_) => return false,
        };
        match secp256k1::recover(&msg, &sig, &rec_id) {
            Ok(res) => res,
            Err(_) => return false,
        }
    } else {
        return false
    };

    secp256k1::verify(&msg, &sig, &pub_key)
}

fn hash_keccak256(payload: String) -> [u8; 32] {
    let message = format!("\x19Ethereum Signed Message:\n{}{}", payload.len(), payload);
    let mut hasher = Keccak256::new();
    hasher.update(message);
    hasher.finalize().into()
}


fn do_clear() {
    STATE.with(|s| {
        s.uuids.borrow_mut().clear();
    })
}

#[pre_upgrade]
fn pre_upgrade() {
    let stable_state : StableState = STATE.with(|s| StableState{
        uuids: s.uuids.take(),
    });
    ic_cdk::storage::stable_save((stable_state, )).expect("failed to save stable state");
}

#[post_upgrade]
fn post_upgrade() {
    do_clear();
    let (stable_state,) : (StableState, ) =
        ic_cdk::storage::stable_restore().expect("failed to restore stable state");

    STATE.with(|s| {
        s.uuids.replace(stable_state.uuids);
    })
}