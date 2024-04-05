use serde_json::Value;

#[tokio::main]
async fn main() {
    let p = reqwest::get("http://localhost:8080/tenant/id/018b33d9-c862-7fde-a0cd-55504d75e5e9")
        .await
        .unwrap();
    println!("{}", p.status().as_str());

    let pp: Value = p.json().await.unwrap();
    println!("Hello, world! {:?}", pp);
}

//create testing journey for invoice creation
//1. user logs in
//2. user sees the company master and asks if its available
//3. if not, user creates the company master
//4. user needs to select company unit,if not available then create it
//5. user should select a template, if none then register a new one may be.
//6. user should select the invoicing series and increment it. if the desired series is not there, then user should create it

//7. once selected, create an invoice by sending the line item details,
// to create the invoice,
// 1. create line items
//8. should be able to get s3 storage link for pdf as well as raw data.
