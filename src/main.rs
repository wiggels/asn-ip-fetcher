use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_yaml::from_reader;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
struct AsnData {
    display_name: String,
    ip_ranges: Vec<String>,
}

async fn fetch_asn_data(client: &Client, asn: &str, version: &str) -> AsnData {
    let url: String = format!(
        "https://raw.githubusercontent.com/ipverse/asn-ip/master/as/{}/aggregated.json",
        asn
    );
    let response: reqwest::Response = client.get(&url).send().await.unwrap();

    if response.status().is_success() {
        let text: String = response.text().await.unwrap();
        let data: Value = serde_json::from_str(&text).unwrap();
        AsnData {
            display_name: data["handle"].as_str().unwrap_or("").to_string(),
            ip_ranges: data["subnets"][version]
                .as_array()
                .unwrap_or(&vec![])
                .iter()
                .map(|ip: &Value| ip.as_str().unwrap().to_string())
                .collect(),
        }
    } else {
        AsnData {
            display_name: format!("ASN {} not found", asn),
            ip_ranges: vec![],
        }
    }
}

async fn fetch_all_asns(asn_list: &Vec<String>, version: &str) -> HashMap<String, AsnData> {
    let version_owned = version.to_string();
    let client: Client = Client::new();
    let mut tasks: Vec<tokio::task::JoinHandle<(String, AsnData)>> = vec![];
    for asn in asn_list {
        let version_clone = version_owned.clone();
        let client_clone: Client = client.clone();
        let asn_clone: String = asn.clone();
        tasks.push(tokio::spawn(async move {
            (
                asn_clone.clone(),
                fetch_asn_data(&client_clone, &asn_clone, &version_clone).await,
            )
        }));
    }
    let mut asn_data: HashMap<String, AsnData> = HashMap::new();
    for task in tasks {
        let (asn, data) = task.await.unwrap();
        if !data.ip_ranges.is_empty() {
            asn_data.insert(asn, data);
        }
    }
    asn_data
}

fn read_asn_list_from_yaml(file_path: &Path) -> Vec<String> {
    let file: File = File::open(file_path).unwrap();
    let yaml_content: HashMap<String, Vec<String>> = from_reader(file).unwrap();
    yaml_content.get("asn").cloned().unwrap_or_default()
}

#[tokio::main]
async fn main() {
    let yaml_filename: String =
        env::var("GET_ASN_LIST").unwrap_or_else(|_| "default_asn_list.yml".to_string());
    let asn_list: Vec<String> = read_asn_list_from_yaml(Path::new(&yaml_filename));
    let asn_data_ipv4: HashMap<String, AsnData> = fetch_all_asns(&asn_list, "ipv4").await;
    let json_file_ipv4: File = File::create("asn_data_ipv4.json").unwrap();
    serde_json::to_writer_pretty(json_file_ipv4, &asn_data_ipv4).unwrap();
    let asn_data_ipv6: HashMap<String, AsnData> = fetch_all_asns(&asn_list, "ipv6").await;
    let json_file_ipv6: File = File::create("asn_data_ipv6.json").unwrap();
    serde_json::to_writer_pretty(json_file_ipv6, &asn_data_ipv6).unwrap();
}
