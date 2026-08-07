# OMNIVERSA API

## REST Endpoints

### GET /health
Health check endpoint.

### POST /v1/events/tick
Inject a market tick event.

### GET /v1/fragility
Current fragility snapshot.

### GET /v1/manifold
Phase space manifold data for visualization.

### GET /v1/engines/{engine}/status
Individual engine status and metrics.

## WebSocket
Connect to wss://host/v1/stream for real-time fragility updates.
