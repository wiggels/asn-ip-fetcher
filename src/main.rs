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

async fn fetch_asn_data(client: &Client, asn: &str) -> AsnData {
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
            ip_ranges: data["subnets"]["ipv4"]
                .as_array()
                .unwrap_or(&vec![])
                .iter()
                .chain(data["subnets"]["ipv6"].as_array().unwrap_or(&vec![]).iter())
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

async fn fetch_all_asns(asn_list: &Vec<String>) -> HashMap<String, AsnData> {
    let client: Client = Client::new();
    let mut tasks: Vec<tokio::task::JoinHandle<(String, AsnData)>> = vec![];
    for asn in asn_list {
        let client_clone: Client = client.clone();
        let asn_clone: String = asn.clone();
        tasks.push(tokio::spawn(async move {
            (
                asn_clone.clone(),
                fetch_asn_data(&client_clone, &asn_clone).await,
            )
        }));
    }
    let mut asn_data: HashMap<String, AsnData> = HashMap::new();
    for task in tasks {
        let (asn, data) = task.await.unwrap();
        if !data.ip_ranges.is_empty() {
            if data.ip_ranges.len() > 2000 {
                for (i, chunk) in data.ip_ranges.chunks(2000).enumerate() {
                    let asn_suffix = if i == 0 {
                        asn.clone()
                    } else {
                        format!("{}_{}", asn, i + 1)
                    };
                    asn_data.insert(asn_suffix, AsnData {
                        display_name: data.display_name.clone(),
                        ip_ranges: chunk.to_vec(),
                    });
                }
            } else {
                asn_data.insert(asn, data);
            }
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
    let asn_data: HashMap<String, AsnData> = fetch_all_asns(&asn_list).await;
    let json_file: File = File::create("asn_data.json").unwrap();
    serde_json::to_writer_pretty(json_file, &asn_data).unwrap();
}
