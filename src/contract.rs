use cosmwasm_std::{
    entry_point, Env, DepsMut, Deps,
    MessageInfo, Response, StdError, Addr,
    Binary, Uint128, CosmosMsg, from_binary, to_binary, StdResult
};
use crate::error::ContractError;
use crate::msg::{MintInfoResponse, ExecuteMsg, InstantiateMsg, QueryMsg, PreLoad, ContractInfo, HandleReceiveMsg};
use crate::state::{ State, PRE_LOAD_STORE, CONFIG_ITEM, ADMIN_VIEWING_KEY_ITEM, ADMIN_ITEM, MY_ADDRESS_ITEM };
use secret_toolkit::{
    snip721::{
        mint_nft_msg, set_viewing_key_msg, Authentication, MediaFile, Extension, Metadata, ViewerInfo
    }
};  
use crate::rand::{extend_entropy, Prng, sha_256};
pub const BLOCK_SIZE: usize = 256;
 
#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg
) -> Result<Response, StdError> {
    // create initial state
    let state = State { 
        owner: deps.api.addr_canonicalize(&info.sender.to_string())?,  
        num_minted: 0,
        total: 0, 
        amount_paid_shill: Uint128::from(0u32),
        amount_paid_scrt: Uint128::from(0u32), 
        entropy_mint: msg.entropy_mint,
        viewing_key: Some(msg.entropy_shill),
        shill_contract: msg.shill_contract,
        scrt_contract: msg.scrt_contract,
        mint_contract: msg.mint_contract
    };

    //Save Contract state
    CONFIG_ITEM.save(deps.storage, &state)?;
    ADMIN_ITEM.save(deps.storage, &deps.api.addr_canonicalize(&info.sender.to_string())?)?;
    MY_ADDRESS_ITEM.save(deps.storage,  &deps.api.addr_canonicalize(&_env.contract.address.to_string())?)?;
 
    let mut response_msgs: Vec<CosmosMsg> = Vec::new();
   
    response_msgs.push(
        set_viewing_key_msg(
            state.viewing_key.clone().unwrap().to_string(),
            None,
            BLOCK_SIZE,
            state.shill_contract.code_hash,
            state.shill_contract.address.to_string(),
        )?
    );
    response_msgs.push(
        set_viewing_key_msg(
            state.viewing_key.clone().unwrap().to_string(),
            None,
            BLOCK_SIZE,
            state.scrt_contract.code_hash,
            state.scrt_contract.address.to_string(),
        )?
    );
    response_msgs.push(
        set_viewing_key_msg(
            state.viewing_key.clone().unwrap().to_string(),
            None,
            BLOCK_SIZE,
            state.mint_contract.code_hash,
            state.mint_contract.address.to_string(),
        )?
    );
   
    deps.api.debug(&format!("Contract was initialized by {}", info.sender));
     
    Ok(Response::new().add_messages(response_msgs))
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg
) -> Result<Response, ContractError> {
    match msg { 
        ExecuteMsg::Receive {
            sender,
            from,
            amount,
            msg
        } => receive(deps, _env, &sender, &from, amount, msg),
        ExecuteMsg::PreLoad { new_data } => pre_load(deps, &info.sender, new_data),
        ExecuteMsg::SetViewingKey { key } => try_set_viewing_key(
            deps,
            _env, 
            &info.sender,
            key
        )
    }
} 

pub fn try_set_viewing_key(
    deps: DepsMut,
    _env: Env,
    sender: &Addr,
    key: String
) -> Result<Response, ContractError> {
    let state = CONFIG_ITEM.load(deps.storage)?;
    let prng_seed: Vec<u8> = sha_256(base64::encode(key).as_bytes()).to_vec();
    let viewing_key = base64::encode(&prng_seed);

    let vk: ViewerInfo = { ViewerInfo {
        address: sender.to_string(),
        viewing_key: viewing_key,
    } };

    let sender_raw = &deps.api.addr_canonicalize(&sender.to_string())?;
    if sender_raw == &state.owner {
        ADMIN_VIEWING_KEY_ITEM.save(deps.storage, &vk)?;
    }  
 
    Ok(Response::default())
}

fn receive(
    deps: DepsMut,
    _env: Env,
    sender: &Addr,
    from: &Addr,
    amount: Uint128,
    msg: Option<Binary>
) -> Result<Response, ContractError> { 
    deps.api.debug(&format!("Receive received"));
    let mut state = CONFIG_ITEM.load(deps.storage)?;

    if sender == &state.scrt_contract.address {
        if amount.u128() != state.scrt_contract.mint_cost {
            return Err(ContractError::CustomError {val: "You have attempted to send the wrong amount of tokens".to_string()});
        }
        state.amount_paid_scrt = state.amount_paid_scrt + amount;
    }
    else if sender == &state.shill_contract.address {
        if amount.u128() != state.shill_contract.mint_cost {
            return Err(ContractError::CustomError {val: "You have attempted to send the wrong amount of tokens".to_string()});
        }
        state.amount_paid_shill = state.amount_paid_shill + amount;
    }
    else{
        return Err(ContractError::CustomError {val: "Address is not correct snip contract".to_string()});
    }

    CONFIG_ITEM.save(deps.storage, &state)?;

    if let Some(bin_msg) = msg {
        match from_binary(&bin_msg)? {
            HandleReceiveMsg::ReceiveMintScrt {} => mint(
                _env,
                deps,
                sender,
                from
            ),
            HandleReceiveMsg::ReceiveMintShill {} => mint(
                _env,
                deps,
                sender,
                from
            ),
        }
    } else {
        return Err(ContractError::CustomError {val: "data should be given".to_string()});
    }
}

