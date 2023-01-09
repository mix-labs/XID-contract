pub mod types;
pub mod verify;
pub mod http;
pub mod rc_bytes;

use std::ptr::null;
use types::{Xid, Contents, StoreArg, ContentType,
            TwitterContent, OffChainContent, XidResponse,
            TwitterStorage, OffStorage, ContentUuid
            , XidArgs, Avatar, State, XidError, SimpleId,
            StableState, ID, XidCenterError, Storage};
use verify::{Payload, VerifyError, MsgIn};
use http::{HttpRequest, HttpResponse, build_404, build_202};
use candid::{candid_method, Principal};
use ic_kit::{ic};
use ic_cdk::{caller};
use ic_cdk_macros::{init, update, query, pre_upgrade, post_upgrade};

thread_local! {
    static STATE : State = State::default();
}
pub const VERSION : u8 = 0;

#[init]
#[candid_method(init)]
fn init(arg : Principal) {
    STATE.with(|s| {
        *s.pub_key.borrow_mut() = Principal::to_text(&arg);
    })
}

#[query(name = "getVersion")]
#[candid_method(query, rename = "getVersion")]
fn get_version() -> u8 {
    VERSION
}

#[query(name = "getCycleBalance")]
#[candid_method(query, rename = "getCycleBalance")]
fn get_cycle_balance() -> u64 {
    ic::balance()
}

#[query(name = "getXid")]
#[candid_method(query, rename = "getXid")]
fn get_xid() -> Xid {
    STATE.with(|s| {
        Xid {
            pub_key: s.pub_key.borrow().clone(),
            name: s.name.borrow().clone(),
            main_id: s.main_id.borrow().clone(),
            ids: s.ids.borrow().iter().map(|l| l.clone()).collect(),
            avatar_url: s.avatar_url.borrow().clone(),
        }
    })
}

#[query(name = "getMainId")]
#[candid_method(query, rename = "getMainId")]
fn get_main_id() -> ID {
    STATE.with(|s|{
        s.main_id.borrow().clone()
    })
}

#[query(name = "getStoreSize")]
#[candid_method(query, rename = "getStoreSize")]
fn get_store_size(arg : ContentType) -> usize {
    STATE.with(|s| {
        return match arg {
            ContentType::Twitter => {
                s.twitter_store.borrow().len()
            },
            ContentType::OffChain => {
                s.off_store.borrow().len()
            }
        }
    })
}

#[query(name = "getStoreList")]
#[candid_method(query, rename = "getStoreList")]
fn get_store_list(arg : ContentType, start : usize, offset : usize) -> Result<Vec<Storage>, XidError> {
    STATE.with(|s|{
        return match arg {
            ContentType::Twitter => {
                if start >= s.twitter_store.borrow().len() {
                    Err(XidError::FieldOutOfRange)
                } else {
                    let mut tmp_offset : usize = 0;
                    if start + offset > s.twitter_store.borrow().len() {
                        tmp_offset = s.twitter_store.borrow().len() - start;
                    } else {
                        tmp_offset = offset;
                    };
                    let mut st : usize = 0;
                    let mut res : Vec<Storage>  = Vec::new();
                    let twitter_store = s.twitter_store.borrow();
                    let twitter_vec  =
                        twitter_store.
                            values();
                    for ts in twitter_vec {
                        if st >= start && st < (start + tmp_offset) {
                            res.push(Storage{
                                owner: ts.owner.clone(),
                                uuid: ts.uuid.clone(),
                                content: Contents::TwitterContent(ts.twitter_content.clone()),
                                d_platform: ts.d_platform.clone(),
                                is_minted: ts.is_minted.clone(),
                                mint_time: ts.mint_time.clone(),
                                upload_time: ts.upload_time.clone(),
                            })
                        };
                        st += 1;
                    }
                    Ok(res)
                }
            },
            ContentType::OffChain => {
                if start >= s.off_store.borrow().len() {
                    Err(XidError::FieldOutOfRange)
                } else {
                    let mut tmp_offset : usize = 0;
                    if start + offset > s.twitter_store.borrow().len() {
                        tmp_offset = s.twitter_store.borrow().len() - start;
                    } else {
                        tmp_offset = offset;
                    };
                    let mut st : usize = 0;
                    let mut res : Vec<Storage>  = Vec::new();
                    let off_store = s.off_store.borrow();
                    let off_vec =
                        off_store.
                            values();
                    for os in off_vec {
                        if st >= start && st < (start + tmp_offset) {
                            res.push(Storage{
                                owner: os.owner.clone(),
                                uuid: os.uuid.clone(),
                                content: Contents::OffChainContent(os.off_content.clone()),
                                d_platform: os.d_platform.clone(),
                                is_minted: os.is_minted.clone(),
                                mint_time: os.mint_time.clone(),
                                upload_time: os.upload_time.clone(),
                            })
                        };
                        st += 1;
                    }
                    Ok(res)
                }
            }
        }
    })
}

