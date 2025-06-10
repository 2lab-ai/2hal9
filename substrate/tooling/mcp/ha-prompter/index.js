/**
 * JavaScript wrapper for ha-prompter binary
 * Provides a Node.js API for the HA Prompter MCP tool
 */

const { spawn } = require('child_process');
const path = require('path');
const EventEmitter = require('events');

class HAPrompterClient extends EventEmitter {
  constructor(options = {}) {
    super();
    
    this.binaryPath = options.binaryPath || path.join(__dirname, 'bin', 'ha-prompter');
    this.process = null;
    this.requestId = 0;
    this.pendingRequests = new Map();
  }

  async start() {
    return new Promise((resolve, reject) => {
      this.process = spawn(this.binaryPath, [], {
        stdio: ['pipe', 'pipe', 'pipe']
      });

      this.process.on('error', (err) => {
        reject(new Error(`Failed to start ha-prompter: ${err.message}`));
      });

      this.process.stdout.once('data', (data) => {
        try {
          const capabilities = JSON.parse(data.toString().trim());
          this.capabilities = capabilities;
          this.setupListeners();
          resolve(capabilities);
        } catch (err) {
          reject(new Error(`Failed to parse capabilities: ${err.message}`));
        }
      });
    });
  }

  setupListeners() {
    let buffer = '';
    
    this.process.stdout.on('data', (data) => {
      buffer += data.toString();
      
      // Process complete JSON messages
      const lines = buffer.split('\n');
      buffer = lines.pop(); // Keep incomplete line in buffer
      
      for (const line of lines) {
        if (line.trim()) {
          try {
            const response = JSON.parse(line);
            const requestId = response.requestId;
            
            if (requestId && this.pendingRequests.has(requestId)) {
              const { resolve } = this.pendingRequests.get(requestId);
              this.pendingRequests.delete(requestId);
              resolve(response.result);
            }
          } catch (err) {
            this.emit('error', err);
          }
        }
      }
    });

    this.process.stderr.on('data', (data) => {
      this.emit('error', new Error(data.toString()));
    });

    this.process.on('close', (code) => {
      this.emit('close', code);
      this.cleanup();
    });
  }

  async sendRequest(tool, parameters) {
    if (!this.process) {
      throw new Error('Client not started. Call start() first.');
    }

    const requestId = ++this.requestId;
    const request = {
      requestId,
      tool,
      parameters
    };

    return new Promise((resolve, reject) => {
      this.pendingRequests.set(requestId, { resolve, reject });
      
      this.process.stdin.write(JSON.stringify(request) + '\n', (err) => {
        if (err) {
          this.pendingRequests.delete(requestId);
          reject(err);
        }
      });

      // Timeout after 30 seconds
      setTimeout(() => {
        if (this.pendingRequests.has(requestId)) {
          this.pendingRequests.delete(requestId);
          reject(new Error('Request timeout'));
        }
      }, 30000);
    });
  }

  // Convenience methods
  async compress(content, dataType, targetLevel, currentLevel = null) {
    const parameters = {
      content,
      data_type: dataType,
      target_level: targetLevel
    };
    
    if (currentLevel !== null) {
      parameters.current_level = currentLevel;
    }
    
    return this.sendRequest('compress', parameters);
  }

  async expand(content, dataType, fromLevel, toLevel) {
    return this.sendRequest('expand', {
      content,
      data_type: dataType,
      from_level: fromLevel,
      to_level: toLevel
    });
  }

  async cascadeDown(content, dataType) {
    return this.sendRequest('cascade_down', {
      content,
      data_type: dataType
    });
  }

  async cascadeUp(content, dataType) {
    return this.sendRequest('cascade_up', {
      content,
      data_type: dataType
    });
  }

  async analyze(content, dataType) {
    return this.sendRequest('analyze', {
      content,
      data_type: dataType
    });
  }

  close() {
    if (this.process) {
      this.process.kill();
      this.cleanup();
    }
  }

  cleanup() {
    this.process = null;
    for (const { reject } of this.pendingRequests.values()) {
      reject(new Error('Client closed'));
    }
    this.pendingRequests.clear();
  }
}

// Export the client
module.exports = { HAPrompterClient };

// CLI usage
if (require.main === module) {
  const client = new HAPrompterClient();
  
  client.start()
    .then(() => {
      console.log('HA Prompter started. Send JSON requests via stdin.');
      
      process.stdin.on('data', async (data) => {
        try {
          const request = JSON.parse(data.toString().trim());
          const result = await client.sendRequest(request.tool, request.parameters);
          console.log(JSON.stringify(result));
        } catch (err) {
          console.error('Error:', err.message);
        }
      });
    })
    .catch((err) => {
      console.error('Failed to start:', err.message);
      process.exit(1);
    });
}