# Grafbase tracing example

This example showcases how to configure tracing with a federated graph.

## Overview

The example is composed of the following:

- Grafbase gateway to serve the federated graph
- OTEL collector to collect tracing data
- Grafana Tempo to store tracing data
- Prometheus to send traces
- Grafana to explore traces

## Running

docker-compose up

- publish the subgraph schema
- open up Grafana at http://localhost:3000/
- issue some requests to the federated graoh
