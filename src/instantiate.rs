use crate::contract::{CRATE_NAME, PACKAGE_VERSION};
use crate::error::contract_err;
use crate::msg::{InstantiateMsg, Validate};
use crate::state::{config, config_read, State};
use crate::ContractError;
use cosmwasm_std::{attr, entry_point, DepsMut, Env, MessageInfo, Response};
use cw2::set_contract_version;
use provwasm_std::{Marker, MarkerType, ProvenanceMsg, ProvenanceQuerier, ProvenanceQuery};

/// Create the initial configuration state
#[entry_point]
pub fn instantiate(
    deps: DepsMut<ProvenanceQuery>,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response<ProvenanceMsg>, ContractError> {
    msg.validate()?;
    // validate params
    if !info.funds.is_empty() {
        return Err(contract_err("no funds should be sent during instantiate"));
    }

    let is_unrestricted_marker = matches!(
        ProvenanceQuerier::new(&deps.querier).get_marker_by_denom(msg.denom.clone()),
        Ok(Marker {
            marker_type: MarkerType::Coin,
            ..
        })
    );

    // only unrestricted markers are supported
    if !is_unrestricted_marker {
        return Err(ContractError::UnsupportedMarkerType);
    }

    // create and store config state
    let contract_info = State {
        admin: info.sender.clone(),
        recipient: deps.api.addr_validate(&msg.recipient)?,
        denom: msg.denom.clone(),
        business_name: msg.business_name.clone(),
    };
    config(deps.storage).save(&contract_info)?;

    set_contract_version(deps.storage, CRATE_NAME, PACKAGE_VERSION)?;

    // build response
    Ok(Response::new().add_attributes(vec![
        attr(
            "contract_info",
            format!("{:?}", config_read(deps.storage).load()?),
        ),
        attr("action", "init"),
    ]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_env, mock_info};
    use cosmwasm_std::{from_binary, Addr, Binary};
    use provwasm_mocks::mock_dependencies;

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies(&[]);
        let info = mock_info("contract_admin", &[]);

        let denom = "unrestricted";
        let recipient_address = Addr::unchecked("recipient");
        let business_name = "please transfer me";

        let init_msg = InstantiateMsg {
            denom: denom.into(),
            recipient: recipient_address.to_string(),
            business_name: business_name.into(),
        };

        let test_marker: Marker = setup_unrestricted_marker();
        deps.querier.with_markers(vec![test_marker]);

        let init_response = instantiate(deps.as_mut(), mock_env(), info.clone(), init_msg.clone());

        // verify initialize response
        match init_response {
            Ok(init_response) => {
                assert_eq!(init_response.messages.len(), 0);

                assert_eq!(init_response.attributes.len(), 2);

                let expected_state = State {
                    admin: info.sender.into(),
                    denom: denom.into(),
                    recipient: recipient_address.to_owned(),
                    business_name: business_name.into(),
                };

                assert_eq!(
                    init_response.attributes[0],
                    attr("contract_info", format!("{:?}", expected_state))
                );
                assert_eq!(init_response.attributes[1], attr("action", "init"));

                let version_info = cw2::get_contract_version(&deps.storage).unwrap();

                assert_eq!(PACKAGE_VERSION, version_info.version);
                assert_eq!(CRATE_NAME, version_info.contract);
            }
            error => panic!("failed to initialize: {:?}", error),
        }
    }

    fn setup_unrestricted_marker() -> Marker {
        let marker_json = b"{
              \"address\": \"tp1l330sxue4suxz9dhc40e2pns0ymrytf8uz4squ\",
              \"coins\": [
                {
                  \"denom\": \"unrestricted\",
                  \"amount\": \"1000\"
                }
              ],
              \"account_number\": 10,
              \"sequence\": 0,
              \"permissions\": [
                {
                  \"permissions\": [
                    \"burn\",
                    \"delete\",
                    \"deposit\",
                    \"admin\",
                    \"mint\",
                    \"withdraw\"
                  ],
                  \"address\": \"tp13pnzut8zdjaqht7aqe7kk4ww5zfq04jzlytnmu\"
                }
              ],
              \"status\": \"active\",
              \"denom\": \"unrestricted\",
              \"total_supply\": \"1000\",
              \"marker_type\": \"coin\",
              \"supply_fixed\": false
            }";

        return from_binary(&Binary::from(marker_json)).unwrap();
    }
}
