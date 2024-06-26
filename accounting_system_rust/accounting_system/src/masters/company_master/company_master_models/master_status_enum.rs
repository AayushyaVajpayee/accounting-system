use anyhow::bail;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Copy, Clone, Default)]
#[serde(try_from = "i32")]
#[serde(into = "i32")]
pub enum MasterStatusEnum {
    PendingApproval = 0,
    #[default]
    Approved = 1,
    ChangesRequested = 2,
    Deleted = 3,
}

impl MasterStatusEnum {
    pub fn get_enum_for_value(value: usize) -> anyhow::Result<Self> {
        match value {
            0 => Ok(MasterStatusEnum::PendingApproval),
            1 => Ok(MasterStatusEnum::Approved),
            2 => Ok(MasterStatusEnum::ChangesRequested),
            3 => Ok(MasterStatusEnum::Deleted),
            _ => bail!("no master status for this value"),
        }
    }
}

impl From<MasterStatusEnum> for i32 {
    fn from(value: MasterStatusEnum) -> Self {
        value as i32
    }
}

impl TryFrom<i32> for MasterStatusEnum {
    type Error = anyhow::Error;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        let value: usize = value.try_into()?;
        MasterStatusEnum::get_enum_for_value(value)
    }
}

#[cfg(test)]
mod master_status_enum_tests {
    use anyhow::{anyhow, Error};
    use rstest::rstest;
    use speculoos::assert_that;
    use speculoos::prelude::ResultAssertions;

    use crate::masters::company_master::company_master_models::master_status_enum::MasterStatusEnum;
    use crate::masters::company_master::company_master_models::master_status_enum::MasterStatusEnum::Approved;
    use crate::masters::company_master::company_master_models::master_status_enum::MasterStatusEnum::ChangesRequested;
    use crate::masters::company_master::company_master_models::master_status_enum::MasterStatusEnum::Deleted;
    use crate::masters::company_master::company_master_models::master_status_enum::MasterStatusEnum::PendingApproval;

    #[rstest]
    #[case(0, Ok(PendingApproval))]
    #[case(1, Ok(Approved))]
    #[case(2, Ok(ChangesRequested))]
    #[case(3, Ok(Deleted))]
    #[case(4, Err(anyhow ! ("no master status for this value")))]
    fn test_master_status_enum(
        #[case] input: usize,
        #[case] output: Result<MasterStatusEnum, Error>,
    ) {
        let k = MasterStatusEnum::get_enum_for_value(input);
        if output.is_err() {
            assert_that!(k)
                .is_err()
                .matches(|a| a.to_string() == output.as_ref().unwrap_err().to_string());
        } else {
            assert_that!(k.unwrap()).is_equal_to(output.unwrap())
        }
    }
}
