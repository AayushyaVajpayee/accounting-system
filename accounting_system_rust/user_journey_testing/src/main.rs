use std::str::FromStr;

use lazy_static::lazy_static;
use rand::Rng;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::business_entity::create_random_business_entity;
use crate::currency::create_random_currency;
use crate::invoice::{BillShipDetail, create_random_invoice};
use crate::invoice_series::create_random_invoice_series_mst;
use crate::product_item::create_random_product;
use crate::tenant::create_random_tenant;
use crate::user::{create_random_user_with_other_user_of_same_tenant, create_random_user_with_super_tenant};
use crate::util::{generate_random_gstin_no, send_request};

mod business_entity;
mod util;
mod product_item;
mod tenant;
mod user;
mod currency;
mod invoice_series;
mod invoice;

const LOCAL_HOST: &str = "http://localhost:8090/";

lazy_static! {
    pub static ref SUPER_TENANT_ID: Uuid =
        Uuid::from_str("018b33d9-c862-7fde-a0cd-55504d75e5e9").unwrap();
}

lazy_static! {
    pub static ref SUPER_USER_ID: Uuid =
        Uuid::from_str("018b3444-dc75-7a3f-a4d9-02c41071d3bd").unwrap();
}




#[tokio::main]
async fn main() {
    let tenant_id = create_random_tenant().await;
    println!("{}", tenant_id);
    let user_id = create_random_user_with_super_tenant(tenant_id).await;
    println!("user_id {}", user_id);
    let user_id_1 = create_random_user_with_other_user_of_same_tenant(tenant_id, user_id).await;
    println!("user_id_1 {}", user_id_1);
    let currency_id = create_random_currency(tenant_id, user_id_1).await;
    println!("currency id {}", currency_id);
    let product_id = create_random_product(tenant_id, user_id).await;
    println!("product_id {}", product_id);
    let invoicing_series_mst_id = create_random_invoice_series_mst(tenant_id, user_id_1).await;
    println!("invoicing series mst {}", invoicing_series_mst_id);
    let supplier_id = create_random_business_entity(tenant_id, user_id_1).await;
    println!("business entity id {}", supplier_id);
    let bill_to_id = create_random_business_entity(tenant_id, user_id_1).await;
    let bill_ship_dtl = BillShipDetail {
        billed_to_customer_id: bill_to_id,
        shipped_to_customer_id: bill_to_id,
    };
    let invoice = create_random_invoice(product_id, supplier_id,bill_ship_dtl,
                                        currency_id,invoicing_series_mst_id,
                                        tenant_id,user_id_1).await;
    println!("generated invoice {}",invoice);
    // let product_item_id=create_product();
}
