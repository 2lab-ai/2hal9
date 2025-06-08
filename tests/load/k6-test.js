import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate } from 'k6/metrics';

// Custom metrics
const errorRate = new Rate('errors');

// Test configuration
export let options = {
    // Test stages
    stages: [
        { duration: '30s', target: 10 },    // Warm up to 10 users
        { duration: '1m', target: 100 },    // Ramp to 100 users
        { duration: '2m', target: 1000 },   // Ramp to 1000 users
        { duration: '5m', target: 1000 },   // Stay at 1000 users
        { duration: '1m', target: 100 },    // Ramp down to 100
        { duration: '30s', target: 0 },     // Ramp down to 0
    ],
    
    // Thresholds
    thresholds: {
        http_req_duration: ['p(95)<500', 'p(99)<1000'], // 95% under 500ms, 99% under 1s
        http_req_failed: ['rate<0.01'],                  // Error rate under 1%
        errors: ['rate<0.05'],                           // Custom error rate under 5%
    },
    
    // Tags
    tags: {
        test_type: 'performance',
        test_name: '1000_user_load_test',
    },
};

const BASE_URL = __ENV.BASE_URL || 'http://localhost:8080';

// Test data generators
function generateSignal() {
    const layers = ['L4', 'L3', 'L2'];
    const neurons = ['strategic-planner', 'task-analyzer', 'executor'];
    
    return {
        content: `Test signal ${Math.random()}`,
        layer: layers[Math.floor(Math.random() * layers.length)],
        neuron_id: neurons[Math.floor(Math.random() * neurons.length)],
    };
}

function generateApiKey() {
    return `test_key_${Math.random().toString(36).substring(7)}`;
}

// API endpoints
const endpoints = {
    health: `${BASE_URL}/health`,
    status: `${BASE_URL}/api/v1/status`,
    signal: `${BASE_URL}/api/v1/signal`,
    neurons: `${BASE_URL}/api/v1/neurons`,
    metrics: `${BASE_URL}/api/v1/metrics`,
    auth: {
        register: `${BASE_URL}/api/v1/auth/register`,
        login: `${BASE_URL}/api/v1/auth/login`,
        profile: `${BASE_URL}/api/v1/auth/profile`,
    },
};

// Headers
const headers = {
    'Content-Type': 'application/json',
};

// Test scenarios
export default function() {
    // Scenario weights
    const scenario = Math.random();
    
    if (scenario < 0.4) {
        // 40% - Signal submission (most common)
        testSignalSubmission();
    } else if (scenario < 0.6) {
        // 20% - Status checks
        testStatusCheck();
    } else if (scenario < 0.8) {
        // 20% - Neuron queries
        testNeuronQueries();
    } else if (scenario < 0.95) {
        // 15% - Authentication flows
        testAuthentication();
    } else {
        // 5% - Metrics queries
        testMetrics();
    }
    
    sleep(1);
}

// Test functions
function testSignalSubmission() {
    const signal = generateSignal();
    
    const res = http.post(endpoints.signal, JSON.stringify(signal), { headers });
    
    const checkRes = check(res, {
        'signal submission status is 200': (r) => r.status === 200,
        'signal submission has signal_id': (r) => {
            try {
                const body = JSON.parse(r.body);
                return body.data && body.data.signal_id;
            } catch (e) {
                return false;
            }
        },
        'signal submission time < 200ms': (r) => r.timings.duration < 200,
    });
    
    errorRate.add(!checkRes);
}

function testStatusCheck() {
    const res = http.get(endpoints.status, { headers });
    
    const checkRes = check(res, {
        'status check is 200': (r) => r.status === 200,
        'status has running field': (r) => {
            try {
                const body = JSON.parse(r.body);
                return body.data && typeof body.data.running === 'boolean';
            } catch (e) {
                return false;
            }
        },
        'status check time < 100ms': (r) => r.timings.duration < 100,
    });
    
    errorRate.add(!checkRes);
}

function testNeuronQueries() {
    const res = http.get(endpoints.neurons, { headers });
    
    const checkRes = check(res, {
        'neuron list is 200': (r) => r.status === 200,
        'neuron list is array': (r) => {
            try {
                const body = JSON.parse(r.body);
                return body.data && Array.isArray(body.data);
            } catch (e) {
                return false;
            }
        },
        'neuron query time < 150ms': (r) => r.timings.duration < 150,
    });
    
    errorRate.add(!checkRes);
    
    // If successful, query individual neuron
    if (res.status === 200) {
        try {
            const body = JSON.parse(res.body);
            if (body.data && body.data.length > 0) {
                const neuronId = body.data[0].id;
                const neuronRes = http.get(`${endpoints.neurons}/${neuronId}`, { headers });
                
                check(neuronRes, {
                    'individual neuron query is 200': (r) => r.status === 200,
                });
            }
        } catch (e) {
            // Ignore parse errors
        }
    }
}

function testAuthentication() {
    // Register new user
    const username = `user_${Date.now()}_${Math.random().toString(36).substring(7)}`;
    const password = 'Test123!@#';
    
    const registerRes = http.post(
        endpoints.auth.register,
        JSON.stringify({ username, password }),
        { headers }
    );
    
    if (registerRes.status === 200) {
        // Login
        const loginRes = http.post(
            endpoints.auth.login,
            JSON.stringify({ username, password }),
            { headers }
        );
        
        const checkRes = check(loginRes, {
            'login is 200': (r) => r.status === 200,
            'login returns token': (r) => {
                try {
                    const body = JSON.parse(r.body);
                    return body.data && body.data.access_token;
                } catch (e) {
                    return false;
                }
            },
        });
        
        errorRate.add(!checkRes);
        
        // Get profile with token
        if (loginRes.status === 200) {
            try {
                const body = JSON.parse(loginRes.body);
                const token = body.data.access_token;
                
                const profileRes = http.get(endpoints.auth.profile, {
                    headers: {
                        ...headers,
                        'Authorization': `Bearer ${token}`,
                    },
                });
                
                check(profileRes, {
                    'profile query is 200': (r) => r.status === 200,
                });
            } catch (e) {
                // Ignore errors
            }
        }
    }
}

function testMetrics() {
    const res = http.get(endpoints.metrics, { headers });
    
    const checkRes = check(res, {
        'metrics query is 200': (r) => r.status === 200,
        'metrics has data': (r) => {
            try {
                const body = JSON.parse(r.body);
                return body.data && body.data.signals_sent >= 0;
            } catch (e) {
                return false;
            }
        },
        'metrics query time < 100ms': (r) => r.timings.duration < 100,
    });
    
    errorRate.add(!checkRes);
}

// Handle test lifecycle
export function setup() {
    console.log('Starting HAL9 load test...');
    
    // Check if server is available
    const healthRes = http.get(endpoints.health);
    if (healthRes.status !== 200) {
        throw new Error('HAL9 server is not available');
    }
    
    return { startTime: new Date() };
}

export function teardown(data) {
    console.log(`Test completed. Duration: ${new Date() - data.startTime}ms`);
}