mod common;

use common::{create_sign_args, deploy_contract, deploy_mpc_contract};
use near_sdk::serde_json;
use near_workspaces::types::{Gas, NearToken};

#[tokio::test]
async fn test_sign_works() -> Result<(), Box<dyn std::error::Error>> {
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

    // Imitate sending a response to MPC contract
    let respond_response = mpc
        .call("respond")
        .args_json(serde_json::json!({
            "request": {
            "epsilon": {
              "scalar": "C78ED94A11AE23926598BBCE0B4352B16E67AE783A6641145F441D6BEE5CA2E8"
            },
            "payload_hash": {
              "scalar": "3A501B26BDE9794DDA0CFDEDFBC02F0C63FDCB7AAD2C0B07F7722173AD560731"
            }
          },
          // The response object is just a mock
          "response": {
            "big_r": {
              "affine_point": "03214BB5B327CEC619FB0447C84E23E5DF462FD758D46F0A21A36EF9BC083EF53B"
            },
            "recovery_id": 0,
            "s": {
              "scalar": "314BA3D6CC3B41C255C857C1216FFCC9AE71A17C0B38146613D4C6EFE5416FC7"
            }
          }
        }))
        .max_gas()
        .transact()
        .await?;

    dbg!(&respond_response);

    let sign_response = sign_tx.await?;

    dbg!(&sign_response);

    assert!(respond_response.is_success());

    let json_response = sign_response.json::<serde_json::Value>()?;

    // ensure that returned signature is the same as provided above
    assert_eq!(
        json_response["big_r"]["affine_point"],
        "03214BB5B327CEC619FB0447C84E23E5DF462FD758D46F0A21A36EF9BC083EF53B"
    );
    assert_eq!(json_response["recovery_id"], 0);
    assert_eq!(
        json_response["s"]["scalar"],
        "314BA3D6CC3B41C255C857C1216FFCC9AE71A17C0B38146613D4C6EFE5416FC7"
    );

    let balance = user_account
        .call(contract.id(), "available_deposit")
        .args_json(serde_json::json!({"account_id": user_account.id()}))
        .view()
        .await?
        .json::<NearToken>()?;

    assert_eq!(
        balance.as_yoctonear(),
        NearToken::from_near(1).as_yoctonear() - 5
    );

    Ok(())
}

#[tokio::test]
async fn test_deposit_works() -> Result<(), Box<dyn std::error::Error>> {
    let sandbox = near_workspaces::sandbox().await?;

    let mpc = deploy_mpc_contract(&sandbox).await?;
    let contract = deploy_contract(&sandbox, mpc.id()).await?;

    let user = sandbox.dev_create_account().await?;

    let balance = contract
        .call("available_deposit")
        .args_json(serde_json::json!({"account_id": user.id()}))
        .view()
        .await?
        .json::<NearToken>()?;

    assert_eq!(balance.as_yoctonear(), 0);

    let _ = user
        .call(contract.id(), "deposit")
        .deposit(NearToken::from_near(1))
        .transact()
        .await?;

    let balance = contract
        .call("available_deposit")
        .args_json(serde_json::json!({"account_id": user.id()}))
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
async fn test_withdraw_works() -> Result<(), Box<dyn std::error::Error>> {
    let sandbox = near_workspaces::sandbox().await?;

    let mpc = deploy_mpc_contract(&sandbox).await?;
    let contract = deploy_contract(&sandbox, mpc.id()).await?;

    let user = sandbox.dev_create_account().await?;

    // default balance is 100 Near
    let account_balance = user.view_account().await?.balance;
    assert!(account_balance >= NearToken::from_near(100));

    let _ = user
        .call(contract.id(), "deposit")
        .deposit(NearToken::from_near(1))
        .transact()
        .await?;

    let _ = user
        .call(contract.id(), "withdraw")
        .args_json(serde_json::json!({
            "amount": NearToken::from_millinear(500) // 0.5 Near
        }))
        .transact()
        .await?;

    // wait for a few blocks to pass by
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    let balance = contract
        .call("available_deposit")
        .args_json(serde_json::json!({"account_id": user.id()}))
        .view()
        .await?
        .json::<NearToken>()?;

    assert_eq!(
        balance.as_yoctonear(),
        NearToken::from_millinear(500).as_yoctonear()
    );

    // must be very close to 99.5 Near, because quite a few yocto have gone as fees
    let account_balance = user.view_account().await?.balance;
    assert!(account_balance >= NearToken::from_millinear(99_499)); // more than 99.499 Near

    Ok(())
}
