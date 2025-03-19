#!/usr/bin/env bash

# Inspired by:
# https://github.com/emarsden/dash-mpd-cli/blob/main/tests/create-certs.sh
# https://github.com/seanmonstar/reqwest/issues/2041#issuecomment-1826386492

set -euxo pipefail

echo "Cleaning up exisitng certificates"
rm -rf certificates || true
mkdir -p certificates

pushd certificates

# Create the certificates necessary for our tests:
#   - a root certificate authority
#   - a server running on localhost
#   - a client making authenticated requests to the server
#
# We generate these with openssl, but could try using the rcgen crate.
#
# To dump the content of a certificate, openssl x509 -in cert.csr -text
#
openssl genrsa -aes256 -passout pass:'grafbase' -out root-CA.key 4096
openssl genrsa -aes256 -passout pass:'grafbase' -out server.key 4096
openssl genrsa -aes256 -passout pass:'grafbase' -out client.key 4096

# create the certificate for the root Certificate Authority
openssl req -x509 -new -nodes \
    -passin pass:'grafbase' \
    -sha512 -days 1000 \
    -subj "/C=FR/L=Toulouse/O=Test" \
    -addext "basicConstraints=critical,CA:true,pathlen:0" \
    -addext "keyUsage=critical,keyCertSign,cRLSign" \
    -key root-CA.key \
    -out root-CA.crt

# Create the certificate for the server on server. Note that rustls is finicky, requiring the
# subjectAltName field to be present.
openssl req -new -sha512 \
    -passin pass:'grafbase' \
    -subj "/C=FR/L=Toulouse/O=Test/CN=localhost" \
    -addext 'subjectAltName=DNS:localhost' \
    -addext 'basicConstraints=critical,CA:FALSE' \
    -addext 'extendedKeyUsage=serverAuth' \
    -key server.key \
    -out server.csr
openssl x509 -req \
    -passin pass:'grafbase' \
    -CAcreateserial -days 1000 -sha512 -copy_extensions copy \
    -in server.csr \
    -CA root-CA.crt \
    -CAkey root-CA.key \
    -out server.crt

# create the certificate for the client
openssl req -new -sha512 -nodes \
    -passin pass:'grafbase' \
    -subj "/C=FR/L=Toulouse/O=Test/CN=local-test-client" \
    -addext "basicConstraints=critical,CA:false" \
    -addext "extendedKeyUsage=clientAuth" \
    -key client.key \
    -out client.csr
openssl x509 -req \
    -passin pass:'grafbase' \
    -CAcreateserial -days 1000 -sha512 -copy_extensions copy \
    -CA root-CA.crt \
    -CAkey root-CA.key \
    -in client.csr \
    -out client.crt

# We only need those today.
openssl rsa -passin pass:'grafbase' -in server.key -text >server-key.pem
openssl x509 -inform PEM -in server.crt >server-crt.pem
openssl x509 -inform PEM -in root-CA.crt >root-CA-crt.pem

popd

echo "Cleaning non-pem files"
find certificates -type f -not -name "*.pem" -exec rm {} \;
