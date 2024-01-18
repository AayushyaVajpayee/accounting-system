use crate::masters::address_master::address_model::CreateAddressRequest;

pub fn create_address_input_for_db_function(request: &CreateAddressRequest) -> String {
    let simple_query = format!(
        r#"Row('{}','{}','{}',{},{},'{}','{}','{}','{}','{}',{}::smallint)"#,
        request.idempotence_key,
        request.tenant_id,
        request.line_1,
        request.line_2.as_ref().map(|a| format!("'{}'", a))
            .unwrap_or_else(|| "null".to_string()),
        request.landmark.as_ref().map(|a| format!("'{}'", a))
            .unwrap_or_else(|| "null".to_string()),
        request.city_id,
        request.state_id,
        request.country_id,
        request.pincode_id,
        request.created_by,
        1
    );
    simple_query
}