#[query(name = "getStoreByUuid")]
#[candid_method(query, rename = "getStoreByUuid")]
fn get_store_by_uuid(arg : Vec<ContentUuid>) -> Vec<Storage> {
    let mut res : Vec<Storage> = Vec::new();
    STATE.with(|s| {
        let twitter_store = s.twitter_store.borrow();
        let off_store = s.off_store.borrow();
        for ts in arg {
            match ts.content_type {
                ContentType::Twitter => {
                    match twitter_store.get(&ts.uuid) {
                        Some(store) => {
                            res.push(Storage{
                                owner: store.owner.clone(),
                                uuid: store.uuid.clone(),
                                content: Contents::TwitterContent(store.twitter_content.clone()),
                                d_platform: store.d_platform.clone(),
                                is_minted: store.is_minted.clone(),
                                mint_time: store.mint_time.clone(),
                                upload_time: store.upload_time.clone(),
                            })
                        },
                        None => {},
                    };
                },
                ContentType::OffChain => {
                    match off_store.get(&ts.uuid) {
                        Some(store) => {
                            res.push(Storage{
                                owner: store.owner.clone(),
                                uuid: store.uuid.clone(),
                                content: Contents::OffChainContent(store.off_content.clone()),
                                d_platform: store.d_platform.clone(),
                                is_minted: store.is_minted.clone(),
                                mint_time: store.mint_time.clone(),
                                upload_time: store.upload_time.clone(),
                            })
                        },
                        None => {},
                    };
                },
            };
        };

    });
    res
}

#[query(name = "http_request")]
#[candid_method(query, rename = "http_request")]
fn http_request(request : HttpRequest) -> HttpResponse {
    STATE.with(|s| {
        let path = request.url.split_terminator("/").collect::<Vec<&str>>();
        if path.len() == 3 {
            if path[1] == "avatar" {
                let avatar = s.avatar.borrow().clone();
                return build_202(avatar.image_data, avatar.image_type);
            }
        }
        build_404()
    })
}

#[update(name = "changeMainId", guard="is_authorized")]
#[candid_method(update, rename = "changeMainId")]
async fn change_main_id(arg : ID) -> Result<XidResponse, XidError> {
    STATE.with(|s| {
        if s.ids.borrow().contains(&arg) {
            *s.main_id.borrow_mut() = arg;
            Ok(XidResponse::ChangeIdOk)
        } else {
            Err(XidError::IDNotExist)
        }
    })
}

