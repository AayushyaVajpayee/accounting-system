use std::collections::HashMap;
use lazy_static::lazy_static;
use rand::{Rng, thread_rng};
use rand::distributions::Alphanumeric;
use regex::Regex;
use serde::Serialize;
use uuid::Uuid;
use crate::LOCAL_HOST;


const GST_STATE_CODE_LIST: [u16; 39] = [
    1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 26,
    27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 97, 99,
];
const ALPHABETS: &[u8] = b"ABCDEFGHIJKLNMNOPQRSTUVWXYZ";
const SEED_GSTIN: &str = "05AABCA5291p1ZD";

lazy_static! {
    static ref REGEX: Regex =
        Regex::new("\\d{2}[a-zA-Z]{5}\\d{4}[a-zA-Z]{1}[a-zA-Z\\d]{1}[zZ]{1}[a-zA-Z\\d]{1}")
            .unwrap();
}
static CONVERSION_TABLE: &str = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ";

lazy_static! {
    static ref ALPHABET_TO_INT_MAP: HashMap<char, usize> = CONVERSION_TABLE
        .chars()
        .enumerate()
        .map(|(a, i)| (i, a))
        .collect::<HashMap<char, usize>>();
}
lazy_static! {
    static ref INT_TO_ALPHABET_MAP: HashMap<usize, char> = CONVERSION_TABLE
        .chars()
        .enumerate()
        .map(|(a, i)| (a, i))
        .collect::<HashMap<usize, char>>();
}
pub fn gstin_checksum(gstin: &str) -> Result<char, &str> {
    let gstin = gstin.to_uppercase();
    let checked_digit = gstin.chars().nth(14);
    if checked_digit.is_none() {
        return Err("less than 14 chars in gstin. cannot calculate checksum");
    }
    let candidate = gstin.chars().take(14);
    let mut multiply_by_2 = false;
    let mut hash_sum = 0;
    for char in candidate {
        if multiply_by_2 {
            let value = ALPHABET_TO_INT_MAP.get(&char).unwrap();
            let product = value * 2;
            let quotient = product / 36;
            let remainder = product % 36;
            hash_sum = hash_sum + quotient + remainder;
            multiply_by_2 = false;
        } else {
            let value = ALPHABET_TO_INT_MAP.get(&char).unwrap();
            let product = value * 1;
            let quotient = product / 36;
            let remainder = product % 36;
            hash_sum = hash_sum + quotient + remainder;
            multiply_by_2 = true;
        }
    }
    let hash_sum_remainder = hash_sum % 36;
    let check_digit = (36 - hash_sum_remainder) % 36;
    let check_alpha = INT_TO_ALPHABET_MAP.get(&check_digit).unwrap();
    Ok(*check_alpha)
}
pub fn generate_random_gstin_no() -> String {
    let mut rng = rand::thread_rng();
    let gst_idx = rng.gen_range(0..GST_STATE_CODE_LIST.len());
    let gst_state_code = format!("{:0>2}", GST_STATE_CODE_LIST[gst_idx]);
    let gst_mid_random_part = (0..5)
        .map(|_| {
            let idx = rng.gen_range(0..ALPHABETS.len());
            ALPHABETS[idx] as char
        })
        .collect::<String>();
    let mut new_gst = format!(
        "{}{}{}",
        gst_state_code,
        gst_mid_random_part,
        &SEED_GSTIN[7..]
    );
    let check_sum = gstin_checksum(new_gst.as_str()).unwrap();
    new_gst.remove(14);
    new_gst.push(check_sum);
    new_gst
}
pub fn generate_random_string_of_numbers(len: usize) -> String {
    let mut rng = rand::thread_rng();
    (0..len)
        .map(|_| rng.gen_range(0..9).to_string())
        .collect()
}

pub fn generate_random_string(len: usize) -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .map(char::from)
        .collect()
}

pub async fn send_request<T: Serialize>(request: &T, tenant_id: Uuid, user_id: Uuid, path: &str) -> Uuid {
    let cli = reqwest::Client::new();
    let path = format!("{}{}", LOCAL_HOST, path);

    let req = cli
        .post(path.as_str())
        .json(request)
        .header("x-acc-tenant-id", tenant_id.to_string())
        .header("x-acc-user-id", user_id.to_string())
        .send()
        .await
        .unwrap();
    let d: Uuid = req.json().await.unwrap();
    d
}