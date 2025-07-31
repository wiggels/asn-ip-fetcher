# ASN Data Fetcher

A Rust utility that fetches Autonomous System Number (ASN) data including display names and IP ranges from the [ipverse/asn-ip](https://github.com/ipverse/asn-ip) repository and outputs the results to a JSON file.

## Features

- Fetches ASN data asynchronously for improved performance
- Supports both IPv4 and IPv6 subnets
- Automatically splits large ASNs (>2000 IP ranges) into multiple entries
- Configurable ASN list via YAML file
- Outputs clean, pretty-printed JSON

## Dependencies

```toml
[dependencies]
reqwest = { version = "0.11", features = ["json"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
serde_yaml = "0.9"
tokio = { version = "1.0", features = ["full"] }
```

## Configuration

Create a YAML file with your ASN list:

```yaml
asn:
  - "13335"  # Cloudflare
  - "15169"  # Google
  - "16509"  # Amazon
  - "32934"  # Facebook
```

## Usage

### Basic Usage

```bash
# Uses default_asn_list.yml
cargo run
```

### Custom ASN List

```bash
# Use a custom YAML file
GET_ASN_LIST=my_asn_list.yml cargo run
```

## Environment Variables

- `GET_ASN_LIST`: Path to the YAML file containing the ASN list (default: `default_asn_list.yml`)

## Output

The program generates `asn_data.json` with the following structure:

```json
{
  "13335": {
    "display_name": "CLOUDFLARENET",
    "ip_ranges": [
      "1.1.1.0/24",
      "1.0.0.0/24",
      "2606:4700::/32"
    ]
  }
}
```

### Large ASN Handling

ASNs with more than 2000 IP ranges are automatically split into multiple entries:

```json
{
  "16509": {
    "display_name": "AMAZON-02",
    "ip_ranges": ["...first 2000 ranges..."]
  },
  "16509_2": {
    "display_name": "AMAZON-02_2",
    "ip_ranges": ["...next 2000 ranges..."]
  }
}
```

## Data Source

This tool fetches data from the [ipverse/asn-ip](https://github.com/ipverse/asn-ip) repository, which provides regularly updated ASN to IP range mappings in JSON format.

## Error Handling

- If an ASN is not found, it will be logged with "ASN {number} not found" and empty IP ranges
- Only ASNs with valid IP ranges are included in the output
- Network errors will cause the program to panic -- TODO: fix that!

## Example YAML Configuration

```yaml
# default_asn_list.yml
asn:
  - "13335"  # Cloudflare
  - "15169"  # Google LLC
  - "16509"  # Amazon.com, Inc.
  - "32934"  # Facebook, Inc.
  - "8075"   # Microsoft Corporation
  - "2906"   # Netflix, Inc.
```

## Building and Running

```bash
# Build the project
cargo build --release

# Run with default configuration
cargo run

# Run with custom ASN list
GET_ASN_LIST=custom_list.yml cargo run
```
