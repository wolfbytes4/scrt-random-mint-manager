use schemars::JsonSchema;
use serde::{ Deserialize, Serialize};

use cosmwasm_std::{CanonicalAddr, Uint128};

use secret_toolkit::{ 
    storage:: { Item, Keymap },
    snip721:: {ViewerInfo}
};
use crate::msg::{ContractInfo, PreLoad};

pub static CONFIG_KEY: &[u8] = b"config"; 
pub const ADMIN_KEY: &[u8] = b"admin";
pub const MY_ADDRESS_KEY: &[u8] = b"my_address"; 
pub const PRE_LOAD_KEY: &[u8] = b"pre_load"; 
pub const ADMIN_VIEWING_KEY: &[u8] = b"admin_viewing_key";

pub static CONFIG_ITEM: Item<State> = Item::new(CONFIG_KEY); 
pub static ADMIN_ITEM: Item<CanonicalAddr> = Item::new(ADMIN_KEY); 
pub static MY_ADDRESS_ITEM: Item<CanonicalAddr> = Item::new(MY_ADDRESS_KEY); 
pub static PRE_LOAD_STORE: Keymap<u16, PreLoad> = Keymap::new(PRE_LOAD_KEY);
pub static ADMIN_VIEWING_KEY_ITEM: Item<ViewerInfo> = Item::new(ADMIN_VIEWING_KEY);

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct State {  
    pub owner: CanonicalAddr, 
    pub num_minted: u16,
    pub total: u16, 
    pub amount_paid_shill: Uint128,
    pub amount_paid_scrt: Uint128,
    pub viewing_key: Option<String>,
    pub shill_contract: ContractInfo,
    pub scrt_contract: ContractInfo,
    pub mint_contract: ContractInfo,
    pub entropy_mint: String
}