#[update(name = "unboundId", guard="is_authorized")]
#[candid_method(update, rename = "unboundId")]
async fn unbound_id(arg : ID) -> Result<XidResponse, XidError> {
    let mut flag = Ok(XidResponse::ChangeIdOk);
    STATE.with(|s| {
        if s.ids.borrow().contains(&arg) {
            let mut main_id = s.main_id.borrow_mut();
            if *main_id == arg {
                *main_id = ID{
                    platform: "".to_string(),
                    identity: "".to_string(),
                    bind_time: "".to_string(),
                };
                let mut ids = s.ids.borrow_mut();
                let _ = ids.remove(&arg);
            } else {
                let mut ids = s.ids.borrow_mut();
                let _ = ids.remove(&arg);
            };
        } else {
            flag = Err(XidError::IDNotExist);
        };
    });
    return match flag {
        Ok(ok) => {
            let simple_id = SimpleId{
                platform: arg.platform,
                identity: arg.identity,
            };
            let xid_center = Principal::from_text("sgdrt-caaaa-aaaal-qbola-cai").unwrap();
            if let Ok((x, )) = ic::call::<_, (Result<(), XidCenterError>, ), _>(
                xid_center,
                "deleteID",
                (&simple_id, )
            ).await {
                match x {
                    Ok(_) => {},
                    Err(er) => {
                        return match er {
                            XidCenterError::IDNotExist => {
                                Err(XidError::IDNotExist)
                            },
                            XidCenterError::XidNotExist => {
                                Err(XidError::XidNotExist)
                            },
                            _ => { Err(XidError::XidCNoNameErr) },
                        };
                    },
                }
            }
            Ok(ok)
        },
        Err(err) => { Err(err) }
    }
}

// xid owner调用约定待绑定ic身份
#[update(name = "verifyIcPre", guard="is_authorized")]
#[candid_method(update, rename = "verifyIcPre")]
async fn verify_ic_pre(arg : String) -> bool {
    STATE.with(|s| {
        *s.ic_verify.borrow_mut() = arg;
        true
    })
}

// ic被绑定身份调用
#[update(name = "verifyIcPost", guard="is_ic_authorized")]
#[candid_method(update, rename = "verifyIcPost")]
async fn verify_ic_post() -> Result<XidResponse, VerifyError> {
    let ic_verify = STATE.with(|s| {
        let mut ic_verify = s.ic_verify.borrow_mut();
        let ic_tmp = ic_verify.clone();
        *ic_verify = "".to_string();
        ic_tmp
    });
    let id = ID {
        platform: "ic".to_string(),
        identity: ic_verify.clone(),
        bind_time: ic_cdk::api::time().to_string(),
    };
    match Principal::from_text(&ic_verify.clone()) {
        Err(_) => { return  Err(VerifyError::IcPrincipalErr); },
        Ok(r) => {
            if r == caller() {
                let simple_id = SimpleId{
                    platform: "ic".to_string(),
                    identity: ic_verify.clone(),
                };
                let xid_center = Principal::from_text("sgdrt-caaaa-aaaal-qbola-cai").unwrap();
                if let Ok((x, )) = ic::call::<_, (Result<(), XidCenterError>, ), _>(
                    xid_center,
                    "putID",
                    (&simple_id, )
                ).await {
                    match x {
                        Ok(_) => {},
                        Err(er) => {
                            return match er {
                                XidCenterError::IDExist => {
                                    Err(VerifyError::IDExist)
                                },
                                XidCenterError::XidNotExist => {
                                    Err(VerifyError::XidNotExist)
                                },
                                _ => { Err(VerifyError::XidCNoNameErr) },
                            };
                        },
                    }
                };
            } else {
                return Err(VerifyError::VerifyErr);
            };
        },
    };
    STATE.with(|s | {
        let mut ids = s.ids.borrow_mut();
        let mut main_id = s.main_id.borrow_mut();
        if *main_id.identity == "".to_string() {
            *main_id = id.clone();
        };
        ids.insert(id);
        Ok(XidResponse::VerifyOk)
    })
}

