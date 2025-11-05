# Python Backend API Integration Guide

> **Note**: This document describes the **alternative** REST API integration approach. The **primary and recommended** approach is to use **MCP (Model Context Protocol)**. See `MCP_INTEGRATION.md` and `MCP_CLIENT_IMPLEMENTATION.md` for details.
>
> **Why MCP?** With MCP, the LLM autonomously calls backend tools, eliminating the need for complex intent extraction, entity extraction, and manual routing logic. This results in 70% less code and a much simpler architecture.
>
> This REST API documentation is kept for reference if you need direct HTTP access or want to compare approaches.

This document describes how the Rust agent could integrate with the existing Python FastAPI backend via REST API (alternative to MCP).

## Base Configuration

```rust
// Environment variables
BACKEND_API_URL=http://localhost:8000/api
BACKEND_API_TIMEOUT=30  // seconds
```

## API Endpoints Reference

### 1. Business/Salon Search

**Endpoint**: `GET /api/businesses/search`

**Query Parameters**:
- `query` (optional): Search query for business name
- `city` (optional): City filter
- `state` (optional): State filter
- `business_type` (optional): Business type filter
- `latitude` (optional): User latitude for distance sorting
- `longitude` (optional): User longitude for distance sorting
- `skip` (optional, default: 0): Number of results to skip
- `limit` (optional, default: 100): Maximum number of results

**Example Request**:
```rust
let url = format!(
    "{}/businesses/search?query={}&city={}&latitude={}&longitude={}&limit={}",
    backend_url,
    query,
    city,
    latitude,
    longitude,
    limit
);
```

**Response** (`BusinessSearchResponse`):
```rust
struct BusinessSearchResponse {
    businesses: Vec<Business>,
    total: usize,
    skip: usize,
    limit: usize,
}

struct Business {
    id: String,
    name: String,
    description: Option<String>,
    email: String,
    phone: Option<String>,
    address: String,
    city: String,
    state: String,
    zip_code: Option<String>,
    country: String,
    website: Option<String>,
    business_type: String,
    latitude: Option<f64>,
    longitude: Option<f64>,
    google_rating: Option<f32>,
    google_reviews: Vec<Review>,
    images: Vec<String>,
    // ... other fields
}
```

### 2. Get Business Details

**Endpoint**: `GET /api/businesses/{business_id}`

**Example Request**:
```rust
let url = format!("{}/businesses/{}", backend_url, business_id);
```

**Response**: `BusinessResponse` (same as Business in search response)

### 3. Get Services

**Endpoint**: `GET /api/services/`

**Query Parameters**:
- `business_id` (optional): Filter by business ID
- `category` (optional): Filter by service category
- `active_only` (optional, default: true): Only active services
- `skip` (optional, default: 0)
- `limit` (optional, default: 100)

**Example Request**:
```rust
let url = format!(
    "{}/services/?business_id={}&category={}&active_only=true",
    backend_url,
    business_id,
    category
);
```

**Response**: `Vec<ServiceResponse>`
```rust
struct ServiceResponse {
    id: String,
    business_id: String,
    name: String,
    description: Option<String>,
    duration_minutes: i32,
    price: f64,
    category: String,  // "hair", "nails", "spa", "beauty", etc.
    status: String,    // "active", "inactive"
    is_online_booking_enabled: bool,
    requires_consultation: bool,
    // ... other fields
}
```

### 4. Create Booking

**Endpoint**: `POST /api/bookings/`

**Request Body** (`BookingCreate`):
```rust
struct BookingCreate {
    user_id: String,
    business_id: String,
    service_id: String,
    employee_id: Option<String>,
    booking_datetime: DateTime<Utc>,  // ISO 8601 format
    duration_minutes: Option<i32>,    // Defaults to service duration
    price: Option<f64>,               // Defaults to service price
    notes: Option<String>,
    special_requests: Option<String>,
}
```

