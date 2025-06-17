# HAL9 API Documentation

## Base URL
- Local: `http://localhost:8080`
- Production: `https://api.hal9.ai` (TBD)

## Available Endpoints

### Health Check
- **GET** `/health`
- **Description**: Basic health check endpoint
- **Response**:
  ```json
  {
    "service": "hal9-server",
    "status": "healthy",
    "version": "0.1.0"
  }
  ```

### Server Status
- **GET** `/api/v1/status`
- **Description**: Detailed server status including neurons and metrics
- **Response**:
  ```json
  {
    "success": true,
    "data": {
      "running": true,
      "uptime_seconds": 1281,
      "neurons": [
        {
          "id": "neuron-l3-design",
          "layer": "L3",
          "state": "Running",
          "health": "healthy"
        }
      ],
      "metrics": {
        "signals_sent": 0,
        "signals_processed": 0,
        "signals_failed": 0,
        "average_latency_ms": 0.0
      },
      "network_status": null
    },
    "error": null
  }
  ```

### Neurons Management
- **GET** `/api/v1/neurons`
- **Description**: List all active neurons
- **Response**:
  ```json
  {
    "success": true,
    "data": [
      {
        "id": "neuron-l3-design",
        "layer": "L3",
        "state": "Running",
        "is_healthy": true
      }
    ],
    "error": null
  }
  ```

### Metrics
- **GET** `/api/v1/metrics`
- **Description**: System metrics and performance data
- **Response**:
  ```json
  {
    "success": true,
    "data": {
      "uptime_seconds": 1281,
      "signals_sent": 0,
      "signals_processed": 0,
      "signals_failed": 0,
      "signals_per_second": 0.0,
      "neurons_active": 3,
      "neurons_failed": 0,
      "neurons_processing": 0,
      "layer_latencies": {},
      "processing_times": {},
      "tokens_prompt": 0,
      "tokens_completion": 0,
      "tokens_total": 0,
      "cost_hourly": 0.0,
      "cost_daily": 0.0,
      "cost_total": 0.0,
      "errors_by_type": {},
      "memory_usage_mb": 0.0
    },
    "error": null
  }
  ```

### Code Generation
- **GET** `/api/v1/codegen/health`
- **Description**: Code generation service health
- **Response**:
  ```json
  {
    "status": "healthy",
    "message": "5/5 code generation neurons operational",
    "neurons": {
      "healthy": 5,
      "total": 5
    }
  }
  ```

- **GET** `/api/v1/codegen/templates`
- **Description**: Available project templates
- **Response**:
  ```json
  {
    "templates": {
      "api": {
        "name": "API Service",
        "description": "RESTful or GraphQL API",
        "frameworks": ["fastapi", "express", "gin", "axum"],
        "databases": ["postgresql", "mysql", "mongodb", "redis"]
      },
      "cli": {
        "name": "CLI Tool",
        "description": "Command-line application",
        "languages": ["rust", "go", "python", "node"]
      },
      "microservice": {
        "name": "Microservice",
        "description": "Cloud-native microservice",
        "languages": ["go", "rust", "python", "java"],
        "patterns": ["rest", "grpc", "graphql", "event-driven"]
      },
      "web-app": {
        "name": "Web Application",
        "description": "Full-stack web application",
        "frontends": ["react", "vue", "angular", "svelte"],
        "backends": ["express", "fastapi", "django", "axum"],
        "databases": ["postgresql", "mysql", "mongodb", "sqlite"]
      }
    }
  }
  ```

## Missing/TODO Endpoints

1. **Authentication** (404 - Not configured)
   - POST `/api/v1/auth/register`
   - POST `/api/v1/auth/login`
   - POST `/api/v1/auth/refresh`
   - GET `/api/v1/auth/profile`

2. **Signal Processing** (404 - Not implemented)
   - POST `/api/v1/signals`
   - GET `/api/v1/signals/:id`
   - GET `/api/v1/signals/:id/trace`

3. **Layers** (404 - Not implemented)
   - GET `/api/v1/layers`

## Error Responses

All endpoints return consistent error responses:
```json
{
  "success": false,
  "data": null,
  "error": "Error message here"
}
```

## Status Codes
- `200 OK`: Successful request
- `400 Bad Request`: Invalid request data
- `401 Unauthorized`: Authentication required
- `404 Not Found`: Endpoint or resource not found
- `500 Internal Server Error`: Server error

## Next Steps

1. Implement missing endpoints:
   - Signal submission and tracing
   - Layer management
   - Authentication (if needed)

2. Add request/response validation
3. Implement rate limiting
4. Add API versioning strategy
5. Set up OpenAPI/Swagger documentation