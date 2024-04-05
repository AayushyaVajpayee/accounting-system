use std::fs::File;
use std::io::Write;

fn main() {
    generate_rust_code_for_gst_codes("hsn.csv", "HSN_SET", "hsn_code_generated");
    generate_rust_code_for_gst_codes("sac.csv", "SAC_SET", "sac_code_generated");
    println!("cargo:rerun-if-changed=sac.csv");
    println!("cargo:rerun-if-changed=hsn.csv");
}

fn generate_rust_code_for_gst_codes(filename: &str, variable_name: &str, rust_file_name: &str) {
    let vec: Vec<String> = read_hsn_sac_codes_from_file(filename);
    let all_tuples = generate_codes_comma_separated_list(vec);
    let code = generate_rust_code(all_tuples, variable_name);
    let path = format!("src/{}.rs", rust_file_name);
    let mut file = File::create(path).unwrap();
    let _p = file.write(code.as_bytes()).unwrap();
}

fn read_hsn_sac_codes_from_file(filename: &str) -> Vec<String> {
    let mut reader = csv::Reader::from_path(filename).unwrap();
    let mut vec: Vec<String> = Vec::with_capacity(25000);
    for re in reader.records() {
        let re = re.unwrap();
        let code = re.get(0).unwrap();
        let s = code.replace(' ', "").to_string();
        vec.push(s);
    }
    vec
}

fn generate_codes_comma_separated_list(hsn_list: Vec<String>) -> String {
    hsn_list
        .chunks(25)
        .map(|a| a.join(","))
        .collect::<Vec<String>>()
        .join(",\n")
}
fn generate_rust_code(comma_separated_str: String, variable_name: &str) -> String {
    format!(
        r#"
    use lazy_static::lazy_static;
    use std::collections::HashSet;
    lazy_static!{{
       pub static ref {variable_name}:HashSet<u32> = [{}]
            .into_iter().collect();
    }}
    "#,
        comma_separated_str
    )
}