**Example Request**:
```rust
let booking = BookingCreate {
    user_id: "user_123".to_string(),
    business_id: "business_456".to_string(),
    service_id: "service_789".to_string(),
    employee_id: None,
    booking_datetime: chrono::Utc::now() + chrono::Duration::days(1),
    duration_minutes: Some(60),
    price: Some(50.0),
    notes: Some("Please confirm".to_string()),
    special_requests: None,
};

let response = client
    .post(format!("{}/bookings/", backend_url))
    .json(&booking)
    .send()
    .await?;
```

**Response**: `BookingResponse`
```rust
struct BookingResponse {
    id: String,
    user_id: String,
    business_id: String,
    service_id: String,
    employee_id: Option<String>,
    booking_datetime: DateTime<Utc>,
    duration_minutes: i32,
    price: f64,
    status: String,  // "pending", "confirmed", "cancelled", "completed"
    notes: Option<String>,
    special_requests: Option<String>,
    created_at: DateTime<Utc>,
    // ... other fields
}
```

### 5. Get Bookings

**Endpoint**: `GET /api/bookings/`

**Query Parameters**:
- `user_id` (optional): Filter by user ID
- `business_id` (optional): Filter by business ID
- `status` (optional): Filter by status
- `skip` (optional, default: 0)
- `limit` (optional, default: 100)

**Example Request**:
```rust
let url = format!(
    "{}/bookings/?user_id={}&status=pending",
    backend_url,
    user_id
);
```

**Response**: `Vec<BookingResponse>`

### 6. Confirm Booking

**Endpoint**: `PUT /api/bookings/{booking_id}/confirm`

**Example Request**:
```rust
let url = format!("{}/bookings/{}/confirm", backend_url, booking_id);
let response = client.put(url).send().await?;
```

**Response**: `BookingResponse` (with updated status)

### 7. Cancel Booking

**Endpoint**: `PUT /api/bookings/{booking_id}/cancel`

**Query Parameters** (optional):
- `reason` (optional): Cancellation reason

**Example Request**:
```rust
let url = format!("{}/bookings/{}/cancel?reason={}", backend_url, booking_id, reason);
let response = client.put(url).send().await?;
```

**Response**: `BookingResponse` (with updated status)

## Rust HTTP Client Implementation

### Basic Client Setup

