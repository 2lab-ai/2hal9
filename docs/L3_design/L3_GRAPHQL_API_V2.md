# HAL9 GraphQL API v2 Documentation

## Overview

HAL9's GraphQL API v2 provides a powerful, flexible interface for interacting with the distributed AI consciousness system. It offers:

- **Type-safe queries** with automatic validation
- **Real-time subscriptions** for live updates
- **Federation support** for distributed architectures
- **Advanced caching** and query optimization
- **Enterprise features** including persisted queries and schema registry

## Architecture

```
┌─────────────────────────────────────────────┐
│            GraphQL Client                    │
│  (Web, Mobile, CLI, or any GraphQL client)  │
└─────────────────┬───────────────────────────┘
                  │
┌─────────────────▼───────────────────────────┐
│           GraphQL Gateway                    │
│  • Schema validation                         │
│  • Query planning & optimization             │
│  • Real-time subscriptions                   │
│  • Apollo tracing & monitoring               │
└─────────────────┬───────────────────────────┘
                  │
┌─────────────────▼───────────────────────────┐
│          GraphQL Resolvers                   │
│  • Authentication & authorization            │
│  • Data fetching with DataLoader            │
│  • Caching & memoization                     │
│  • Event publishing                          │
└─────────────────┬───────────────────────────┘
                  │
┌─────────────────▼───────────────────────────┐
│            HAL9 Core Services                │
│  • Neuron management                         │
│  • Signal routing                            │
│  • Memory system                             │
│  • Learning engine                           │
└─────────────────────────────────────────────┘
```

## Getting Started

### Endpoint

```
POST https://api.hal9.ai/graphql
WS   wss://api.hal9.ai/graphql/ws  (for subscriptions)
```

### Authentication

All requests require a valid JWT token:

```http
Authorization: Bearer <your-jwt-token>
```

### GraphQL Playground

Interactive playground available at: `https://api.hal9.ai/graphql/playground`

## Schema Overview

### Core Types

```graphql
# Signal for communication between neurons
type SignalResponse {
  id: ID!
  signalId: UUID!
  content: String!
  layer: String!
  priority: Int!
  status: String!
  createdAt: DateTime!
  processedAt: DateTime
  result: JSON
}

# Neuron information
type NeuronInfo {
  id: ID!
  neuronId: UUID!
  name: String!
  neuronType: String!
  layer: String!
  state: String!
  config: JSON!
  metrics: NeuronMetrics!
  createdAt: DateTime!
  updatedAt: DateTime!
}

# System metrics
type SystemMetrics {
  totalNeurons: Int!
  activeNeurons: Int!
  signalsProcessed: Int!
  averageResponseTimeMs: Float!
  uptimeSeconds: Int!
  memoryUsageMb: Float!
  cpuUsagePercent: Float!
}
```

## Query Examples

### 1. Send a Signal

```graphql
mutation SendSignal($input: SignalInput!) {
  sendSignal(input: $input) {
    id
    content
    status
    result
    processedAt
  }
}

# Variables
{
  "input": {
    "content": "Analyze the current system performance",
    "layer": "L4",
    "priority": 5
  }
}
```

### 2. List Neurons with Pagination

```graphql
query ListNeurons($layer: String, $pagination: PaginationInput) {
  neurons(layer: $layer, pagination: $pagination) {
    edges {
      node {
        id
        name
        state
        layer
        metrics {
          processedCount
          errorCount
          successRate
          averageLatencyMs
        }
      }
      cursor
    }
    pageInfo {
      hasNextPage
      hasPreviousPage
      totalCount
    }
  }
}

# Variables
{
  "layer": "L3",
  "pagination": {
    "limit": 10,
    "offset": 0
  }
}
```

### 3. Search Memory

```graphql
query SearchMemory($query: String!, $limit: Int) {
  searchMemory(query: $query, limit: $limit) {
    id
    key
    content
    embeddingSimilarity
    accessCount
    lastAccessed
  }
}

# Variables
{
  "query": "distributed systems architecture",
  "limit": 5
}
```

### 4. Get System Metrics

```graphql
query SystemStatus {
  systemMetrics {
    totalNeurons
    activeNeurons
    signalsProcessed
    averageResponseTimeMs
    memoryUsageMb
    cpuUsagePercent
  }
  
  clusterHealth {
    status
    regions {
      name
      status
      nodeCount
      healthyNodes
      latencyMs
    }
  }
}
```

## Subscription Examples

### 1. Subscribe to Signal Updates

```graphql
subscription SignalProgress($signalId: ID) {
  signalUpdates(signalId: $signalId) {
    signalId
    status
    progress
    message
    timestamp
  }
}
```

### 2. Monitor Neuron State Changes

```graphql
subscription NeuronMonitoring($layer: String) {
  neuronStateChanges(layer: $layer) {
    neuronId
    previousState
    newState
    reason
    timestamp
  }
}
```

### 3. Real-time Metrics

```graphql
subscription MetricsStream {
  metricsUpdates(metricType: "performance") {
    metricType
    value
    labels
    timestamp
  }
}
```

### 4. Learning Events

```graphql
subscription LearningProgress {
  learningEvents {
    eventType
    patternId
    confidenceDelta
    description
    timestamp
  }
}
```

## Advanced Features

### 1. Federated Queries

```graphql
# Query across multiple HAL9 instances
query FederatedNeurons {
  _service {
    sdl
  }
  
  neurons {
    edges {
      node {
        id
        name
        # Will be resolved by the owning service
        @external
      }
    }
  }
}
```

### 2. Persisted Queries

