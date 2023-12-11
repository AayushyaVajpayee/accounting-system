use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::masters::company_master::company_master_models::master_status_enum::MasterStatusEnum;
use crate::masters::company_master::company_master_models::master_updation_remarks::MasterUpdationRemarks;

#[derive(Debug, Serialize, Deserialize)]
pub struct BaseMasterFields {
    pub id: Uuid,
    pub entity_version_id: i32,
    pub tenant_id: Uuid,
    pub active: bool,
    pub approval_status: MasterStatusEnum,
    pub remarks: Option<MasterUpdationRemarks>,
}


#[cfg(test)]
pub mod tests {
    use uuid::Uuid;
    use crate::masters::company_master::company_master_models::base_master_fields::BaseMasterFields;
    use crate::masters::company_master::company_master_models::master_status_enum::MasterStatusEnum;
    use crate::masters::company_master::company_master_models::master_status_enum::MasterStatusEnum::Approved;
    use crate::masters::company_master::company_master_models::master_updation_remarks::MasterUpdationRemarks;
    use crate::tenant::tenant_models::SEED_TENANT_ID;

    #[derive(Debug, Default)]
    pub struct BaseMasterFieldsTestDataBuilder {
        pub id: Option<Uuid>,
        pub entity_version_id: Option<i32>,
        pub tenant_id: Option<Uuid>,
        pub active: Option<bool>,
        pub approval_status: Option<MasterStatusEnum>,
        pub remarks: Option<MasterUpdationRemarks>,
    }

    pub fn a_base_master_field(builder: BaseMasterFieldsTestDataBuilder) -> BaseMasterFields {
        BaseMasterFields {
            id: builder.id.unwrap_or_else(Uuid::now_v7),
            entity_version_id: builder.entity_version_id.unwrap_or(0),
            tenant_id: builder.tenant_id.unwrap_or(*SEED_TENANT_ID),
            active: builder.active.unwrap_or(true),
            approval_status: builder.approval_status.unwrap_or(Approved),
            remarks: builder.remarks,
        }
    }
}