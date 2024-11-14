use near_sdk::{serde_json, AccountId, NearToken};
use near_workspaces::{error::Error, network::Sandbox, Contract, Worker};

pub async fn deploy_contract(
    sandbox: &Worker<Sandbox>,
    mpc_contract_id: &AccountId,
) -> Result<Contract, Error> {
    let wasm = near_workspaces::compile_project("./").await?;

    let contract = sandbox
        .root_account()?
        .create_subaccount("proxy")
        .initial_balance(NearToken::from_near(10))
        .transact()
        .await?
        .result
        .deploy(&wasm)
        .await?
        .result;

    let _ = contract
        .call("new")
        .args_json(serde_json::json!({ "mpc_contract_id": mpc_contract_id }))
        .transact()
        .await?;

    Ok(contract)
}

pub async fn deploy_mpc_contract(sandbox: &Worker<Sandbox>) -> Result<Contract, Error> {
    let wasm = std::fs::read("./tests/mpc_contract.wasm").expect("No MPC contract Wasm file");

    let contract = sandbox
        .root_account()?
        .create_subaccount("mpc")
        .initial_balance(NearToken::from_near(20))
        .transact()
        .await?
        .result
        .deploy(&wasm)
        .await?
        .result;

    let _ = contract
    .call("init_running")
    .args_json(serde_json::json!({
        "epoch": 0,
        "threshold": 2,
        "participants": {
            "next_id": 3,
            "participants": {
                "1.near": {
                    "account_id": "1.near",
                    "url": "127.0.0.1",
                    "cipher_pk": [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                    "sign_pk": "ed25519:2Y9Rz7ri9Js4jC3UagR226fNDaFDLRYrR3AX2edBR41r"
                },
                "2.near": {
                    "account_id": "2.near",
                    "url": "127.0.0.1",
                    "cipher_pk": [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                    "sign_pk": "ed25519:2Y9Rz7ri9Js4jC3UagR226fNDaFDLRYrR3AX2edBR41r"
                },
                "3.near": {
                    "account_id": "3.near",
                    "url": "127.0.0.1",
                    "cipher_pk": [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                    "sign_pk": "ed25519:2Y9Rz7ri9Js4jC3UagR226fNDaFDLRYrR3AX2edBR41r"
                },
            },
            "account_to_participant_id": {
                "1.near": 0,
                "2.near": 1,
                "3.near": 2
            }
        },
        "public_key": "secp256k1:54hU5wcCmVUPFWLDALXMh1fFToZsVXrx9BbTbHzSfQq1Kd1rJZi52iPa4QQxo6s5TgjWqgpY8HamYuUDzG6fAaUq"
    }))
    .transact()
    .await?;

    Ok(contract)
}

pub fn create_sign_args() -> Vec<u8> {
    serde_json::json!({
      "request": {
        "payload": [
          58,
          80,
          27,
          38,
          189,
          233,
          121,
          77,
          218,
          12,
          253,
          237,
          251,
          192,
          47,
          12,
          99,
          253,
          203,
          122,
          173,
          44,
          11,
          7,
          247,
          114,
          33,
          115,
          173,
          86,
          7,
          49
        ],
        "path": "arbitrum-1",
        "key_version": 0
      }
    })
    .to_string()
    .into_bytes()
}
