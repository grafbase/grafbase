# Grafbase tracing example

This example showcases how to configure tracing when using `grafbase federated-dev`.

## Overview

The example is composed of the following:

- grafbase/v1 is a simple graphql schema that calls a resolver to resolve a field
- grafbase/v2 is a federated graphql schema
- docker-compose.yaml that spins up the following
  * an otel collector to send tracing data to
  * a grafana-tempo to store the tracing data
  * a prometheus instance to send tracing metrics 
  * a grafana to explore

## Running

    cd examples/tracing/
    docker-compose up

    # Refer to the README's of each `grafbase/` projects to run them.

- open up grafana at http://localhost:3000/
- issue some requests to the federated server 