#[update(name = "verifyID", guard="is_authorized")]
#[candid_method(update, rename = "verifyID")]
async fn verify_id(msg : MsgIn) -> Result<XidResponse, VerifyError> {
    let verify = Principal::from_text("sbcxh-pyaaa-aaaal-qbolq-cai").unwrap();
    let mut pay_load = Payload::default();
    if let Ok((x, )) = ic::call::<_, (Result<Payload, VerifyError>, ), _>(
        verify,
        "msg_in",
        (&msg, )
    ).await {
        match x {
            Ok(p) => {
                pay_load = p;
            },
            Err(er) => {return Err(er)},
        }
    };
    let id = ID {
        platform: pay_load.platform.clone(),
        identity: pay_load.identity.clone(),
        bind_time: ic_cdk::api::time().to_string(),
    };
    let simple_id = SimpleId{
        platform: pay_load.platform,
        identity: pay_load.identity,
    };
    let xid_center = Principal::from_text("sgdrt-caaaa-aaaal-qbola-cai").unwrap();
    if let Ok((x, )) = ic::call::<_, (Result<(), XidCenterError>, ), _>(
        xid_center,
        "putID",
        (&simple_id, )
    ).await {
        match x {
            Ok(_) => {},
            Err(er) => {
                return match er {
                    XidCenterError::IDExist => {
                        Err(VerifyError::IDExist)
                    },
                    XidCenterError::XidNotExist => {
                        Err(VerifyError::XidNotExist)
                    },
                    _ => { Err(VerifyError::XidCNoNameErr) },
                };
            },
        }
    };
    STATE.with(|s| {
        let mut ids = s.ids.borrow_mut();
        let mut main_id = s.main_id.borrow_mut();
        if *main_id.identity == "".to_string() {
            *main_id = id.clone();
        };
        ids.insert(id);
    });
    Ok(XidResponse::VerifyOk)
}

#[update(name = "setXid", guard="is_authorized")]
#[candid_method(update, rename = "setXid")]
async fn set_xid(args : XidArgs) -> bool {
    STATE.with(|s| {
        match args.name {
            Some(n) => {
                *s.name.borrow_mut() = n;
            },
            None => {},
        };
        match args.avatar_url {
            Some(url) => {
                *s.avatar_url.borrow_mut() = url;
            },
            None => {},
        }
    });
    true
}

#[update(name = "uploadAvatar", guard="is_authorized")]
#[candid_method(update, rename = "uploadAvatar")]
async fn upload_avatar(avatar : Avatar) -> bool {
    STATE.with(|s| {
        *s.avatar.borrow_mut() = avatar;
    });
    true
}

#[update(name = "uploadStore", guard="is_authorized")]
#[candid_method(update, rename = "uploadStore")]
async fn upload_store(arg : StoreArg) -> Result<XidResponse, XidError> {
     STATE.with(|s| {
         let pub_key = s.pub_key.borrow();
         return match arg.content {
             Contents::TwitterContent(t) => {
                 let mut twitter_store = s.twitter_store.borrow_mut();
                 if twitter_store.contains_key(&arg.uuid) {
                     Err(XidError::UuidRepeat)
                 } else {
                     twitter_store.insert(arg.uuid.clone(), TwitterStorage {
                         owner: pub_key.clone(),
                         uuid: arg.uuid,
                         twitter_content: t,
                         d_platform: arg.d_platform,
                         is_minted: false,
                         mint_time: "".to_string(),
                         upload_time: ic_cdk::api::time().to_string(),
                     });
                     Ok(XidResponse::StoreOk)
                 }
             },
             Contents::OffChainContent(o) => {
                 let mut off_store = s.off_store.borrow_mut();
                 if off_store.contains_key(&arg.uuid) {
                     Err(XidError::UuidRepeat)
                 } else {
                     off_store.insert(arg.uuid.clone(), OffStorage {
                         owner: pub_key.clone(),
                         uuid: arg.uuid,
                         off_content: o,
                         d_platform: arg.d_platform,
                         is_minted: false,
                         mint_time: "".to_string(),
                         upload_time: ic_cdk::api::time().to_string(),
                     });
                     Ok(XidResponse::StoreOk)
                 }
             }
         };
    })
}

#[update(name = "deleteStore", guard="is_authorized")]
#[candid_method(update, rename = "deleteStore")]
async fn delete_store(arg : ContentUuid) -> Result<XidResponse, XidError> {
    STATE.with(|s|{
        return match arg.content_type {
            ContentType::OffChain => {
                let mut off_store = s.off_store.borrow_mut();
                match off_store.remove(&arg.uuid) {
                    Some(_) => {
                        Ok(XidResponse::DeleteOk)
                    },
                    None => {
                        Err(XidError::UuidNotExist)
                    },
                }
            },
            ContentType::Twitter => {
                let mut twitter_store = s.twitter_store.borrow_mut();
                match twitter_store.remove(&arg.uuid) {
                    Some(_) => {
                        Ok(XidResponse::DeleteOk)
                    },
                    None => {
                        Err(XidError::UuidNotExist)
                    },
                }
            }
        };
    })
}