fn pre_load(
    deps: DepsMut, 
    sender: &Addr, 
    new_data: Vec<PreLoad>
) -> Result<Response, ContractError> {
    
    let mut state = CONFIG_ITEM.load(deps.storage)?;

    let sender_raw = &deps.api.addr_canonicalize(&sender.to_string())?;

    if &state.owner != sender_raw {
        return Err(ContractError::CustomError {val: "You are not allowed to use this function".to_string()});
    }

    for data in new_data.iter() {
        state.total = state.total + 1; 
        PRE_LOAD_STORE.insert(deps.storage, &state.total, &data)?;
    }

    CONFIG_ITEM.save(deps.storage, &state)?;
    Ok(Response::default())
}

pub fn mint(
    _env: Env,
    deps: DepsMut,
    sender: &Addr,
    from: &Addr,
) -> Result<Response, ContractError> {
    let mut state = CONFIG_ITEM.load(deps.storage)?;

    // Checks how many tokens are left
    if state.total == 0 {
        return Err(ContractError::CustomError {val: "All tokens have been minted".to_string()}); 
    }

    // Pull random token data for minting then remove from data pool
    let rng_entropy = extend_entropy(&_env, state.entropy_mint.as_ref(), &sender);
    let mut rng = Prng::new(state.entropy_mint.as_ref(), &rng_entropy);
    let num = (rng.next_u32() % (state.total as u32)) as u16 + 1; // an id number between 1 and count

    let token_data: PreLoad = PRE_LOAD_STORE.get(deps.storage,&num)
                                            .ok_or_else(|| StdError::generic_err("Token ID pool is corrupt"))?;
    
    state.total = state.total - 1;

    PRE_LOAD_STORE.remove(deps.storage, &num)?;
    CONFIG_ITEM.save(deps.storage, &state)?;

    let public_metadata = Some(Metadata {
        token_uri: None,
        extension: Some(Extension {
            image: None,
            image_data: None,
            external_url: None,
            description: Some("This bone is imbued with special magic power. Feed it to your wolf and watch something amazing happen!".to_string()),
            name: Some("Magic Bone #".to_string() + &token_data.id.to_string()),
            attributes: token_data.attributes,
            background_color: None,
            animation_url: None,
            youtube_url: None,
            media: Some(vec![MediaFile {
                file_type: Some("image".to_string()),
                extension: Some("png".to_string()),
                url: String::from(token_data.img_url),
                authentication: Some(Authentication {
                    key: None,
                    user: None,
                }),
            }]),
            protected_attributes: None,
        }),
    });

    let private_metadata = None;
    let memo = None;
    let padding = None;
    let token_id: Option<String> = Some(token_data.id.clone());

    //add message to send funds to owner
    Ok(Response::new()
        .add_message(mint_nft_msg(
            token_id,
            Some(from.to_string()),
            public_metadata,
            private_metadata,
            memo,
            padding,
            BLOCK_SIZE,
            state.mint_contract.code_hash.to_string(),
            state.mint_contract.address.to_string()
        )?)
    )
}



#[entry_point]
pub fn query(
    deps: Deps,
    _env: Env,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {  
        QueryMsg::GetMintInfo {viewer} => to_binary(&query_mint_info(deps, viewer)?),  
    }
}
  

fn query_mint_info( 
    deps: Deps,
    viewer: ViewerInfo
) -> StdResult<MintInfoResponse> { 
    check_admin_key(deps, viewer)?;
    let state = CONFIG_ITEM.load(deps.storage)?; 
    Ok(MintInfoResponse { num_minted: state.num_minted, total: state.total, amount_paid_shill: state.amount_paid_shill, amount_paid_scrt: state.amount_paid_scrt })
} 

fn check_admin_key(deps: Deps, viewer: ViewerInfo) -> StdResult<()> {
    let admin_viewing_key = ADMIN_VIEWING_KEY_ITEM.load(deps.storage)?;  
    let prng_seed: Vec<u8> = sha_256(base64::encode(viewer.viewing_key).as_bytes()).to_vec();
    let vk = base64::encode(&prng_seed);

    if vk != admin_viewing_key.viewing_key || viewer.address != admin_viewing_key.address{
        return Err(StdError::generic_err(
            "Wrong viewing key for this address or viewing key not set",
        )); 
    }

    return Ok(());
}