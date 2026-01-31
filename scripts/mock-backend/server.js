/**
 * zkIPFS Proof - Mock Backend Server
 * 
 * This is a lightweight mock backend for development/demo purposes.
 * It responds to all API endpoints with realistic sample data.
 */

const http = require('http');
const { randomUUID } = require('crypto');

// In-memory job storage
const jobs = new Map();

// Sample proof data
const sampleProof = {
  proof_id: randomUUID(),
  proof_type: 'RISC0',
  proof_data: 'ZK_PROOF_MOCK_DATA_' + Date.now(),
  public_inputs: {
    file_hash: 'QmXoypizjW3WknFiJnKLwHCnL72vedxjQkDDP1mXWo6uco',
    content_hash: 'bafy2bzacegtea7h27w7k3ym6jfkbz6v54i6vxb35ynp4qpf6rbhqgpzrqtq',
    timestamp: Date.now()
  },
  verification_key: 'VK_' + randomUUID().slice(0, 8),
  created_at: new Date().toISOString(),
  verified: true
};

// Parse JSON body
async function parseBody(req) {
  return new Promise((resolve, reject) => {
    let body = '';
    req.on('data', chunk => body += chunk);
    req.on('end', () => {
      try {
        resolve(body ? JSON.parse(body) : {});
      } catch {
        resolve({});
      }
    });
    req.on('error', reject);
  });
}

// Route handlers
const routes = {
  'GET /health': () => ({
    status: 'healthy',
    service: 'zkipfs-proof-mock',
    version: '1.0.0',
    uptime: process.uptime(),
    timestamp: new Date().toISOString()
  }),

  'GET /api/health': () => ({
    status: 'healthy',
    service: 'zkipfs-proof-mock',
    version: '1.0.0'
  }),

  'POST /generate': async (req) => {
    const jobId = randomUUID();
    const job = {
      job_id: jobId,
      status: 'Pending',
      created_at: new Date().toISOString(),
      progress: 0
    };
    jobs.set(jobId, job);

    // Simulate async processing
    setTimeout(() => {
      const j = jobs.get(jobId);
      if (j) {
        j.status = 'Processing';
        j.progress = 50;
      }
    }, 2000);

    setTimeout(() => {
      const j = jobs.get(jobId);
      if (j) {
        j.status = 'Completed';
        j.progress = 100;
        j.result = { ...sampleProof, proof_id: jobId };
      }
    }, 5000);

    return { job_id: jobId };
  },

  'GET /status/:id': (req, params) => {
    const job = jobs.get(params.id);
    if (!job) {
      return { error: 'Job not found', _httpStatus: 404 };
    }
    return {
      status: job.status,
      progress: job.progress,
      result: job.result || null
    };
  },

  'POST /verify': async () => ({
    verified: true,
    verification_time_ms: Math.floor(Math.random() * 100) + 50,
    proof_type: 'RISC0',
    timestamp: new Date().toISOString()
  }),

  'GET /proofs': () => ({
    proofs: [sampleProof],
    total: 1,
    page: 1,
    per_page: 10
  }),

  'GET /proofs/:id': (req, params) => ({
    ...sampleProof,
    proof_id: params.id
  })
};

// Match route with path parameters
function matchRoute(method, path) {
  const key = `${method} ${path}`;
  if (routes[key]) return { handler: routes[key], params: {} };

  // Try pattern matching for :id routes
  for (const [pattern, handler] of Object.entries(routes)) {
    const [routeMethod, routePath] = pattern.split(' ');
    if (routeMethod !== method) continue;

    const patternParts = routePath.split('/');
    const pathParts = path.split('/');
    if (patternParts.length !== pathParts.length) continue;

    const params = {};
    let match = true;
    for (let i = 0; i < patternParts.length; i++) {
      if (patternParts[i].startsWith(':')) {
        params[patternParts[i].slice(1)] = pathParts[i];
      } else if (patternParts[i] !== pathParts[i]) {
        match = false;
        break;
      }
    }
    if (match) return { handler, params };
  }
  return null;
}

// Create server
const server = http.createServer(async (req, res) => {
  const url = new URL(req.url, `http://${req.headers.host}`);
  const path = url.pathname;
  const method = req.method;

  // CORS headers
  res.setHeader('Access-Control-Allow-Origin', '*');
  res.setHeader('Access-Control-Allow-Methods', 'GET, POST, OPTIONS');
  res.setHeader('Access-Control-Allow-Headers', 'Content-Type');

  if (method === 'OPTIONS') {
    res.writeHead(204);
    res.end();
    return;
  }

  const match = matchRoute(method, path);
  
  if (!match) {
    res.writeHead(404, { 'Content-Type': 'application/json' });
    res.end(JSON.stringify({ error: 'Not found', path }));
    return;
  }

  try {
    const result = await match.handler(req, match.params);
    const statusCode = result._httpStatus || 200;
    delete result._httpStatus;
    
    res.writeHead(statusCode, { 'Content-Type': 'application/json' });
    res.end(JSON.stringify(result));
  } catch (error) {
    res.writeHead(500, { 'Content-Type': 'application/json' });
    res.end(JSON.stringify({ error: error.message }));
  }
});

const PORT = process.env.PORT || 3000;
server.listen(PORT, '0.0.0.0', () => {
  console.log(`🚀 zkIPFS Mock Backend running on http://0.0.0.0:${PORT}`);
  console.log('📝 Endpoints:');
  console.log('   GET  /health     - Health check');
  console.log('   POST /generate   - Submit proof generation job');
  console.log('   GET  /status/:id - Check job status');
  console.log('   POST /verify     - Verify a proof');
  console.log('   GET  /proofs     - List proofs');
});
