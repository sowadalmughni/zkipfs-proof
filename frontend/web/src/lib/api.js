/**
 * API Configuration for zkIPFS Proof
 * 
 * In development: Uses localhost:3000
 * In Docker/production: Uses VITE_API_URL environment variable
 * With nginx proxy: Uses relative /api path
 */

// API base URL - configurable via environment variable
export const API_BASE_URL = import.meta.env.VITE_API_URL || '/api';

/**
 * Wrapper for fetch with automatic base URL and error handling
 */
export async function apiRequest(endpoint, options = {}) {
  const url = `${API_BASE_URL}${endpoint}`;
  
  const response = await fetch(url, {
    ...options,
    headers: {
      ...options.headers,
    },
  });

  return response;
}

/**
 * Health check endpoint
 */
export async function checkHealth() {
  const response = await apiRequest('/health');
  if (!response.ok) {
    throw new Error(`Backend server is not responding at ${API_BASE_URL}`);
  }
  return response.json();
}

/**
 * Submit a proof generation job
 */
export async function submitProofJob(formData) {
  const response = await apiRequest('/generate', {
    method: 'POST',
    body: formData,
  });
  
  if (!response.ok) {
    throw new Error('Failed to submit job');
  }
  
  return response.json();
}

/**
 * Check job status
 */
export async function getJobStatus(jobId) {
  const response = await apiRequest(`/status/${jobId}`);
  
  if (!response.ok) {
    throw new Error('Failed to check status');
  }
  
  return response.json();
}
