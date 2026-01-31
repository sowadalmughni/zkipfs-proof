import React, { useState } from 'react';
import { CheckCircle, XCircle, Upload, AlertCircle, Shield, Clock, Hash } from 'lucide-react';

const ProofVerification = () => {
  const [proofData, setProofData] = useState('');
  const [verificationResult, setVerificationResult] = useState(null);
  const [isVerifying, setIsVerifying] = useState(false);
  const [error, setError] = useState(null);

  const handleFileUpload = (event) => {
    const file = event.target.files[0];
    if (file) {
      const reader = new FileReader();
      reader.onload = (e) => {
        try {
          const content = e.target.result;
          JSON.parse(content); // Validate JSON
          setProofData(content);
          setError(null);
        } catch {
          setError('Invalid JSON file. Please upload a valid proof file.');
        }
      };
      reader.readAsText(file);
    }
  };

  const verifyProof = async () => {
    if (!proofData.trim()) {
      setError('Please provide proof data to verify.');
      return;
    }

    setIsVerifying(true);
    setError(null);
    setVerificationResult(null);

    try {
      const proof = JSON.parse(proofData);
      
      // Simulate verification process
      await new Promise(resolve => setTimeout(resolve, 2000));
      
      // Mock verification result
      const isValid = Math.random() > 0.2; // 80% chance of valid proof for demo
      
      const result = {
        isValid,
        proofId: proof.id || 'unknown',
        contentHash: proof.contentHash || 'unknown',
        rootHash: proof.rootHash || 'unknown',
        securityLevel: proof.metadata?.securityLevel || 128,
        verificationTime: Date.now(),
        details: {
          zkProofValid: isValid,
          hashesMatch: isValid,
          signatureValid: isValid,
          timestampValid: true,
          securityLevelMet: true
        },
        metadata: proof.metadata || {}
      };

      setVerificationResult(result);
    } catch {
      setError('Invalid proof format. Please check your proof data.');
    } finally {
      setIsVerifying(false);
    }
  };

  const formatTimestamp = (timestamp) => {
    return new Date(timestamp).toLocaleString();
  };

  return (
    <div className="space-y-6">
      {/* Input Section */}
      <div className="bg-gray-50 dark:bg-gray-800 rounded-lg p-6">
        <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">
          Proof Input
        </h3>
        
        <div className="space-y-4">
          {/* File Upload */}
          <div>
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              Upload Proof File
            </label>
            <div className="flex items-center space-x-4">
              <label className="inline-flex items-center px-4 py-2 border border-gray-300 dark:border-gray-600 text-sm font-medium rounded-md text-gray-700 dark:text-gray-300 bg-white dark:bg-gray-700 hover:bg-gray-50 dark:hover:bg-gray-600 cursor-pointer transition-colors">
                <Upload className="mr-2 h-4 w-4" />
                Choose File
                <input
                  type="file"
                  accept=".json"
                  onChange={handleFileUpload}
                  className="hidden"
                />
              </label>
              <span className="text-sm text-gray-500 dark:text-gray-400">
                or paste JSON below
              </span>
            </div>
          </div>

          {/* Text Input */}
          <div>
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              Proof JSON Data
            </label>
            <textarea
              value={proofData}
              onChange={(e) => setProofData(e.target.value)}
              placeholder="Paste your proof JSON data here..."
              rows={8}
              className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700 text-gray-900 dark:text-white font-mono text-sm"
            />
          </div>
        </div>
      </div>

      {/* Verify Button */}
      <div className="flex justify-center">
        <button
          onClick={verifyProof}
          disabled={!proofData.trim() || isVerifying}
          className="inline-flex items-center px-6 py-3 border border-transparent text-base font-medium rounded-md text-white bg-green-600 hover:bg-green-700 disabled:bg-gray-400 disabled:cursor-not-allowed transition-colors"
        >
          <Shield className="mr-2 h-5 w-5" />
          {isVerifying ? 'Verifying...' : 'Verify Proof'}
        </button>
      </div>

      {/* Error */}
      {error && (
        <div className="flex items-center space-x-2 p-4 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-md">
          <AlertCircle className="h-5 w-5 text-red-500" />
          <p className="text-sm text-red-600 dark:text-red-400">{error}</p>
        </div>
      )}

      {/* Verification Result */}
      {verificationResult && (
        <div className="space-y-4">
          {/* Status */}
          <div className={`flex items-center space-x-2 p-4 rounded-md ${
            verificationResult.isValid
              ? 'bg-green-50 dark:bg-green-900/20 border border-green-200 dark:border-green-800'
              : 'bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800'
          }`}>
            {verificationResult.isValid ? (
              <CheckCircle className="h-5 w-5 text-green-500" />
            ) : (
              <XCircle className="h-5 w-5 text-red-500" />
            )}
            <p className={`text-sm font-medium ${
              verificationResult.isValid
                ? 'text-green-600 dark:text-green-400'
                : 'text-red-600 dark:text-red-400'
            }`}>
              {verificationResult.isValid ? 'Proof is valid!' : 'Proof verification failed!'}
            </p>
          </div>

          {/* Details */}
          <div className="bg-gray-50 dark:bg-gray-800 rounded-lg p-6">
            <h4 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">
              Verification Details
            </h4>
            
            <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
              {/* Basic Info */}
              <div className="space-y-3">
                <h5 className="font-medium text-gray-900 dark:text-white">Basic Information</h5>
                <div className="space-y-2 text-sm">
                  <div className="flex items-center space-x-2">
                    <Hash className="h-4 w-4 text-gray-400" />
                    <span className="font-medium text-gray-700 dark:text-gray-300">Proof ID:</span>
                    <span className="font-mono text-gray-600 dark:text-gray-400 break-all">
                      {verificationResult.proofId}
                    </span>
                  </div>
                  <div className="flex items-center space-x-2">
                    <Shield className="h-4 w-4 text-gray-400" />
                    <span className="font-medium text-gray-700 dark:text-gray-300">Security Level:</span>
                    <span className="text-gray-600 dark:text-gray-400">
                      {verificationResult.securityLevel}-bit
                    </span>
                  </div>
                  <div className="flex items-center space-x-2">
                    <Clock className="h-4 w-4 text-gray-400" />
                    <span className="font-medium text-gray-700 dark:text-gray-300">Verified At:</span>
                    <span className="text-gray-600 dark:text-gray-400">
                      {formatTimestamp(verificationResult.verificationTime)}
                    </span>
                  </div>
                </div>
              </div>

              {/* Verification Checks */}
              <div className="space-y-3">
                <h5 className="font-medium text-gray-900 dark:text-white">Verification Checks</h5>
                <div className="space-y-2">
                  {Object.entries(verificationResult.details).map(([check, passed]) => (
                    <div key={check} className="flex items-center space-x-2">
                      {passed ? (
                        <CheckCircle className="h-4 w-4 text-green-500" />
                      ) : (
                        <XCircle className="h-4 w-4 text-red-500" />
                      )}
                      <span className="text-sm text-gray-700 dark:text-gray-300 capitalize">
                        {check.replace(/([A-Z])/g, ' $1').trim()}
                      </span>
                    </div>
                  ))}
                </div>
              </div>
            </div>

            {/* Hashes */}
            <div className="mt-6 space-y-3">
              <h5 className="font-medium text-gray-900 dark:text-white">Cryptographic Hashes</h5>
              <div className="space-y-2 text-sm">
                <div>
                  <span className="font-medium text-gray-700 dark:text-gray-300">Content Hash:</span>
                  <div className="mt-1 p-2 bg-gray-100 dark:bg-gray-700 rounded font-mono text-xs break-all">
                    {verificationResult.contentHash}
                  </div>
                </div>
                <div>
                  <span className="font-medium text-gray-700 dark:text-gray-300">Root Hash:</span>
                  <div className="mt-1 p-2 bg-gray-100 dark:bg-gray-700 rounded font-mono text-xs break-all">
                    {verificationResult.rootHash}
                  </div>
                </div>
              </div>
            </div>

            {/* Metadata */}
            {verificationResult.metadata && Object.keys(verificationResult.metadata).length > 0 && (
              <div className="mt-6 space-y-3">
                <h5 className="font-medium text-gray-900 dark:text-white">Proof Metadata</h5>
                <div className="space-y-2 text-sm">
                  {verificationResult.metadata.fileName && (
                    <div>
                      <span className="font-medium text-gray-700 dark:text-gray-300">File Name:</span>
                      <span className="ml-2 text-gray-600 dark:text-gray-400">
                        {verificationResult.metadata.fileName}
                      </span>
                    </div>
                  )}
                  {verificationResult.metadata.fileSize && (
                    <div>
                      <span className="font-medium text-gray-700 dark:text-gray-300">File Size:</span>
                      <span className="ml-2 text-gray-600 dark:text-gray-400">
                        {(verificationResult.metadata.fileSize / (1024 * 1024)).toFixed(2)} MB
                      </span>
                    </div>
                  )}
                  {verificationResult.metadata.contentSelection && (
                    <div>
                      <span className="font-medium text-gray-700 dark:text-gray-300">Content Selection:</span>
                      <span className="ml-2 text-gray-600 dark:text-gray-400 capitalize">
                        {verificationResult.metadata.contentSelection}
                      </span>
                    </div>
                  )}
                </div>
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  );
};

export default ProofVerification;