```rust
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub struct BackendApiClient {
    client: Client,
    base_url: String,
}

impl BackendApiClient {
    pub fn new(base_url: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");
        
        Self { client, base_url }
    }
    
    pub async fn search_businesses(
        &self,
        query: Option<&str>,
        city: Option<&str>,
        latitude: Option<f64>,
        longitude: Option<f64>,
        business_type: Option<&str>,
        limit: usize,
    ) -> Result<BusinessSearchResponse, ApiError> {
        let mut url = format!("{}/businesses/search", self.base_url);
        let mut params = Vec::new();
        
        if let Some(q) = query {
            params.push(("query", q));
        }
        if let Some(c) = city {
            params.push(("city", c));
        }
        if let Some(lat) = latitude {
            params.push(("latitude", &lat.to_string()));
        }
        if let Some(lng) = longitude {
            params.push(("longitude", &lng.to_string()));
        }
        if let Some(bt) = business_type {
            params.push(("business_type", bt));
        }
        params.push(("limit", &limit.to_string()));
        
        let response = self.client
            .get(&url)
            .query(&params)
            .send()
            .await?;
        
        let search_response: BusinessSearchResponse = response.json().await?;
        Ok(search_response)
    }
    
    pub async fn get_business(&self, business_id: &str) -> Result<Business, ApiError> {
        let url = format!("{}/businesses/{}", self.base_url, business_id);
        let response = self.client.get(&url).send().await?;
        let business: Business = response.json().await?;
        Ok(business)
    }
    
    pub async fn get_services(
        &self,
        business_id: Option<&str>,
        category: Option<&str>,
    ) -> Result<Vec<ServiceResponse>, ApiError> {
        let mut url = format!("{}/services/", self.base_url);
        let mut params = Vec::new();
        
        if let Some(bid) = business_id {
            params.push(("business_id", bid));
        }
        if let Some(cat) = category {
            params.push(("category", cat));
        }
        params.push(("active_only", "true"));
        
        let response = self.client
            .get(&url)
            .query(&params)
            .send()
            .await?;
        
        let services: Vec<ServiceResponse> = response.json().await?;
        Ok(services)
    }
    
    pub async fn create_booking(
        &self,
        booking: &BookingCreate,
    ) -> Result<BookingResponse, ApiError> {
        let url = format!("{}/bookings/", self.base_url);
        let response = self.client
            .post(&url)
            .json(booking)
            .send()
            .await?;
        
        let booking_response: BookingResponse = response.json().await?;
        Ok(booking_response)
    }
    
    pub async fn get_bookings(
        &self,
        user_id: Option<&str>,
        business_id: Option<&str>,
        status: Option<&str>,
    ) -> Result<Vec<BookingResponse>, ApiError> {
        let mut url = format!("{}/bookings/", self.base_url);
        let mut params = Vec::new();
        
        if let Some(uid) = user_id {
            params.push(("user_id", uid));
        }
        if let Some(bid) = business_id {
            params.push(("business_id", bid));
        }
        if let Some(s) = status {
            params.push(("status", s));
        }
        
        let response = self.client
            .get(&url)
            .query(&params)
            .send()
            .await?;
        
        let bookings: Vec<BookingResponse> = response.json().await?;
        Ok(bookings)
    }
    
    pub async fn confirm_booking(&self, booking_id: &str) -> Result<BookingResponse, ApiError> {
        let url = format!("{}/bookings/{}/confirm", self.base_url, booking_id);
        let response = self.client.put(&url).send().await?;
        let booking: BookingResponse = response.json().await?;
        Ok(booking)
    }
    
    pub async fn cancel_booking(
        &self,
        booking_id: &str,
        reason: Option<&str>,
    ) -> Result<BookingResponse, ApiError> {
        let mut url = format!("{}/bookings/{}/cancel", self.base_url, booking_id);
        if let Some(r) = reason {
            url = format!("{}?reason={}", url, r);
        }
        let response = self.client.put(&url).send().await?;
        let booking: BookingResponse = response.json().await?;
        Ok(booking)
    }
}
```

## Error Handling

```rust
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),
    
    #[error("JSON deserialization failed: {0}")]
    Json(#[from] serde_json::Error),
    
    #[error("Backend API error: {0}")]
    Backend(String),
    
    #[error("Business not found")]
    NotFound,
    
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
}
```

## Authentication (Future)

If the Python backend requires authentication for certain endpoints:

```rust
impl BackendApiClient {
    pub fn with_auth(mut self, token: String) -> Self {
        self.client = Client::builder()
            .default_headers({
                let mut headers = reqwest::header::HeaderMap::new();
                headers.insert(
                    reqwest::header::AUTHORIZATION,
                    format!("Bearer {}", token).parse().unwrap(),
                );
                headers
            })
            .build()
            .unwrap();
        self
    }
}
```

## Usage Example

```rust
let backend_client = BackendApiClient::new("http://localhost:8000/api".to_string());

// Search for salons
let businesses = backend_client
    .search_businesses(
        Some("haircut"),
        Some("Athens"),
        Some(37.9838),
        Some(23.7275),
        None,
        10,
    )
    .await?;

// Get services for a business
let services = backend_client
    .get_services(Some(&businesses.businesses[0].id), Some("hair"))
    .await?;

// Create a booking
let booking = BookingCreate {
    user_id: "user_123".to_string(),
    business_id: businesses.businesses[0].id.clone(),
    service_id: services[0].id.clone(),
    employee_id: None,
    booking_datetime: chrono::Utc::now() + chrono::Duration::days(1),
    duration_minutes: Some(services[0].duration_minutes),
    price: Some(services[0].price),
    notes: None,
    special_requests: None,
};

let created_booking = backend_client.create_booking(&booking).await?;
```