```graphql
# Register a query
POST /graphql/persist
{
  "query": "query GetNeuron($id: ID!) { neuron(id: $id) { ... } }",
  "operationName": "GetNeuron"
}

# Use persisted query
POST /graphql
{
  "extensions": {
    "persistedQuery": {
      "version": 1,
      "sha256Hash": "<query-hash>"
    }
  },
  "variables": { "id": "123" }
}
```

### 3. Query Batching

```graphql
# Send multiple queries in one request
[
  {
    "query": "query { systemMetrics { ... } }"
  },
  {
    "query": "query { neurons { ... } }"
  }
]
```

## Error Handling

### Error Format

```json
{
  "errors": [
    {
      "message": "Neuron not found",
      "extensions": {
        "code": "NEURON_NOT_FOUND",
        "neuronId": "123e4567-e89b-12d3-a456-426614174000"
      },
      "path": ["neuron"],
      "locations": [{ "line": 2, "column": 3 }]
    }
  ],
  "data": null
}
```

### Error Codes

- `UNAUTHENTICATED`: Missing or invalid authentication
- `FORBIDDEN`: Insufficient permissions
- `NOT_FOUND`: Resource not found
- `VALIDATION_ERROR`: Input validation failed
- `INTERNAL_ERROR`: Server error
- `RATE_LIMITED`: Too many requests
- `COMPLEXITY_ERROR`: Query too complex

## Performance Optimization

### 1. Use Field Selection

```graphql
# Good - only request needed fields
query {
  neurons {
    edges {
      node {
        id
        name
        state
      }
    }
  }
}

# Avoid - requesting all fields
query {
  neurons {
    edges {
      node {
        ...AllNeuronFields
      }
    }
  }
}
```

### 2. Utilize DataLoader

The API automatically batches and caches requests:

```graphql
# These will be batched into a single database query
query {
  neuron1: neuron(id: "1") { name }
  neuron2: neuron(id: "2") { name }
  neuron3: neuron(id: "3") { name }
}
```

### 3. Use Pagination

```graphql
query LargeDataset($cursor: String) {
  signals(pagination: { limit: 20, cursor: $cursor }) {
    edges {
      node { ... }
      cursor
    }
    pageInfo {
      hasNextPage
      endCursor
    }
  }
}
```

## Security Best Practices

1. **Always use HTTPS** for GraphQL endpoints
2. **Implement query depth limiting** to prevent malicious queries
3. **Use query complexity analysis** to reject expensive queries
4. **Enable rate limiting** per user/organization
5. **Validate and sanitize** all input variables
6. **Use field-level permissions** for sensitive data
7. **Monitor query patterns** for anomalies

## Monitoring & Analytics

### Apollo Studio Integration

HAL9 supports Apollo Studio for:
- Query performance tracking
- Error monitoring
- Schema usage analytics
- Client awareness

### Metrics Exposed

- `graphql_query_duration`: Query execution time
- `graphql_query_complexity`: Calculated query complexity
- `graphql_field_usage`: Field-level usage statistics
- `graphql_error_rate`: Error rate by error type
- `graphql_subscription_count`: Active subscriptions

## Client Libraries

### JavaScript/TypeScript

```typescript
import { ApolloClient, InMemoryCache, gql } from '@apollo/client';

const client = new ApolloClient({
  uri: 'https://api.hal9.ai/graphql',
  cache: new InMemoryCache(),
  headers: {
    authorization: `Bearer ${token}`,
  },
});

// Query example
const result = await client.query({
  query: gql`
    query GetNeurons {
      neurons {
        edges {
          node {
            id
            name
          }
        }
      }
    }
  `,
});
```

### Python

```python
from gql import gql, Client
from gql.transport.aiohttp import AIOHTTPTransport

transport = AIOHTTPTransport(
    url="https://api.hal9.ai/graphql",
    headers={"Authorization": f"Bearer {token}"}
)

client = Client(transport=transport)

query = gql("""
    query GetNeurons {
        neurons {
            edges {
                node {
                    id
                    name
                }
            }
        }
    }
""")

result = await client.execute_async(query)
```

## Migration from REST API v1

### Mapping REST to GraphQL

| REST Endpoint | GraphQL Query/Mutation |
|--------------|----------------------|
| `GET /api/v1/neurons` | `query { neurons { ... } }` |
| `POST /api/v1/signals` | `mutation { sendSignal(input: {...}) { ... } }` |
| `GET /api/v1/status` | `query { systemMetrics { ... } }` |
| `GET /api/v1/signals/:id/trace` | `query { signal(id: ...) { ... } }` |

### Benefits of Migration

1. **Reduced overfetching** - Request only needed fields
2. **Single request** for multiple resources
3. **Real-time updates** via subscriptions
4. **Strong typing** with automatic validation
5. **Better performance** through intelligent caching

## Troubleshooting

### Common Issues

1. **Query Timeout**
   - Reduce query complexity
   - Use pagination for large datasets
   - Check network connectivity

2. **Authentication Errors**
   - Verify JWT token is valid
   - Check token expiration
   - Ensure proper permissions

3. **Subscription Connection Drops**
   - Implement reconnection logic
   - Check WebSocket compatibility
   - Monitor keep-alive messages

4. **High Memory Usage**
   - Enable query result caching
   - Limit subscription buffer size
   - Use field selection wisely

## Changelog

### v2.0.0 (Current)
- Initial GraphQL API release
- Full query, mutation, and subscription support
- Federation and enterprise features
- Real-time event system
- Advanced caching and optimization

### Planned Features
- GraphQL schema stitching
- Custom directives for AI operations
- Automatic persisted query management
- Enhanced monitoring integration
- WebAssembly custom resolvers