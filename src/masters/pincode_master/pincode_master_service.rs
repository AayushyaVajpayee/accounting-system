use async_trait::async_trait;

#[async_trait]
trait PincodeMasterService {
    async fn get_all_pincodes();
    async fn get_pincode_by_id();
}

#[cfg(test)]
mod tests {
    // #[test]
    // fn test_s() {
    //     let mut p = env::current_dir().unwrap();
    //     p.push("pincode.csv");
    //     let mut k = csv::Reader::from_path(p).unwrap();
    //     println!("{}", k.has_headers());
    //     let mut pincode_city_map: HashMap<u32, String> = HashMap::with_capacity(20000);
    //     let mut city_state_map: HashMap<String, String> = HashMap::with_capacity(800);
    //     let mut pincodes: HashSet<u32> = HashSet::with_capacity(40000);
    //     let mut cities: HashSet<String> = HashSet::with_capacity(1000);
    //     let mut states: HashSet<String> = HashSet::with_capacity(30);
    //     for rec in k.records() {
    //         let reca = rec.unwrap();
    //         let pincode_raw = reca.get(4).unwrap().trim();
    //         let city_raw = reca.get(7).unwrap().trim().to_ascii_uppercase();
    //         let state = reca.get(8).unwrap().trim().to_ascii_uppercase();
    //         let pincode = pincode_raw.parse::<u32>().unwrap();
    //         cities.insert(city_raw.clone());
    //         states.insert(state.clone());
    //         pincodes.insert(pincode);
    //         pincode_city_map.insert(pincode, city_raw.clone());
    //         city_state_map.insert(city_raw, state);
    //     }
    //     let mut cities = cities.into_iter().collect::<Vec<String>>();
    //     cities.sort_unstable();
    //     let mut states = states.into_iter().collect::<Vec<String>>();
    //     states.sort_unstable();
    //     let mut pincodes = pincodes.into_iter().collect::<Vec<u32>>();
    //     pincodes.sort_unstable();
    //     let mut state_rows: Vec<String> = Vec::with_capacity(34);
    //     let state_row_header =
    //         "id,state_code,state_name,created_by,updated_by,created_at,updated_at".to_string();
    //     state_rows.push(state_row_header);
    //     for (index, state) in states.iter().enumerate() {
    //         let now = SystemTime::now()
    //             .duration_since(UNIX_EPOCH)
    //             .unwrap()
    //             .as_micros();
    //         let row = format!("{index},,{state},system,,{now},{now}");
    //         state_rows.push(row);
    //     }
    //     let city_row_header =
    //         "id,city_name,state_id,created_by,updated_by,created_at,updated_at".to_string();
    //     let mut city_rows: Vec<String> = Vec::with_capacity(800);
    //     city_rows.push(city_row_header);
    //     for (index, city) in cities.iter().enumerate() {
    //         let now = SystemTime::now()
    //             .duration_since(UNIX_EPOCH)
    //             .unwrap()
    //             .as_micros();
    //         let state = city_state_map.get(city.as_str()).unwrap();
    //         let state_index = states
    //             .iter()
    //             .enumerate()
    //             .find(|(i, s)| state.eq_ignore_ascii_case(s))
    //             .unwrap()
    //             .0;
    //         let row = format!("{index},{city},{state_index},system,,{now},{now}");
    //         city_rows.push(row);
    //     }
    //
    //     let pincode_header_row =
    //         "id,pincode,city_id,created_by,updated_by,created_at,updated_at".to_string();
    //     let mut pincode_rows = Vec::with_capacity(20_000);
    //     pincode_rows.push(pincode_header_row);
    //     for (index, pincode) in pincodes.iter().enumerate() {
    //         let now = SystemTime::now()
    //             .duration_since(UNIX_EPOCH)
    //             .unwrap()
    //             .as_micros();
    //         let city = pincode_city_map.get(pincode).unwrap();
    //         let city_index = cities
    //             .iter()
    //             .enumerate()
    //             .find(|(i, c)| city.eq_ignore_ascii_case(c))
    //             .unwrap()
    //             .0;
    //         let row = format!("{index},{pincode},{city_index},system,,{now},{now}");
    //         pincode_rows.push(row);
    //     }
    //     let mut state_master_file =File::create("state_master.csv").unwrap();
    //     let mut city_master_file = File::create("city_master.csv").unwrap();
    //     let mut pincode_master_file = File::create("pincode_master.csv").unwrap();
    //     state_master_file.write_all(state_rows.join("\n").as_bytes()).unwrap();
    //     city_master_file.write_all(city_rows.join("\n").as_bytes()).unwrap();
    //     pincode_master_file.write_all(pincode_rows.join("\n").as_bytes()).unwrap();
    //
    //     //created_by,updated_by,created_at,updated_at
    //     //pincode_master
    //     //id,pincode,city_id,created_by,updated_by,created_at,updated_at
    //     //city_master
    //     //id,city_name,state_id,created_by,updated_by,created_at,updated_at
    //     //state_master
    //     //id,state_code,state_name,created_by,updated_by,created_at,updated_at
    //     println!("{}", pincodes.len());
    //     println!("{}", cities.len());
    //     println!("{}", states.len())
    // }
}
