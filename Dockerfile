FROM ubuntu:latest

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

ENV SSL_CERT_FILE=/etc/ssl/certs/ca-certificates.crt

COPY default_asn_list.yml /default_asn_list.yml
RUN chmod 644 /default_asn_list.yml

COPY asn-ip-fetcher /usr/bin/asn-ip-fetcher
RUN chmod 755 /usr/bin/asn-ip-fetcher

ENTRYPOINT []
