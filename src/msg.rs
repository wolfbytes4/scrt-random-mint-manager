use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{
   Addr, Uint128, Binary
}; 
use secret_toolkit::{ 
    snip721:: { Trait, ViewerInfo }
};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct InstantiateMsg { 
    //nft contract information
    pub mint_contract: ContractInfo,
    pub shill_contract: ContractInfo,
    pub scrt_contract: ContractInfo,
    pub entropy_shill: String,
    pub entropy_mint: String
}
 
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct ContractInfo {
    /// contract's code hash string
    pub code_hash: String,
    /// contract's address
    pub address: Addr,
    pub mint_cost: u128
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct PreLoad {
    pub id: String,
    pub img_url: String,
    pub attributes: Option<Vec<Trait>>
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg { 
    Receive{ 
        sender: Addr,
        from: Addr,
        amount: Uint128,
        msg: Option<Binary>
    },  
    PreLoad{
        new_data: Vec<PreLoad>
    },
    SetViewingKey{
        key: String
    }
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleReceiveMsg {
    ReceiveMintScrt {},
    ReceiveMintShill {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {  
    GetMintInfo {viewer: ViewerInfo}
}
 

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct MintInfoResponse {
    pub num_minted: u16,
    pub total: u16, 
    pub amount_paid_shill: Uint128,
    pub amount_paid_scrt: Uint128,
}