use crate::error::ContractError;
use cosmwasm_std::Uint128;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub denom: String,
    pub recipient: String,
    pub business_name: String,
}

/// Simple validation of InstantiateMsg data
///
/// ### Example
///
/// ```rust
/// use invoice::msg::{InstantiateMsg, Validate};
/// pub fn instantiate(msg: InstantiateMsg) {
///
///     let result = msg.validate();
/// }
/// ```
impl Validate for InstantiateMsg {
    fn validate(&self) -> Result<(), ContractError> {
        let mut invalid_fields: Vec<&str> = vec![];

        if self.denom.is_empty() {
            invalid_fields.push("denom");
        }

        if self.business_name.is_empty() {
            invalid_fields.push("business_name");
        }

        match invalid_fields.len() {
            0 => Ok(()),
            _ => Err(ContractError::InvalidFields {
                fields: invalid_fields.into_iter().map(|item| item.into()).collect(),
            }),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct MigrateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    AddInvoice {
        id: String,
        amount: Uint128,
        description: Option<String>,
    },
    PayInvoice {
        id: String,
    },
    CancelInvoice {
        id: String,
    },
}

impl Validate for ExecuteMsg {
    /// Simple validation of ExecuteMsg data
    ///
    /// ### Example
    ///
    /// ```rust
    /// use invoice::msg::{ExecuteMsg, Validate};
    ///
    /// pub fn execute(msg: ExecuteMsg) {
    ///     let result = msg.validate();
    ///     todo!()
    /// }
    /// ```
    fn validate(&self) -> Result<(), ContractError> {
        let mut invalid_fields: Vec<&str> = vec![];

        match self {
            ExecuteMsg::AddInvoice {
                id,
                amount,
                description,
            } => {
                if Uuid::parse_str(id).is_err() {
                    invalid_fields.push("id");
                }

                if amount.lt(&Uint128::new(1)) {
                    invalid_fields.push("amount");
                }

                match description {
                    Some(d) => {
                        if d.is_empty() || d.len() > 64 {
                            invalid_fields.push("description");
                        }
                    }
                    None => {
                        // noop
                    }
                }
            }
            ExecuteMsg::PayInvoice { id } => {
                if Uuid::parse_str(id).is_err() {
                    invalid_fields.push("id");
                }
            }
            ExecuteMsg::CancelInvoice { id } => {
                if Uuid::parse_str(id).is_err() {
                    invalid_fields.push("id");
                }
            }
        }

        match invalid_fields.len() {
            0 => Ok(()),
            _ => Err(ContractError::InvalidFields {
                fields: invalid_fields.into_iter().map(|item| item.into()).collect(),
            }),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetInvoice { id: String },
    GetContractInfo {},
    GetVersionInfo {},
}

impl Validate for QueryMsg {
    /// Simple validation of QueryMsg data
    ///
    /// ### Example
    ///
    /// ```rust
    /// use invoice::msg::{QueryMsg, Validate};
    /// pub fn query(msg: QueryMsg) {
    ///
    ///     let result = msg.validate();
    /// }
    /// ```
    fn validate(&self) -> Result<(), ContractError> {
        let mut invalid_fields: Vec<&str> = vec![];

        match self {
            QueryMsg::GetInvoice { id } => {
                if Uuid::parse_str(id).is_err() {
                    invalid_fields.push("id");
                }
            }
            QueryMsg::GetContractInfo {} => {}
            QueryMsg::GetVersionInfo {} => {}
        }

        match invalid_fields.len() {
            0 => Ok(()),
            _ => Err(ContractError::InvalidFields {
                fields: invalid_fields.into_iter().map(|item| item.into()).collect(),
            }),
        }
    }
}

pub trait Validate {
    fn validate(&self) -> Result<(), ContractError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::msg::ExecuteMsg::{AddInvoice, CancelInvoice, PayInvoice};

    #[test]
    fn validate_add_invoice() {
        let invalid_add_msg = AddInvoice {
            id: "fake-id".to_string(),
            amount: Uint128::new(0),
            description: Option::Some("".to_string()),
        };

        let validate_response = invalid_add_msg.validate();

        match validate_response {
            Ok(..) => panic!("expected error but was ok"),
            Err(error) => match error {
                ContractError::InvalidFields { fields } => {
                    assert_eq!(3, fields.len());
                    assert!(fields.contains(&"id".into()));
                    assert!(fields.contains(&"amount".into()));
                    assert!(fields.contains(&"description".into()));
                }
                error => panic!("unexpected error: {:?}", error),
            },
        }
    }

    #[test]
    fn validate_pay_invoice() {
        let invalid_pay_msg = PayInvoice {
            id: "not-a-real-uuid".to_string(),
        };

        let validate_response = invalid_pay_msg.validate();

        match validate_response {
            Ok(..) => panic!("expected error but was ok"),
            Err(error) => match error {
                ContractError::InvalidFields { fields } => {
                    assert_eq!(1, fields.len());
                    assert!(fields.contains(&"id".into()));
                }
                error => panic!("unexpected error: {:?}", error),
            },
        }
    }

    #[test]
    fn validate_cancel_invoice() {
        let invalid_cancel_msg = CancelInvoice {
            id: "not-a-real-uuid".to_string(),
        };

        let validate_response = invalid_cancel_msg.validate();

        match validate_response {
            Ok(..) => panic!("expected error but was ok"),
            Err(error) => match error {
                ContractError::InvalidFields { fields } => {
                    assert_eq!(1, fields.len());
                    assert!(fields.contains(&"id".into()));
                }
                error => panic!("unexpected error: {:?}", error),
            },
        }
    }
}
