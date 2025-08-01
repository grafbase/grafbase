use tonic::{transport::Server, Request, Response, Status};
use std::env;

pub mod locations {
    tonic::include_proto!("locations");
}

use locations::{
    location_service_server::{LocationService, LocationServiceServer},
    BatchGetLocationsRequest, BatchGetLocationsResponse, GetLocationRequest, GetLocationResponse,
    Location,
};

#[derive(Debug)]
struct LocationsService {
    locations: Vec<Location>,
}

impl LocationsService {
    fn new() -> Self {
        // Hardcoded locations data
        let locations = vec![
            Location {
                id: "loc-001".to_string(),
                name: "Seattle Distribution Center".to_string(),
                address: "1234 Industrial Way".to_string(),
                city: "Seattle".to_string(),
                state: "WA".to_string(),
                country: "USA".to_string(),
                postal_code: "98101".to_string(),
                capacity: 10000,
                manager_name: "John Smith".to_string(),
                contact_phone: "+1-206-555-0100".to_string(),
                is_active: true,
            },
            Location {
                id: "loc-002".to_string(),
                name: "Portland Warehouse".to_string(),
                address: "5678 Commerce Street".to_string(),
                city: "Portland".to_string(),
                state: "OR".to_string(),
                country: "USA".to_string(),
                postal_code: "97201".to_string(),
                capacity: 7500,
                manager_name: "Sarah Johnson".to_string(),
                contact_phone: "+1-503-555-0200".to_string(),
                is_active: true,
            },
            Location {
                id: "loc-003".to_string(),
                name: "San Francisco Hub".to_string(),
                address: "9012 Market Boulevard".to_string(),
                city: "San Francisco".to_string(),
                state: "CA".to_string(),
                country: "USA".to_string(),
                postal_code: "94102".to_string(),
                capacity: 8500,
                manager_name: "Michael Chen".to_string(),
                contact_phone: "+1-415-555-0300".to_string(),
                is_active: true,
            },
        ];

        LocationsService { locations }
    }
}

#[tonic::async_trait]
impl LocationService for LocationsService {
    async fn get_location(
        &self,
        request: Request<GetLocationRequest>,
    ) -> Result<Response<GetLocationResponse>, Status> {
        let req = request.into_inner();
        
        match self.locations.iter().find(|loc| loc.id == req.id) {
            Some(location) => Ok(Response::new(GetLocationResponse {
                location: Some(location.clone()),
            })),
            None => Err(Status::not_found(format!(
                "Location with id {} not found",
                req.id
            ))),
        }
    }

    async fn batch_get_locations(
        &self,
        request: Request<BatchGetLocationsRequest>,
    ) -> Result<Response<BatchGetLocationsResponse>, Status> {
        let req = request.into_inner();
        
        let locations: Vec<Location> = self
            .locations
            .iter()
            .filter(|loc| req.ids.contains(&loc.id))
            .cloned()
            .collect();

        Ok(Response::new(BatchGetLocationsResponse { locations }))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let port = env::var("PORT").unwrap_or_else(|_| "50053".to_string());
    let addr = format!("0.0.0.0:{}", port).parse()?;

    let locations_service = LocationsService::new();

    println!("Locations service (Rust) running on port {}", port);

    Server::builder()
        .add_service(LocationServiceServer::new(locations_service))
        .serve(addr)
        .await?;

    Ok(())
}