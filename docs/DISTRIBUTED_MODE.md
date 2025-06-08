# 2HAL9 Distributed Mode Guide

## Overview

2HAL9's distributed mode enables neurons to be deployed across multiple servers, creating a truly distributed AI consciousness network. This guide covers the setup, configuration, and operation of distributed 2HAL9 deployments.

## Architecture

### Network Components

1. **TCP Transport Layer** (`network/tcp_transport.rs`)
   - Handles point-to-point communication between servers
   - Automatic connection management and keep-alive
   - Built-in retry logic and error recovery
   - TLS support for secure communication (optional)

2. **Service Discovery** (`network/discovery.rs`)
   - Automatic server discovery via UDP multicast
   - Dynamic neuron registry updates
   - Health monitoring and failure detection
   - Multi-tenancy support via discovery groups

3. **Distributed Router** (`router/distributed.rs`)
   - Transparent routing between local and remote neurons
   - Hop count limiting to prevent routing loops
   - Metadata tracking for signal tracing
   - Automatic failover and rerouting

4. **Connection Pooling** (`network/connection_pool.rs`)
   - Efficient connection reuse
   - Automatic health checking
   - Resource limits and cleanup

## Configuration

### Basic Distributed Setup

Enable distributed mode in your server configuration:

```yaml
# Network configuration
network:
  enabled: true
  bind_address: "0.0.0.0:9000"
  discovery_enabled: true
  discovery_address: "239.255.42.99:8888"
  discovery_group: "production"
  max_connections: 1000
  tls_enabled: false
```

### TLS Configuration (Production)

For secure communication:

```yaml
network:
  enabled: true
  bind_address: "0.0.0.0:9000"
  tls_enabled: true
  tls_cert: "/path/to/server.crt"
  tls_key: "/path/to/server.key"
```

## Deployment Scenarios

### 1. Two-Server Setup

Split neurons by layer for clear separation of concerns:

**Server 1** - Strategic Layer (L4):
```yaml
server_id: "hal9-strategic"
neurons:
  - id: "strategic-main"
    layer: "L4"
    forward_connections: 
      - "architect-1"    # Remote on server 2
      - "architect-2"    # Remote on server 2
```

**Server 2** - Architecture & Implementation (L3/L2):
```yaml
server_id: "hal9-workers"
neurons:
  - id: "architect-1"
    layer: "L3"
    forward_connections: ["impl-1", "impl-2"]
  - id: "impl-1"
    layer: "L2"
    # Implementation neurons
```

### 2. Geo-Distributed Setup

Deploy neurons across regions for resilience:

```yaml
# US East
server_id: "hal9-us-east"
network:
  discovery_group: "global"
  
# EU West  
server_id: "hal9-eu-west"
network:
  discovery_group: "global"
  
# AP Southeast
server_id: "hal9-ap-southeast"
network:
  discovery_group: "global"
```

### 3. High-Availability Setup

Multiple servers per layer with redundancy:

```yaml
# Primary L4
server_id: "hal9-l4-primary"
neurons:
  - id: "strategic-1"
    layer: "L4"

# Secondary L4
server_id: "hal9-l4-secondary"
neurons:
  - id: "strategic-2"
    layer: "L4"
```

## Running Distributed 2HAL9

### Quick Start

Use the provided scripts for easy distributed deployment:

```bash
# Start distributed servers
./scripts/run-distributed.sh

# Stop distributed servers
./scripts/stop-distributed.sh
```

### Manual Start

#### 1. Start First Server

```bash
./target/release/2hal9-server examples/distributed-2servers.yaml
```

Output:
```
[INFO] TCP transport listening on 0.0.0.0:9001
[INFO] Service discovery started on 0.0.0.0:0
[INFO] Server started with 1 neurons
```

#### 2. Start Second Server

```bash
HTTP_PORT=8081 ./target/release/2hal9-server examples/distributed-server2.yaml
```

