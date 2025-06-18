use actix_web::{App, HttpResponse, HttpServer, Result, web};
use chrono::{DateTime, Utc};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use tokio::signal;
use tracing::info;
use tracing_actix_web::TracingLogger;
use tracing_subscriber::{EnvFilter, fmt};

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Call {
    id: String,
    summary: String,
    duration: i32, // Duration in seconds
    associated_deal_ids: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Deal {
    id: String,
    name: String,
    created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Product {
    id: String,
    name: String,
    description: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Order {
    id: String,
    created_at: DateTime<Utc>,
    deal_id: String,
    name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct LineItem {
    id: String,
    quantity: i32,
    product_id: String,
}

#[derive(Debug, Serialize)]
struct ApiResponse<T> {
    items: Vec<DataWrapper<T>>,
}

#[derive(Debug, Serialize)]
struct DataWrapper<T> {
    data: T,
}

// Dummy data
static CALLS: Lazy<Vec<Call>> = Lazy::new(|| {
    vec![
        Call {
            id: "call-1".to_string(),
            summary: "Initial sales call with Acme Corp".to_string(),
            duration: 1800,
            associated_deal_ids: vec!["deal-1".to_string(), "deal-2".to_string()],
        },
        Call {
            id: "call-2".to_string(),
            summary: "Follow-up call with TechStart Inc".to_string(),
            duration: 2400,
            associated_deal_ids: vec!["deal-3".to_string()],
        },
        Call {
            id: "call-3".to_string(),
            summary: "Product demo for Enterprise Solutions".to_string(),
            duration: 3600,
            associated_deal_ids: vec!["deal-1".to_string(), "deal-4".to_string()],
        },
    ]
});

static DEALS: Lazy<BTreeMap<String, Deal>> = Lazy::new(|| {
    vec![
        Deal {
            id: "deal-1".to_string(),
            name: "Acme Corp Enterprise License".to_string(),
            created_at: DateTime::parse_from_rfc3339("2023-12-16T10:30:00Z")
                .unwrap()
                .with_timezone(&Utc),
        },
        Deal {
            id: "deal-2".to_string(),
            name: "Acme Corp Support Package".to_string(),
            created_at: DateTime::parse_from_rfc3339("2023-12-21T10:30:00Z")
                .unwrap()
                .with_timezone(&Utc),
        },
        Deal {
            id: "deal-3".to_string(),
            name: "TechStart Inc Starter Plan".to_string(),
            created_at: DateTime::parse_from_rfc3339("2023-12-31T10:30:00Z")
                .unwrap()
                .with_timezone(&Utc),
        },
        Deal {
            id: "deal-4".to_string(),
            name: "Enterprise Solutions Premium".to_string(),
            created_at: DateTime::parse_from_rfc3339("2024-01-10T10:30:00Z")
                .unwrap()
                .with_timezone(&Utc),
        },
    ]
    .into_iter()
    .map(|d| (d.id.clone(), d))
    .collect()
});

static PRODUCTS: Lazy<BTreeMap<String, Product>> = Lazy::new(|| {
    vec![
        Product {
            id: "prod-1".to_string(),
            name: "CRM Pro License".to_string(),
            description: "Professional CRM software license with advanced features".to_string(),
        },
        Product {
            id: "prod-2".to_string(),
            name: "Support Package Gold".to_string(),
            description: "24/7 premium support with dedicated account manager".to_string(),
        },
        Product {
            id: "prod-3".to_string(),
            name: "Integration Module".to_string(),
            description: "API integration module for third-party services".to_string(),
        },
        Product {
            id: "prod-4".to_string(),
            name: "Training Package".to_string(),
            description: "Comprehensive training program for teams".to_string(),
        },
    ]
    .into_iter()
    .map(|p| (p.id.clone(), p))
    .collect()
});

static ORDERS: Lazy<BTreeMap<String, Order>> = Lazy::new(|| {
    vec![
        Order {
            id: "order-1".to_string(),
            name: "Initial Purchase Order".to_string(),
            created_at: DateTime::parse_from_rfc3339("2023-12-26T10:30:00Z")
                .unwrap()
                .with_timezone(&Utc),
            deal_id: "deal-1".to_string(),
        },
        Order {
            id: "order-2".to_string(),
            name: "Additional Licenses".to_string(),
            created_at: DateTime::parse_from_rfc3339("2024-01-05T10:30:00Z")
                .unwrap()
                .with_timezone(&Utc),
            deal_id: "deal-1".to_string(),
        },
        Order {
            id: "order-3".to_string(),
            name: "Support Services".to_string(),
            created_at: DateTime::parse_from_rfc3339("2024-01-15T10:30:00Z")
                .unwrap()
                .with_timezone(&Utc),
            deal_id: "deal-2".to_string(),
        },
        Order {
            id: "order-4".to_string(),
            name: "Starter Package".to_string(),
            created_at: DateTime::parse_from_rfc3339("2024-01-01T10:30:00Z")
                .unwrap()
                .with_timezone(&Utc),
            deal_id: "deal-3".to_string(),
        },
    ]
    .into_iter()
    .map(|o| (o.id.clone(), o))
    .collect()
});

static LINE_ITEMS: Lazy<BTreeMap<String, Vec<LineItem>>> = Lazy::new(|| {
    let mut items = BTreeMap::new();

    items.insert(
        "order-1".to_string(),
        vec![
            LineItem {
                id: "line-1".to_string(),
                quantity: 10,
                product_id: "prod-1".to_string(),
            },
            LineItem {
                id: "line-2".to_string(),
                quantity: 1,
                product_id: "prod-3".to_string(),
            },
        ],
    );

    items.insert(
        "order-2".to_string(),
        vec![LineItem {
            id: "line-3".to_string(),
            quantity: 5,
            product_id: "prod-1".to_string(),
        }],
    );

    items.insert(
        "order-3".to_string(),
        vec![
            LineItem {
                id: "line-4".to_string(),
                quantity: 1,
                product_id: "prod-2".to_string(),
            },
            LineItem {
                id: "line-5".to_string(),
                quantity: 2,
                product_id: "prod-4".to_string(),
            },
        ],
    );

    items.insert(
        "order-4".to_string(),
        vec![LineItem {
            id: "line-6".to_string(),
            quantity: 3,
            product_id: "prod-1".to_string(),
        }],
    );

    items
});

// Handler functions
async fn get_calls() -> Result<HttpResponse> {
    info!("GET /v2/calls");
    let response = ApiResponse {
        items: CALLS.iter().map(|call| DataWrapper { data: call.clone() }).collect(),
    };
    Ok(HttpResponse::Ok().json(response))
}

async fn get_deal(path: web::Path<String>) -> Result<HttpResponse> {
    let deal_id = path.into_inner();
    info!("GET /v2/deals/{} - deal_id: {}", deal_id, deal_id);

    if let Some(deal) = DEALS.get(&deal_id) {
        Ok(HttpResponse::Ok().json(DataWrapper { data: deal.clone() }))
    } else {
        Ok(HttpResponse::NotFound().finish())
    }
}

#[derive(Deserialize)]
struct IdsQuery {
    ids: String,
}

async fn get_products(query: web::Query<IdsQuery>) -> Result<HttpResponse> {
    let ids: Vec<String> = query.ids.split(',').map(|s| s.trim().to_string()).collect();
    info!("GET /v2/products - ids: {:?}", ids);

    let response = ApiResponse {
        items: ids
            .iter()
            .filter_map(|id| PRODUCTS.get(id))
            .map(|product| DataWrapper { data: product.clone() })
            .collect(),
    };

    Ok(HttpResponse::Ok().json(response))
}

async fn get_deals(query: web::Query<IdsQuery>) -> Result<HttpResponse> {
    let ids: Vec<String> = query.ids.split(',').map(|s| s.trim().to_string()).collect();
    info!("GET /v2/deals - ids: {:?}", ids);

    let response = ApiResponse {
        items: ids
            .iter()
            .filter_map(|id| DEALS.get(id))
            .map(|deal| DataWrapper { data: deal.clone() })
            .collect(),
    };

    Ok(HttpResponse::Ok().json(response))
}

#[derive(Deserialize)]
struct DealIdQuery {
    deal_id: String,
}

async fn get_orders(query: web::Query<DealIdQuery>) -> Result<HttpResponse> {
    info!("GET /v2/orders - deal_id: {}", query.deal_id);
    let orders: Vec<Order> = ORDERS
        .values()
        .filter(|order| order.deal_id == query.deal_id)
        .cloned()
        .collect();

    let response = ApiResponse {
        items: orders.into_iter().map(|order| DataWrapper { data: order }).collect(),
    };

    Ok(HttpResponse::Ok().json(response))
}

async fn get_line_items(path: web::Path<String>) -> Result<HttpResponse> {
    let order_id = path.into_inner();
    info!("GET /v2/orders/{}/line_items - order_id: {}", order_id, order_id);

    let items = LINE_ITEMS.get(&order_id).cloned().unwrap_or_default();

    let response = ApiResponse {
        items: items.into_iter().map(|item| DataWrapper { data: item }).collect(),
    };

    Ok(HttpResponse::Ok().json(response))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize tracing subscriber
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    fmt()
        .with_env_filter(env_filter)
        .with_target(false)
        .with_thread_ids(false)
        .with_thread_names(false)
        .with_file(false)
        .with_line_number(false)
        .init();

    info!("Starting Zendesk CRM API on http://localhost:8080");

    let server = HttpServer::new(|| {
        App::new()
            .wrap(TracingLogger::default())
            .route("/v2/calls", web::get().to(get_calls))
            .route("/v2/deals/{id}", web::get().to(get_deal))
            .route("/v2/products", web::get().to(get_products))
            .route("/v2/deals", web::get().to(get_deals))
            .route("/v2/orders", web::get().to(get_orders))
            .route("/v2/orders/{id}/line_items", web::get().to(get_line_items))
    })
    .bind("127.0.0.1:8080")?
    .run();

    // Get server handle for graceful shutdown
    let server_handle = server.handle();

    // Spawn the server
    let server_task = tokio::spawn(server);

    // Set up signal handlers for graceful shutdown
    let shutdown_signal = async {
        let ctrl_c = async {
            signal::ctrl_c().await.expect("failed to install Ctrl+C handler");
        };

        #[cfg(unix)]
        let terminate = async {
            signal::unix::signal(signal::unix::SignalKind::terminate())
                .expect("failed to install signal handler")
                .recv()
                .await;
        };

        #[cfg(not(unix))]
        let terminate = std::future::pending::<()>();

        tokio::select! {
            _ = ctrl_c => {
                info!("Received Ctrl+C, starting graceful shutdown...");
            },
            _ = terminate => {
                info!("Received SIGTERM, starting graceful shutdown...");
            },
        }
    };

    // Wait for shutdown signal
    shutdown_signal.await;

    // Initiate graceful shutdown
    server_handle.stop(true).await;
    info!("Server stopped gracefully");

    // Wait for the server task to complete
    let _ = server_task.await;

    Ok(())
}
