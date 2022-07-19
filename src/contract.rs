use std::fmt;

use cosmwasm_std::{
    attr, coins, entry_point, to_binary, BankMsg, Binary, Deps, DepsMut, Env, MessageInfo,
    Response, StdResult, Uint128,
};
use provwasm_std::{ProvenanceMsg, ProvenanceQuery};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, QueryMsg, Validate};
use crate::state::{config_read, get_invoice_storage, get_invoice_storage_read, Invoice};

pub const CRATE_NAME: &str = env!("CARGO_CRATE_NAME");
pub const PACKAGE_VERSION: &str = env!("CARGO_PKG_VERSION");

// smart contract execute entrypoint
#[entry_point]
pub fn execute(
    deps: DepsMut<ProvenanceQuery>,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response<ProvenanceMsg>, ContractError> {
    msg.validate()?;

    match msg {
        ExecuteMsg::AddInvoice {
            id,
            amount,
            description,
        } => add_invoice(deps, info, id, amount, description),
        ExecuteMsg::CancelInvoice { id } => cancel_invoice(deps, info, id),
        ExecuteMsg::PayInvoice { id } => pay_invoice(deps, info, id),
    }
}

fn add_invoice(
    deps: DepsMut<ProvenanceQuery>,
    info: MessageInfo,
    id: String,
    amount: Uint128,
    description: Option<String>,
) -> Result<Response<ProvenanceMsg>, ContractError> {
    // get state for auth and attrs
    let state = &config_read(deps.storage).load()?;

    // ensure message sender is admin
    if info.sender != state.admin {
        return Err(ContractError::Unauthorized {
            error: String::from("Only admin can add invoice"),
        });
    }

    // funds should not be sent
    if !info.funds.is_empty() {
        return Err(ContractError::SentFundsUnsupported);
    }

    // invoice model
    let invoice = Invoice {
        id,
        amount,
        description,
    };

    // ensure id is unique
    let mut invoice_storage = get_invoice_storage(deps.storage);
    if invoice_storage.may_load(invoice.id.as_bytes())?.is_some() {
        return Err(ContractError::InvalidFields {
            fields: vec![String::from("id")],
        });
    }

    let response = Response::new().add_attributes(vec![
        attr("action", Action::Add.to_string()),
        attr("id", &invoice.id),
        attr("denom", &state.denom),
        attr("amount", &invoice.amount.to_string()),
        attr("recipient", &state.recipient),
    ]);

    // save invoice
    invoice_storage.save(invoice.id.as_bytes(), &invoice)?;

    Ok(response)
}

fn cancel_invoice(
    deps: DepsMut<ProvenanceQuery>,
    info: MessageInfo,
    id: String,
) -> Result<Response<ProvenanceMsg>, ContractError> {
    // get state for auth and attrs
    let state = &config_read(deps.storage).load()?;

    // ensure message sender is admin
    if info.sender != state.admin {
        return Err(ContractError::Unauthorized {
            error: String::from("Only admin can cancel invoice"),
        });
    }

    // funds should not be sent
    if !info.funds.is_empty() {
        return Err(ContractError::SentFundsUnsupported);
    }

    // ensure invoice exists
    let mut invoice_storage = get_invoice_storage(deps.storage);
    let invoice = invoice_storage
        .load(id.as_bytes())
        .map_err(|error| ContractError::LoadInvoiceFailed { error })?;

    let response = Response::new().add_attributes(vec![
        attr("action", Action::Cancel.to_string()),
        attr("id", &invoice.id),
        attr("denom", &state.denom),
        attr("amount", &invoice.amount.to_string()),
        attr("recipient", &state.recipient),
    ]);

    // remove invoice
    invoice_storage.remove(invoice.id.as_bytes());

    Ok(response)
}

fn pay_invoice(
    deps: DepsMut<ProvenanceQuery>,
    info: MessageInfo,
    id: String,
) -> Result<Response<ProvenanceMsg>, ContractError> {
    // get state for attrs
    let state = &config_read(deps.storage).load()?;

    // ensure invoice exists
    let mut invoice_storage = get_invoice_storage(deps.storage);
    let invoice = invoice_storage
        .load(id.as_bytes())
        .map_err(|error| ContractError::LoadInvoiceFailed { error })?;

    // ensure funds match invoice
    let amount = coins(invoice.amount.into(), state.denom.to_owned());
    if info.funds.ne(&amount) {
        return Err(ContractError::SentFundsInvoiceMismatch);
    }

    let mut response = Response::new().add_attributes(vec![
        attr("action", Action::Pay.to_string()),
        attr("id", &invoice.id),
        attr("denom", &state.denom),
        attr("amount", &invoice.amount.to_string()),
        attr("sender", &info.sender.to_owned()),
        attr("recipient", &state.recipient),
    ]);

    // transfer coins to recipient
    response = response.add_message(BankMsg::Send {
        to_address: state.recipient.to_string(),
        amount,
    });

    // remove invoice
    invoice_storage.remove(invoice.id.as_bytes());

    Ok(response)
}

#[entry_point]
pub fn query(deps: Deps<ProvenanceQuery>, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    msg.validate()?;

    match msg {
        QueryMsg::GetContractInfo {} => to_binary(&config_read(deps.storage).load()?),
        QueryMsg::GetVersionInfo {} => to_binary(&cw2::get_contract_version(deps.storage)?),
        QueryMsg::GetInvoice { id } => {
            to_binary(&get_invoice_storage_read(deps.storage).load(id.as_bytes())?)
        }
    }
}

enum Action {
    Add,
    Cancel,
    Pay,
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Action::Add => write!(f, "add_invoice"),
            Action::Cancel => write!(f, "cancel_invoice"),
            Action::Pay => write!(f, "pay_invoice"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::state::{config, State};
    use cosmwasm_std::testing::{mock_env, mock_info};
    use cosmwasm_std::{coin, Addr, CosmosMsg, StdError, Storage};
    use provwasm_mocks::mock_dependencies;

    use crate::state::get_invoice_storage_read;

    use super::*;

    const TEST_DENOM: &str = "testdenom";
    const INVOICE_ID: &str = "63069195-bc51-41bd-80d7-0ab84b98e283";
    const BUSINESS_NAME: &str = "company";
    const ADMIN: &str = "admin";
    const RECIPIENT: &str = "recipient";
    const DESCRIPTION: &str = "description";

    #[test]
    fn create_invoice_success() {
        let mut deps = mock_dependencies(&[]);

        setup_test_base(
            &mut deps.storage,
            &State {
                admin: Addr::unchecked(ADMIN),
                recipient: Addr::unchecked(RECIPIENT),
                denom: TEST_DENOM.into(),
                business_name: BUSINESS_NAME.into(),
            },
        );

        let amount = Uint128::new(100);
        let add_msg = ExecuteMsg::AddInvoice {
            id: INVOICE_ID.into(),
            amount: amount.into(),
            description: Option::Some(DESCRIPTION.into()),
        };

        let sender_info = mock_info(ADMIN, &[]);

        // execute add invoice
        let add_response = execute(
            deps.as_mut(),
            mock_env(),
            sender_info.clone(),
            add_msg.clone(),
        );

        // verify invoice response
        match add_response {
            Ok(response) => {
                assert_eq!(response.attributes.len(), 5);
                assert_eq!(
                    response.attributes[0],
                    attr("action", Action::Add.to_string())
                );
                assert_eq!(response.attributes[1], attr("id", INVOICE_ID));
                assert_eq!(response.attributes[2], attr("denom", TEST_DENOM));
                assert_eq!(response.attributes[3], attr("amount", amount.to_string()));
                assert_eq!(response.attributes[4], attr("recipient", RECIPIENT));
            }
            Err(error) => {
                panic!("failed to create add invoice: {:?}", error)
            }
        }

        // verify invoice stored
        let invoice_storage = get_invoice_storage_read(&deps.storage);

        match invoice_storage.load(INVOICE_ID.as_bytes()) {
            Ok(stored_invoice) => {
                assert_eq!(
                    stored_invoice,
                    Invoice {
                        id: INVOICE_ID.into(),
                        amount,
                        description: Option::Some(DESCRIPTION.into())
                    }
                )
            }
            _ => {
                panic!("invoice was not found in storage")
            }
        }
    }

    #[test]
    fn create_invoice_with_funds_throws_error() {
        let mut deps = mock_dependencies(&[]);

        setup_test_base(
            &mut deps.storage,
            &State {
                admin: Addr::unchecked(ADMIN),
                recipient: Addr::unchecked(RECIPIENT),
                denom: TEST_DENOM.into(),
                business_name: BUSINESS_NAME.into(),
            },
        );

        let amount = Uint128::new(100);
        let add_msg = ExecuteMsg::AddInvoice {
            id: INVOICE_ID.into(),
            amount: amount.into(),
            description: Option::Some(DESCRIPTION.into()),
        };

        let sender_info = mock_info(ADMIN, &[coin(amount.u128(), TEST_DENOM)]);

        // execute add invoice
        let add_response = execute(
            deps.as_mut(),
            mock_env(),
            sender_info.clone(),
            add_msg.clone(),
        );

        assert_sent_funds_unsupported_error(add_response);
    }

    #[test]
    fn create_invoice_invalid_data_error() {
        let mut deps = mock_dependencies(&[]);

        setup_test_base(
            &mut deps.storage,
            &State {
                admin: Addr::unchecked(ADMIN),
                recipient: Addr::unchecked(RECIPIENT),
                denom: TEST_DENOM.into(),
                business_name: BUSINESS_NAME.into(),
            },
        );

        let amount = Uint128::new(100);
        let add_msg = ExecuteMsg::AddInvoice {
            id: "".into(),
            amount: amount.into(),
            description: Option::Some(DESCRIPTION.into()),
        };

        let sender_info = mock_info(ADMIN, &[]);

        // execute add invoice
        let add_response = execute(
            deps.as_mut(),
            mock_env(),
            sender_info.clone(),
            add_msg.clone(),
        );

        // verify invoice response
        match add_response {
            Ok(..) => panic!("expected error, but ok"),
            Err(error) => match error {
                ContractError::InvalidFields { fields } => {
                    assert!(fields.contains(&"id".into()));
                }
                error => panic!("unexpected error: {:?}", error),
            },
        }
    }

    #[test]
    fn create_invoice_existing_id_error() {
        let mut deps = mock_dependencies(&[]);

        setup_test_base(
            &mut deps.storage,
            &State {
                admin: Addr::unchecked(ADMIN),
                recipient: Addr::unchecked(RECIPIENT),
                denom: TEST_DENOM.into(),
                business_name: BUSINESS_NAME.into(),
            },
        );

        store_test_invoice(
            &mut deps.storage,
            &Invoice {
                id: INVOICE_ID.into(),
                amount: Uint128::new(1),
                description: Option::None,
            },
        );

        let amount = Uint128::new(100);
        let add_msg = ExecuteMsg::AddInvoice {
            id: INVOICE_ID.into(),
            amount: amount.into(),
            description: Option::Some(DESCRIPTION.into()),
        };

        let sender_info = mock_info(ADMIN, &[]);

        // execute add invoice
        let add_response = execute(
            deps.as_mut(),
            mock_env(),
            sender_info.clone(),
            add_msg.clone(),
        );

        // verify invoice response
        match add_response {
            Ok(..) => panic!("expected error, but ok"),
            Err(error) => match error {
                ContractError::InvalidFields { fields } => {
                    assert!(fields.contains(&"id".into()));
                }
                error => panic!("unexpected error: {:?}", error),
            },
        }
    }

    #[test]
    fn create_invoice_unauthorized_error() {
        let mut deps = mock_dependencies(&[]);

        setup_test_base(
            &mut deps.storage,
            &State {
                admin: Addr::unchecked(ADMIN),
                recipient: Addr::unchecked(RECIPIENT),
                denom: TEST_DENOM.into(),
                business_name: BUSINESS_NAME.into(),
            },
        );

        let amount = Uint128::new(100);
        let add_msg = ExecuteMsg::AddInvoice {
            id: INVOICE_ID.into(),
            amount: amount.into(),
            description: Option::Some(DESCRIPTION.into()),
        };

        let sender_info = mock_info("invalid_sender", &[]);

        // execute add invoice
        let add_response = execute(
            deps.as_mut(),
            mock_env(),
            sender_info.clone(),
            add_msg.clone(),
        );

        assert_not_authorized_error(add_response);
    }

    #[test]
    fn cancel_invoice_success() {
        let mut deps = mock_dependencies(&[]);

        setup_test_base(
            &mut deps.storage,
            &State {
                admin: Addr::unchecked(ADMIN),
                recipient: Addr::unchecked(RECIPIENT),
                denom: TEST_DENOM.into(),
                business_name: BUSINESS_NAME.into(),
            },
        );

        let amount = Uint128::new(5);
        store_test_invoice(
            &mut deps.storage,
            &Invoice {
                id: INVOICE_ID.into(),
                amount: amount.into(),
                description: Option::None,
            },
        );

        let cancel_msg = ExecuteMsg::CancelInvoice {
            id: INVOICE_ID.into(),
        };

        let sender_info = mock_info(ADMIN, &[]);

        // execute cancel invoice
        let cancel_response = execute(
            deps.as_mut(),
            mock_env(),
            sender_info.clone(),
            cancel_msg.clone(),
        );

        // verify invoice response
        match cancel_response {
            Ok(response) => {
                assert_eq!(response.attributes.len(), 5);
                assert_eq!(
                    response.attributes[0],
                    attr("action", Action::Cancel.to_string())
                );
                assert_eq!(response.attributes[1], attr("id", INVOICE_ID));
                assert_eq!(response.attributes[2], attr("denom", TEST_DENOM));
                assert_eq!(response.attributes[3], attr("amount", amount.to_string()));
                assert_eq!(response.attributes[4], attr("recipient", RECIPIENT));
            }
            Err(error) => {
                panic!("failed to create add invoice: {:?}", error)
            }
        }

        // verify invoice stored
        let invoice_storage = get_invoice_storage_read(&deps.storage);

        match invoice_storage.load(INVOICE_ID.as_bytes()) {
            Ok(..) => panic!("expected error, but found"),
            Err(error) => match error {
                StdError::NotFound { .. } => {}
                error => panic!("unexpected error: {:?}", error),
            },
        }
    }

    #[test]
    fn cancel_invoice_not_found_error() {
        let mut deps = mock_dependencies(&[]);

        setup_test_base(
            &mut deps.storage,
            &State {
                admin: Addr::unchecked(ADMIN),
                recipient: Addr::unchecked(RECIPIENT),
                denom: TEST_DENOM.into(),
                business_name: BUSINESS_NAME.into(),
            },
        );

        let cancel_msg = ExecuteMsg::CancelInvoice {
            id: INVOICE_ID.into(),
        };

        let sender_info = mock_info(ADMIN, &[]);

        // execute pay invoice
        let cancel_response = execute(
            deps.as_mut(),
            mock_env(),
            sender_info.clone(),
            cancel_msg.clone(),
        );

        // verify invoice response
        match cancel_response {
            Ok(..) => panic!("expected error, but ok"),
            Err(error) => match error {
                ContractError::LoadInvoiceFailed { .. } => {}
                error => panic!("unexpected error: {:?}", error),
            },
        }
    }

    #[test]
    fn cancel_invoice_with_funds_throws_error() {
        let mut deps = mock_dependencies(&[]);

        setup_test_base(
            &mut deps.storage,
            &State {
                admin: Addr::unchecked(ADMIN),
                recipient: Addr::unchecked(RECIPIENT),
                denom: TEST_DENOM.into(),
                business_name: BUSINESS_NAME.into(),
            },
        );

        let amount = Uint128::new(5);
        store_test_invoice(
            &mut deps.storage,
            &Invoice {
                id: INVOICE_ID.into(),
                amount: amount.into(),
                description: Option::None,
            },
        );

        let cancel_msg = ExecuteMsg::CancelInvoice {
            id: INVOICE_ID.into(),
        };

        let sender_info = mock_info(ADMIN, &[coin(amount.u128(), TEST_DENOM)]);

        // execute cancel invoice
        let cancel_response = execute(
            deps.as_mut(),
            mock_env(),
            sender_info.clone(),
            cancel_msg.clone(),
        );

        assert_sent_funds_unsupported_error(cancel_response);
    }

    #[test]
    fn cancel_invoice_unauthorized_error() {
        let mut deps = mock_dependencies(&[]);

        setup_test_base(
            &mut deps.storage,
            &State {
                admin: Addr::unchecked(ADMIN),
                recipient: Addr::unchecked(RECIPIENT),
                denom: TEST_DENOM.into(),
                business_name: BUSINESS_NAME.into(),
            },
        );

        let amount = Uint128::new(5);
        store_test_invoice(
            &mut deps.storage,
            &Invoice {
                id: INVOICE_ID.into(),
                amount: amount.into(),
                description: Option::None,
            },
        );

        let cancel_msg = ExecuteMsg::CancelInvoice {
            id: INVOICE_ID.into(),
        };

        let sender_info = mock_info("invalid_sender", &[]);

        // execute cancel invoice
        let cancel_response = execute(
            deps.as_mut(),
            mock_env(),
            sender_info.clone(),
            cancel_msg.clone(),
        );

        assert_not_authorized_error(cancel_response);
    }

    #[test]
    fn pay_invoice_success() {
        let mut deps = mock_dependencies(&[]);

        setup_test_base(
            &mut deps.storage,
            &State {
                admin: Addr::unchecked(ADMIN),
                recipient: Addr::unchecked(RECIPIENT),
                denom: TEST_DENOM.into(),
                business_name: BUSINESS_NAME.into(),
            },
        );

        let amount = Uint128::new(5);
        store_test_invoice(
            &mut deps.storage,
            &Invoice {
                id: INVOICE_ID.into(),
                amount: amount.into(),
                description: Option::None,
            },
        );

        let pay_invoice = ExecuteMsg::PayInvoice {
            id: INVOICE_ID.into(),
        };

        let sender_info = mock_info("payer", &[coin(amount.u128(), TEST_DENOM)]);

        // execute pay invoice
        let pay_response = execute(
            deps.as_mut(),
            mock_env(),
            sender_info.clone(),
            pay_invoice.clone(),
        );

        // verify invoice response
        match pay_response {
            Ok(response) => {
                assert_eq!(response.attributes.len(), 6);
                assert_eq!(
                    response.attributes[0],
                    attr("action", Action::Pay.to_string())
                );
                assert_eq!(response.attributes[1], attr("id", INVOICE_ID));
                assert_eq!(response.attributes[2], attr("denom", TEST_DENOM));
                assert_eq!(response.attributes[3], attr("amount", amount.to_string()));
                assert_eq!(response.attributes[4], attr("sender", "payer"));
                assert_eq!(response.attributes[5], attr("recipient", RECIPIENT));

                assert_eq!(response.messages.len(), 1);
                assert_eq!(
                    response.messages[0].msg,
                    CosmosMsg::Bank(BankMsg::Send {
                        to_address: RECIPIENT.to_string(),
                        amount: coins(amount.u128(), TEST_DENOM),
                    })
                );
            }
            Err(error) => {
                panic!("failed to create add invoice: {:?}", error)
            }
        }

        // verify invoice stored
        let invoice_storage = get_invoice_storage_read(&deps.storage);

        match invoice_storage.load(INVOICE_ID.as_bytes()) {
            Ok(..) => panic!("expected error, but found"),
            Err(error) => match error {
                StdError::NotFound { .. } => {}
                error => panic!("unexpected error: {:?}", error),
            },
        }
    }

    #[test]
    fn pay_invoice_not_found_error() {
        let mut deps = mock_dependencies(&[]);

        setup_test_base(
            &mut deps.storage,
            &State {
                admin: Addr::unchecked(ADMIN),
                recipient: Addr::unchecked(RECIPIENT),
                denom: TEST_DENOM.into(),
                business_name: BUSINESS_NAME.into(),
            },
        );

        let pay_msg = ExecuteMsg::PayInvoice {
            id: INVOICE_ID.into(),
        };

        let amount = Uint128::new(5);
        let sender_info = mock_info("payer", &[coin(amount.u128(), TEST_DENOM)]);

        // execute pay invoice
        let pay_response = execute(
            deps.as_mut(),
            mock_env(),
            sender_info.clone(),
            pay_msg.clone(),
        );

        // verify invoice response
        match pay_response {
            Ok(..) => panic!("expected error, but ok"),
            Err(error) => match error {
                ContractError::LoadInvoiceFailed { .. } => {}
                error => panic!("unexpected error: {:?}", error),
            },
        }
    }

    #[test]
    fn pay_invoice_mismatch_funds_error() {
        let mut deps = mock_dependencies(&[]);

        setup_test_base(
            &mut deps.storage,
            &State {
                admin: Addr::unchecked(ADMIN),
                recipient: Addr::unchecked(RECIPIENT),
                denom: TEST_DENOM.into(),
                business_name: BUSINESS_NAME.into(),
            },
        );

        let amount = Uint128::new(5);
        store_test_invoice(
            &mut deps.storage,
            &Invoice {
                id: INVOICE_ID.into(),
                amount: amount.into(),
                description: Option::None,
            },
        );

        let pay_msg = ExecuteMsg::PayInvoice {
            id: INVOICE_ID.into(),
        };

        // mismatch sender on coin amount
        let mut sender_info = mock_info("payer", &[coin(10, TEST_DENOM)]);

        // execute pay invoice
        let mut pay_response = execute(
            deps.as_mut(),
            mock_env(),
            sender_info.clone(),
            pay_msg.clone(),
        );

        // verify invoice response
        match pay_response {
            Ok(..) => panic!("expected error, but ok"),
            Err(error) => match error {
                ContractError::SentFundsInvoiceMismatch => {}
                error => panic!("unexpected error: {:?}", error),
            },
        }

        // mismatch sender on coin denom
        sender_info = mock_info("payer", &[coin(5, "wrongdenom")]);

        pay_response = execute(
            deps.as_mut(),
            mock_env(),
            sender_info.clone(),
            pay_msg.clone(),
        );

        // verify invoice response
        match pay_response {
            Ok(..) => panic!("expected error, but ok"),
            Err(error) => match error {
                ContractError::SentFundsInvoiceMismatch => {}
                error => panic!("unexpected error: {:?}", error),
            },
        }

        // verify invoice stored
        let invoice_storage = get_invoice_storage_read(&deps.storage);

        match invoice_storage.load(INVOICE_ID.as_bytes()) {
            Ok(stored_invoice) => {
                assert_eq!(
                    stored_invoice,
                    Invoice {
                        id: INVOICE_ID.into(),
                        amount,
                        description: Option::None
                    }
                )
            }
            _ => {
                panic!("invoice was not found in storage")
            }
        }
    }

    fn assert_sent_funds_unsupported_error(
        response: Result<Response<ProvenanceMsg>, ContractError>,
    ) {
        match response {
            Ok(..) => panic!("expected error, but ok"),
            Err(error) => match error {
                ContractError::SentFundsUnsupported => {}
                error => panic!("unexpected error: {:?}", error),
            },
        }
    }

    fn assert_not_authorized_error(response: Result<Response<ProvenanceMsg>, ContractError>) {
        // verify invoice response
        match response {
            Ok(..) => panic!("expected error, but ok"),
            Err(error) => match error {
                ContractError::Unauthorized { error } => {
                    assert!(error.contains("admin"));
                }
                error => panic!("unexpected error: {:?}", error),
            },
        }
    }

    fn setup_test_base(storage: &mut dyn Storage, contract_info: &State) {
        if let Err(error) = config(storage).save(&contract_info) {
            panic!("unexpected error: {:?}", error)
        }
    }

    fn store_test_invoice(storage: &mut dyn Storage, invoice: &Invoice) {
        let mut invoice_storage = get_invoice_storage(storage);
        if let Err(error) = invoice_storage.save(invoice.id.as_bytes(), invoice) {
            panic!("unexpected error: {:?}", error)
        };
    }
}