#[update(name = "setMintStatus", guard="is_authorized")]
#[candid_method(update, rename = "setMintStatus")]
async fn set_mint_status(arg : ContentUuid) -> Result<XidResponse, XidError> {
    STATE.with(|s| {
        match arg.content_type {
            ContentType::OffChain => {
                let mut off_store = s.off_store.borrow_mut();
                match off_store.get(&arg.uuid) {
                    Some(os) => {
                        let res = os.clone();
                        off_store.insert( arg.uuid, OffStorage{
                            owner: res.owner,
                            uuid: res.uuid,
                            off_content: res.off_content,
                            d_platform: res.d_platform,
                            is_minted: true,
                            mint_time: ic_cdk::api::time().to_string(),
                            upload_time: res.upload_time,
                        })
                    },
                    None => { return Err(XidError::UuidNotExist) },
                };
            },
            ContentType::Twitter => {
                let mut twitter_store = s.twitter_store.borrow_mut();
                match twitter_store.get(&arg.uuid) {
                    Some(ts) => {
                        let res = ts.clone();
                        twitter_store.insert( arg.uuid, TwitterStorage{
                            owner: res.owner,
                            uuid: res.uuid,
                            twitter_content: res.twitter_content,
                            d_platform: res.d_platform,
                            is_minted: true,
                            mint_time: ic_cdk::api::time().to_string(),
                            upload_time: res.upload_time,
                        })
                    },
                    None => { return Err(XidError::UuidNotExist) }
                };
            }
        };
        return Ok(XidResponse::MintOk)
    })
}

fn is_authorized() -> Result<(), String> {
    STATE.with(|s| {
        if *s.pub_key.borrow() == caller().to_text() {
            Ok(())
        } else {
            Err("Caller is not authorized".to_string())
        }
    })
}

fn is_ic_authorized() -> Result<(), String> {
    STATE.with(|s| {
        if *s.ic_verify.borrow() == caller().to_text() {
            Ok(())
        } else {
            Err("Caller is not authorized".to_string())
        }
    })
}

fn do_clear() {
    STATE.with(|s| {
        s.pub_key.borrow_mut().clear();
        s.name.borrow_mut().clear();
        s.ids.borrow_mut().clear();
        s.ic_verify.borrow_mut().clear();
        s.avatar_url.borrow_mut().clear();
        s.twitter_store.borrow_mut().clear();
        s.off_store.borrow_mut().clear();
    })
}

#[pre_upgrade]
fn pre_upgrade() {
    let stable_state : StableState = STATE.with(|s| StableState{
        pub_key: s.pub_key.take(),
        name: s.name.take(),
        main_id: s.main_id.take(),
        ids: s.ids.take(),
        ic_verify: s.ic_verify.take(),
        avatar_url: s.avatar_url.take(),
        avatar: s.avatar.take(),
        twitter_store: s.twitter_store.take(),
        off_store: s.off_store.take(),
    });
    ic::stable_store((stable_state, )).expect("failed to save stable state");
}

#[post_upgrade]
fn post_upgrade() {
    do_clear();
    let (stable_state,) : (StableState, ) =
        ic::stable_restore().expect("failed to restore stable state");

    STATE.with(|s| {
        s.pub_key.replace(stable_state.pub_key);
        s.name.replace(stable_state.name);
        s.main_id.replace(stable_state.main_id);
        s.ids.replace(stable_state.ids);
        s.ic_verify.replace(stable_state.ic_verify);
        s.avatar_url.replace(stable_state.avatar_url);
        s.avatar.replace(stable_state.avatar);
        s.twitter_store.replace(stable_state.twitter_store);
        s.off_store.replace(stable_state.off_store);
    })
}