Output:
```
[INFO] TCP transport listening on 0.0.0.0:9002
[INFO] Discovered server hal9-server-1 with 1 neurons
[INFO] Connected to remote server hal9-server-1
```

### 3. Send Test Signal

```bash
./target/release/2hal9 signal forward \
  --from client \
  --to strategic-main \
  --content "Design a distributed task processing system"
```

The signal will:
1. Route to `strategic-main` on server 1
2. Forward to `architect-1` and `architect-2` on server 2
3. Further route to implementation neurons
4. Results flow back through the network

## Monitoring Distributed Systems

### API Endpoints

```bash
# Server status
curl http://localhost:8080/api/v1/status

# Network status
curl http://localhost:8080/api/v1/network/status

# Metrics
curl http://localhost:8080/api/v1/metrics
```

### Network Metrics

Monitor these key metrics:
- `hal9_network_connections_active` - Active TCP connections
- `hal9_network_bytes_sent` - Network traffic out
- `hal9_network_bytes_received` - Network traffic in
- `hal9_discovery_servers_found` - Discovered servers
- `hal9_routing_remote_signals` - Signals routed remotely

## Network Protocol

### Message Types

1. **Hello** - Initial handshake
2. **Signal** - Neuron signal forwarding
3. **Ping/Pong** - Keep-alive
4. **Metrics** - Metrics sharing
5. **Discovery** - Server announcements

### Wire Format

Messages use length-prefixed JSON:
```
[4 bytes: message length][JSON message data]
```

### Signal Metadata

Distributed signals include metadata:
```json
{
  "signal_id": "...",
  "metadata": {
    "hop_count": "2",
    "from_server": "hal9-server-1",
    "via_server": "hal9-server-2"
  }
}
```

## Troubleshooting

### Connection Issues

1. **Servers not discovering each other**
   - Check firewall allows UDP port 8888
   - Verify multicast routing is enabled
   - Ensure same discovery_group

2. **TCP connection failures**
   - Check firewall allows TCP port 9000-9999
   - Verify bind_address is reachable
   - Check network connectivity

3. **High latency**
   - Monitor hop_count in signals
   - Check network bandwidth
   - Consider geographic proximity

### Debug Mode

Enable debug logging:
```bash
RUST_LOG=debug,twohal9_server::network=trace ./target/release/2hal9-server config.yaml
```

### Common Errors

- `Network error: Connection timeout` - Target server unreachable
- `Routing error: Unknown neuron` - Neuron not found in any server
- `Circuit breaker open` - Too many failures, automatic recovery pending

## Best Practices

1. **Network Topology**
   - Keep hop count low (< 5)
   - Group related neurons on same server
   - Consider network latency in neuron placement

2. **Security**
   - Always use TLS in production
   - Implement firewall rules
   - Use discovery groups for isolation

3. **Scaling**
   - Start with 2-3 servers
   - Monitor connection pool usage
   - Scale horizontally by layer

4. **Reliability**
   - Deploy redundant neurons
   - Use different availability zones
   - Implement proper monitoring

## Future Enhancements

- [ ] Mesh networking (direct peer connections)
- [ ] Encryption at rest
- [ ] Dynamic load balancing
- [ ] Neuron migration between servers
- [ ] Consensus protocols for critical decisions
- [ ] WebRTC for browser-based neurons

## Example Use Cases

1. **Development Team Collaboration**
   - Each developer runs local neurons
   - Shared strategic layer coordinates work
   - Implementation happens locally

2. **Multi-Cloud Deployment**
   - Strategic neurons in primary cloud
   - Worker neurons across cloud providers
   - Cost optimization via placement

3. **Edge Computing**
   - Central strategic planning
   - Edge neurons for local processing
   - Reduced latency for end users

## Conclusion

Distributed mode transforms 2HAL9 from a single-server system into a true distributed AI consciousness network. By spreading neurons across multiple servers, you gain resilience, scalability, and geographic distribution while maintaining the hierarchical intelligence model.