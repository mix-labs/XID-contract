import Array "mo:base/Array";
import Blob "mo:base/Blob";
import Cycles "mo:base/ExperimentalCycles";
import Hash "mo:base/Hash";
import HashMap "mo:base/HashMap";
import Nat "mo:base/Nat";
import Nat64 "mo:base/Nat64";
import Principal "mo:base/Principal";
import Result "mo:base/Result";
import Iter "mo:base/Iter";
import TrieMap "mo:base/TrieMap";
import TrieSet "mo:base/TrieSet";
import Types "Types";
import Trie "mo:base/Trie";
import Text "mo:base/Text";
import Logs "Logs";
import Prim "mo:⛔";
import Account "lib/Account";
import Ledger "lib/Ledger";
import CMC "lib/Cmc";

shared(installer) actor class XIDC() = this{
    type RustResult<Ok, Err> = {
        #Ok : Ok;
        #Err : Err;
    };
    type simpleId       = Types.simpleId;
    type TopUpArgs      = Types.TopUpArgs;
    type Error          = Types.Error;
    type XidCenterError = Types.XidCenterError;
    type UpgradeXidArgs = Types.UpgradeXidArgs;
    type UpdateWasmArgs = Types.UpdateWasmArgs;

    let management : Types.Management = actor ("aaaaa-aa");
    let ledger : Ledger.Ledger = actor ("ryjl3-tyaaa-aaaaa-aaaba-cai");
    let cmc : CMC.CMC = actor ("rkp4c-7iaaa-aaaaa-aaaca-cai");
    let CYCLE_MINTING_CANISTER = Principal.fromText("rkp4c-7iaaa-aaaaa-aaaca-cai");
    let LEDGER_TRANSFER_FEE = 10_000 : Nat64;
    let TOP_UP_CANISTER_MEMO = 0x50555054 : Nat64;

    stable var xid_version = 0;
    stable var admins = TrieSet.fromArray<Principal>([Principal.fromText("77owi-ydjey-cht3l-nifhw-xkeio-jalgg-bikxb-kl6qa-ccciy-ztlm3-eqe")], Principal.hash, Principal.equal);
    stable var prin_xids_entries : [(Principal, Principal)] = [];
    stable var xid_prin_entries : [(Principal, Principal)] = [];
    stable var ic_xids_entries : [(Principal, Principal)] = [];
    stable var eth_xids_entries : [(Text, Principal)] = [];
    stable var aptos_xids_entries : [(Text, Principal)] = [];
    stable var twitter_xids_entries : [(Text, Principal)] = [];
    stable var bucket_upgrade_params : (Nat, [(Nat,(Nat64, Nat))]) = (0, []);
    stable var log_index = 0;
    stable var xid_wasm : [Nat8] = [];

    var prin_xids : TrieMap.TrieMap<Principal, Principal> = TrieMap.fromEntries<Principal, Principal>(prin_xids_entries.vals(), Principal.equal, Principal.hash);
    var xid_prin : TrieMap.TrieMap<Principal, Principal> = TrieMap.fromEntries<Principal, Principal>(xid_prin_entries.vals(), Principal.equal, Principal.hash);
    var ic_xids : TrieMap.TrieMap<Principal, Principal> = TrieMap.fromEntries<Principal, Principal>(ic_xids_entries.vals(), Principal.equal, Principal.hash);
    var twitter_xids : TrieMap.TrieMap<Text, Principal> = TrieMap.fromEntries<Text, Principal>(twitter_xids_entries.vals(), Text.equal, Text.hash);
    var eth_xids : TrieMap.TrieMap<Text, Principal> = TrieMap.fromEntries<Text, Principal>(eth_xids_entries.vals(), Text.equal, Text.hash);
    var aptos_xids : TrieMap.TrieMap<Text, Principal> = TrieMap.fromEntries<Text, Principal>(aptos_xids_entries.vals(), Text.equal, Text.hash);
    var logs = Logs.Logs(true);

    // 获取xid最新版本
    public query({caller}) func getXidVersion() : async Nat {
        xid_version
    };

    // 获取权限组
    public query func getAdmins(): async [Principal] { TrieSet.toArray(admins) };

    public query func getXidCidByIdentity(id : simpleId) : async Result.Result<Principal, XidCenterError> {
        switch (id.platform) {
            case ("ethereum") {
                switch (eth_xids.get(id.identity)) {
                    case (?xid) { #ok(xid) };
                    case (null) { #err(#XidNotExist) };
                };
            };
            case ("ic") {
                switch (ic_xids.get(Principal.fromText(id.identity))) {
                    case (?xid) { #ok(xid) };
                    case (null) { #err(#XidNotExist) };
                };
            };
            case ("aptos") {
                switch (aptos_xids.get(id.identity)) {
                    case (?xid) { #ok(xid) };
                    case (null) { #err(#XidNotExist) };
                };
            };
            case ("twitter") {
                switch (twitter_xids.get(id.identity)) {
                    case (?xid) { #ok(xid) };
                    case (null) { #err(#XidNotExist) };
                };
            };
            case (_) { #err(#Invalid_Platform)};
        };
    };

    // 通过pub_key Principal获取xid
    public query func getXidCidByPub(prin : Principal) : async Result.Result<Principal, XidCenterError> {
        switch (prin_xids.get(prin)) {
            case (null) { #err(#XidNotExist) };
            case (?xid_canister_id) {
                #ok(xid_canister_id)
            };
        }
    };

    // 获取日志
    public query({caller}) func getLog() : async [(Nat, Text)]{
        if(not TrieSet.mem<Principal>(admins, caller, Principal.hash(caller), Principal.equal)) return [];
        let res = Array.init<(Nat, Text)>(log_index, (0, ""));
        var index = 0;
        for(l in res.vals()){
            res[index] := logs.get(index);
            index += 1;
        };
        Array.freeze<(Nat, Text)>(res)
    };

    public shared({caller}) func deleteID(id : simpleId) : async RustResult<(), XidCenterError> {
        switch (xid_prin.get(caller)) {
            case (null) { return #Err(#XidNotExist) };
            case (?prin) {
                switch (id.platform) {
                    case ("ethereum") {
                        switch (eth_xids.get(id.identity)) {
                            case (?xid) {
                                eth_xids.delete(id.identity);
                            };
                            case (null) {
                                return #Err(#IDNotExist)
                            };
                        };
                    };
                    case ("ic") {
                        switch (ic_xids.get(Principal.fromText(id.identity))) {
                            case (?xid) {
                                ic_xids.delete(Principal.fromText(id.identity));
                            };
                            case (null) {
                                return #Err(#IDNotExist)
                            };
                        };
                    };
                    case ("aptos") {
                        switch (aptos_xids.get(id.identity)) {
                            case (?xid) {
                                aptos_xids.delete(id.identity);
                            };
                            case (null) {
                                return #Err(#IDNotExist)
                            };
                        };
                    };
                    case ("twitter") {
                        switch (twitter_xids.get(id.identity)) {
                            case (?xid) {
                                twitter_xids.delete(id.identity);
                            };
                            case (null) {
                                return #Err(#IDNotExist)
                            };
                        };
                    };
                    case (_) { return #Err(#Invalid_Platform)};
                };
            };
        };
        #Ok(())
    };

    public shared({caller}) func putID(id : simpleId) : async RustResult<(), XidCenterError> {
        switch (xid_prin.get(caller)) {
            case (null) { return #Err(#XidNotExist) };
            case (?prin) {
                switch (id.platform) {
                    case ("ethereum") {
                        switch (eth_xids.get(id.identity)) {
                            case (?xid) { return #Err(#IDExist) };
                            case (null) { 
                                eth_xids.put(id.identity, caller);
                            };
                        };
                    };
                    case ("ic") {
                        switch (ic_xids.get(Principal.fromText(id.identity))) {
                            case (?xid) { return #Err(#IDExist) };
                            case (null) { 
                                ic_xids.put(Principal.fromText(id.identity), caller);
                            };
                        };
                    };
                    case ("aptos") {
                        switch (aptos_xids.get(id.identity)) {
                            case (?xid) { return #Err(#IDExist) };
                            case (null) { 
                                aptos_xids.put(id.identity, caller);
                            };
                        };
                    };
                    case ("twitter") {
                        switch (twitter_xids.get(id.identity)) {
                            case (?xid) { return #Err(#IDExist) };
                            case (null) { 
                                twitter_xids.put(id.identity, caller);
                            };
                        };
                    }; 
                    case (_) { return #Err(#Invalid_Platform)};
                };
            };
        };
        #Ok(())
    };

    // 更新xid最新版本
    public shared({caller}) func updateXidVersion(version : Nat) : async Bool {
        if (not _authorized(caller)) return false;
        xid_version := version;
        true
    };

    // 清空日志
    public shared({caller}) func clearLog() : async (){
        if(TrieSet.mem<Principal>(admins, caller, Principal.hash(caller), Principal.equal)){
            logs.clear();
            log_index := 0;
        }
    };

    // 创建xid
    public shared({caller}) func createXid() : async Principal{
        assert(prin_xids.get(caller) == null);
        Cycles.add(200000000000); // 0.2 T
        let cid = (await management.create_canister({ settings = null; })).canister_id;
        ignore management.install_code({
            arg = Blob.toArray(to_candid(caller));
            wasm_module = xid_wasm;
            mode = #install;
            canister_id = cid;
        });
        prin_xids.put(caller, cid);
        xid_prin.put(cid, caller);
        ignore _addLog(
            "Create Xid Successfully : "
            # " \n Canister Id : \n "
            # debug_show(cid)
            # " \n Owner : \n "
            # debug_show(caller)
        );
        cid
    };

    public shared({caller}) func upgradeXid(args : UpgradeXidArgs) : async Result.Result<(), XidCenterError>{
        var wasm : [Nat8] = [];
        switch(prin_xids.get(caller)){
            case null { return #err(#Invalid_Operation) };
            case(?xid_canister_id){
                if (xid_canister_id != args.canister_id) {
                    return #err(#NotXidOwner);
                }; 
                wasm := xid_wasm 
            }
        };
        ignore management.install_code({
            arg = Blob.toArray(to_candid(caller));
            wasm_module = wasm;
            mode = #upgrade;
            canister_id = args.canister_id;
        });
        #ok(())
    };

    // 用于各个wasm的更新
    public shared({caller}) func updateXidWasm(args: UpdateWasmArgs): async Result.Result<Text, Text> {
        if (not _authorized(caller)) return #err("permission denied");
        ignore _addLog("Update Xid Wasm, Caller: " # debug_show(caller) # "Time: " # debug_show(Prim.time()));
        xid_wasm := args.wasm;
        #ok("update xid successfully")
    };

    // top up box
    public shared({caller}) func topUpXid(
        args : TopUpArgs
    ) : async Result.Result<(), Error>{
        assert(args.icp_amount > 2_000_000);
        let subaccount = Blob.fromArray(Account.principalToSubAccount(caller));
        let cycle_subaccount = Blob.fromArray(Account.principalToSubAccount(args.c_id));
        let cycle_ai = Account.accountIdentifier(CYCLE_MINTING_CANISTER, cycle_subaccount);
        switch(await ledger.transfer({
            to = cycle_ai;
            fee = { e8s = LEDGER_TRANSFER_FEE };
            memo = TOP_UP_CANISTER_MEMO;
            from_subaccount = ?subaccount;
            amount = { e8s = args.icp_amount - 1_010_000 };
            created_at_time = null;
        })){
            case(#Err(e)){
                #err(#LedgerTransferError(_addLog(
                    "Transfer ICP Error : Error Info : "
                    # debug_show(e)
                    # " \n icp amount : \n "
                    # debug_show(args.icp_amount)
                    # " \n Caller : \n "
                    # debug_show(caller)
                    # " \n Time : \n "
                    # debug_show(Prim.time())
                )))
            };
            case(#Ok(height)){
                ignore cmc.notify_top_up(
                    {
                        block_index = height;
                        canister_id = args.c_id;
                    }
                );
                #ok(())
            }
        }
    };

     // 改变权限组
    public shared({caller}) func changeAdmin(_admins: [Principal]): async Bool {
        if(not _authorized(caller)) return false;
        admins := TrieSet.fromArray(_admins, Principal.hash, Principal.equal);
        true
    };

    // 添加权限组成员
    public shared({caller}) func addAdmin(newAdmin: Principal): async Bool {
        if(not _authorized(caller)) return false;
        admins := TrieSet.put(admins, newAdmin, Principal.hash(newAdmin), Principal.equal);
        true
    };

    private func _authorized(caller : Principal) : Bool {
        if(not TrieSet.mem<Principal>(admins, caller, Principal.hash(caller), Principal.equal)) return false;
        true
    };

    private func _addLog(log : Text) : Nat{
        let id = log_index;
        ignore logs.put(id, log);
        log_index += 1;
        id
    };

    system func preupgrade() {
        prin_xids_entries := do {
            let res = Array.init<(Principal, Principal)>(prin_xids.size(), (
                Principal.fromActor(this),
                Principal.fromActor(this)
            ));
            var index = 0;
            for (p in prin_xids.entries()) {
                res[index] := p;
                index += 1;
            };
            Array.freeze<(Principal, Principal)>(res)
        };

        xid_prin_entries := do {
            let res = Array.init<(Principal, Principal)>(xid_prin.size(), (
                Principal.fromActor(this),
                Principal.fromActor(this)
            ));
            var index = 0;
            for (x in xid_prin.entries()) {
                res[index] := x;
                index += 1;
            };
            Array.freeze<(Principal, Principal)>(res)
        };

        ic_xids_entries := do {
            let res = Array.init<(Principal, Principal)>(ic_xids.size(), (
                Principal.fromActor(this),
                Principal.fromActor(this)
            ));
            var index = 0;
            for (i in ic_xids.entries()) {
                res[index] := i;
                index += 1;
            };
            Array.freeze<(Principal, Principal)>(res)
        };

        twitter_xids_entries := do {
            let res = Array.init<(Text, Principal)>(twitter_xids.size(), (
                "",
                Principal.fromActor(this)
            ));
            var index = 0;
            for (t in twitter_xids.entries()) {
                res[index] := t;
                index += 1;
            };
            Array.freeze<(Text, Principal)>(res)
        };

        eth_xids_entries := do {
            let res = Array.init<(Text, Principal)>(eth_xids.size(), (
                "",
                Principal.fromActor(this)
            ));
            var index = 0;
            for (e in eth_xids.entries()) {
                res[index] := e;
                index += 1;
            };
            Array.freeze<(Text, Principal)>(res)
        };

        aptos_xids_entries := do {
            let res = Array.init<(Text, Principal)>(aptos_xids.size(), (
                "",
                Principal.fromActor(this)
            ));
            var index = 0;
            for (a in aptos_xids.entries()) {
                res[index] := a;
                index += 1;
            };
            Array.freeze<(Text, Principal)>(res)
        };

        bucket_upgrade_params := logs.preupgrade();
    };

    system func postupgrade() {
        prin_xids_entries := [];
        xid_prin_entries := [];
        ic_xids_entries := [];
        eth_xids_entries := [];
        aptos_xids_entries := [];
        twitter_xids_entries := [];
        logs.postupgrade(bucket_upgrade_params);
        bucket_upgrade_params := (0, []);
    };

};