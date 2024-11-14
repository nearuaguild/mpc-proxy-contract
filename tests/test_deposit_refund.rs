mod common;

use common::{create_sign_args, deploy_contract, deploy_mpc_contract};
use near_sdk::serde_json;
use near_workspaces::types::{Gas, NearToken};

#[tokio::test]
async fn test_deposit_is_refunded_after_timeout() -> Result<(), Box<dyn std::error::Error>> {
    let sandbox = near_workspaces::sandbox().await?;

    let mpc = deploy_mpc_contract(&sandbox).await?;
    let contract = deploy_contract(&sandbox, mpc.id()).await?;

    let user_account = sandbox.dev_create_account().await?;

    let _ = user_account
        .call(contract.id(), "deposit")
        .deposit(NearToken::from_near(1))
        .transact()
        .await?;

    let sign_tx = user_account
        .call(contract.id(), "sign")
        .deposit(NearToken::from_yoctonear(0))
        .args_json(serde_json::json!({
            "sign_args": create_sign_args(),
            "deposit": NearToken::from_yoctonear(5)
        }))
        .gas(Gas::from_tgas(260))
        .transact_async()
        .await?;

    // wait for a few blocks to pass by
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;

    // must fail due to "Signature request has timed out"
    let _ = sign_tx.await?;

    let balance = user_account
        .call(contract.id(), "available_deposit")
        .args_json(serde_json::json!({"account_id": user_account.id()}))
        .view()
        .await?
        .json::<NearToken>()?;

    assert_eq!(
        balance.as_yoctonear(),
        NearToken::from_near(1).as_yoctonear()
    );

    Ok(())
}

#[tokio::test]
async fn test_deposit_is_refunded_on_duplicated_request() -> Result<(), Box<dyn std::error::Error>>
{
    let sandbox = near_workspaces::sandbox().await?;

    let mpc = deploy_mpc_contract(&sandbox).await?;
    let contract = deploy_contract(&sandbox, mpc.id()).await?;

    let user_account = sandbox.dev_create_account().await?;

    let _ = user_account
        .call(contract.id(), "deposit")
        .deposit(NearToken::from_near(1))
        .transact()
        .await?;

    let sign_tx = user_account
        .call(contract.id(), "sign")
        .deposit(NearToken::from_yoctonear(0))
        .args_json(serde_json::json!({
            "sign_args": create_sign_args(),
            "deposit": NearToken::from_yoctonear(5)
        }))
        .gas(Gas::from_tgas(260))
        .transact_async()
        .await?;

    // wait for a few blocks to pass by
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    // must fail due to "Signature request has already been submitted"
    let _ = user_account
        .call(contract.id(), "sign")
        .deposit(NearToken::from_yoctonear(0))
        .args_json(serde_json::json!({
            "sign_args": create_sign_args(),
            "deposit": NearToken::from_yoctonear(5)
        }))
        .gas(Gas::from_tgas(260))
        .transact()
        .await?;

    // must fail due to "Signature request has timed out"
    let sign_response = sign_tx.await?;

    dbg!(&sign_response);

    let balance = user_account
        .call(contract.id(), "available_deposit")
        .args_json(serde_json::json!({"account_id": user_account.id()}))
        .view()
        .await?
        .json::<NearToken>()?;

    assert_eq!(
        balance.as_yoctonear(),
        NearToken::from_near(1).as_yoctonear()
    );

    Ok(())
}
