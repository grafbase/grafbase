#!/bin/sh

# Clean up previous certs
rm -rf certs
mkdir certs

# --- CA Certificate ---
echo "Generating CA certificate..."
openssl req -x509 -newkey rsa:4096 \
    -keyout certs/ca-key.pem \
    -out certs/ca-cert.pem \
    -days 3650 -nodes \
    -subj "/CN=My Test CA"
echo "CA certificate generated."

# --- Server Certificate ---
echo "Generating server certificate..."
# Create server.conf for SANs
cat > certs/server.conf <<EOF
[req]
distinguished_name = req_distinguished_name
req_extensions = v3_req
prompt = no

[req_distinguished_name]
CN = localhost

[v3_req]
subjectAltName = @alt_names

[alt_names]
DNS.1 = localhost
IP.1 = 127.0.0.1
EOF

# Generate server key and CSR
openssl req -newkey rsa:4096 \
    -keyout certs/server-key.pem \
    -out certs/server-req.pem \
    -nodes \
    -config certs/server.conf # Use the config file for SANs in the CSR

# Sign the server CSR with the CA
openssl x509 -req \
    -in certs/server-req.pem \
    -days 3650 \
    -CA certs/ca-cert.pem \
    -CAkey certs/ca-key.pem \
    -CAcreateserial \
    -out certs/server-cert.pem \
    -extensions v3_req -extfile certs/server.conf # Apply SANs from config during signing
echo "Server certificate generated and signed."

# --- Client Certificate (mTLS) ---
echo "Generating client certificate..."
# Generate client key and CSR
openssl req -newkey rsa:4096 \
    -keyout certs/client-key.pem \
    -out certs/client-req.pem \
    -nodes \
    -subj "/CN=client"

# Sign the client CSR with the CA
# The -CAcreateserial option will create/use a .srl file (e.g., certs/ca-cert.srl)
# to track serial numbers for certificates issued by this CA.
openssl x509 -req \
    -in certs/client-req.pem \
    -days 3650 \
    -CA certs/ca-cert.pem \
    -CAkey certs/ca-key.pem \
    -CAcreateserial \
    -out certs/client-cert.pem
echo "Client certificate generated and signed."

# Create client identity file (certificate + private key)
cat certs/client-cert.pem > certs/client-identity.pem
cat certs/client-key.pem >> certs/client-identity.pem
echo "Client identity file created."

# Optional: Clean up CSRs, conf file, and serial file as they are intermediate
# rm certs/server-req.pem certs/client-req.pem certs/server.conf certs/ca-cert.srl

echo "Certificates generated successfully."
echo "CA: certs/ca-cert.pem"
echo "Server Cert: certs/server-cert.pem, Server Key: certs/server-key.pem"
echo "Client Identity (for reqwest): certs/client-identity.pem"
