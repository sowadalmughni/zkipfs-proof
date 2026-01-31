import React, { useState } from 'react';
import { Play, Download, Copy, CheckCircle, AlertCircle } from 'lucide-react';
import { Progress } from './ui/progress';

const ProofGeneration = ({ file, onProofGenerated }) => {
  const [isGenerating, setIsGenerating] = useState(false);
  const [progress, setProgress] = useState(0);
  const [proof, setProof] = useState(null);
  const [error, setError] = useState(null);
  const [settings, setSettings] = useState({
    securityLevel: 128,
    contentSelection: 'pattern',
    pattern: '',
    regex: '',
    xpath: '',
    byteStart: 0,
    byteEnd: 1000,
    compressionEnabled: true
  });

  const generateProof = async () => {
    if (!file) return;

    setIsGenerating(true);
    setProgress(0);
    setError(null);

    try {
      // Simulate proof generation with realistic progress
      const steps = [
        { message: 'Processing file...', duration: 1000 },
        { message: 'Extracting IPFS blocks...', duration: 1500 },
        { message: 'Generating ZK circuit...', duration: 2000 },
        { message: 'Computing proof...', duration: 3000 },
        { message: 'Finalizing...', duration: 500 }
      ];

      let currentProgress = 0;
      for (let i = 0; i < steps.length; i++) {
        const step = steps[i];
        const stepProgress = (100 / steps.length);
        
        await new Promise(resolve => setTimeout(resolve, step.duration));
        currentProgress += stepProgress;
        setProgress(currentProgress);
      }

      // Generate mock proof data
      const mockProof = {
        id: `proof_${Date.now()}`,
        contentHash: Array.from({ length: 32 }, () => 
          Math.floor(Math.random() * 256).toString(16).padStart(2, '0')
        ).join(''),
        rootHash: Array.from({ length: 32 }, () => 
          Math.floor(Math.random() * 256).toString(16).padStart(2, '0')
        ).join(''),
        zkProof: {
          receipt: 'mock_receipt_data',
          publicInputs: 'mock_public_inputs',
          formatVersion: '1.0'
        },
        metadata: {
          fileName: file.name,
          fileSize: file.size,
          generationTime: Date.now(),
          securityLevel: settings.securityLevel,
          contentSelection: settings.contentSelection,
          selectionValue: settings.contentSelection === 'pattern' ? settings.pattern :
                         settings.contentSelection === 'regex' ? settings.regex :
                         settings.contentSelection === 'xpath' ? settings.xpath :
                         settings.contentSelection === 'byteRange' ? `${settings.byteStart}-${settings.byteEnd}` : 'multiple'
        },
        createdAt: new Date().toISOString()
      };

      setProof(mockProof);
      onProofGenerated?.(mockProof);
    } catch (err) {
      setError(err.message || 'Failed to generate proof');
    } finally {
      setIsGenerating(false);
    }
  };

  const downloadProof = () => {
    if (!proof) return;
    
    const dataStr = JSON.stringify(proof, null, 2);
    const dataBlob = new Blob([dataStr], { type: 'application/json' });
    const url = URL.createObjectURL(dataBlob);
    const link = document.createElement('a');
    link.href = url;
    link.download = `zkipfs_proof_${proof.id}.json`;
    link.click();
    URL.revokeObjectURL(url);
  };

  const copyProof = async () => {
    if (!proof) return;
    
    try {
      await navigator.clipboard.writeText(JSON.stringify(proof, null, 2));
      // You could add a toast notification here
    } catch (err) {
      console.error('Failed to copy proof:', err);
    }
  };

  return (
    <div className="space-y-6">
      {/* Settings */}
      <div className="bg-gray-50 dark:bg-gray-800 rounded-lg p-6">
        <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">
          Proof Settings
        </h3>
        
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          <div>
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              Security Level
            </label>
            <select
              value={settings.securityLevel}
              onChange={(e) => setSettings(prev => ({ ...prev, securityLevel: parseInt(e.target.value) }))}
              className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
            >
              <option value={128}>128-bit</option>
              <option value={192}>192-bit</option>
              <option value={256}>256-bit</option>
            </select>
          </div>
          
          <div>
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              Content Selection
            </label>
            <select
              value={settings.contentSelection}
              onChange={(e) => setSettings(prev => ({ ...prev, contentSelection: e.target.value }))}
              className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
            >
              <option value="pattern">Pattern Match</option>
              <option value="regex">Regex Match</option>
              <option value="xpath">XPath Selector</option>
              <option value="byteRange">Byte Range</option>
              <option value="multiple">Multiple Selections</option>
            </select>
          </div>
        </div>

        {settings.contentSelection === 'pattern' && (
          <div className="mt-4">
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              Pattern to Prove
            </label>
            <input
              type="text"
              value={settings.pattern}
              onChange={(e) => setSettings(prev => ({ ...prev, pattern: e.target.value }))}
              placeholder="Enter text pattern to prove exists in file"
              className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
            />
          </div>
        )}

        {settings.contentSelection === 'regex' && (
          <div className="mt-4">
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              Regex Pattern
            </label>
            <input
              type="text"
              value={settings.regex}
              onChange={(e) => setSettings(prev => ({ ...prev, regex: e.target.value }))}
              placeholder="Enter regex pattern (e.g., ^\d{3}-\d{2}-\d{4}$)"
              className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700 text-gray-900 dark:text-white font-mono"
            />
          </div>
        )}

        {settings.contentSelection === 'xpath' && (
          <div className="mt-4">
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              XPath Selector
            </label>
            <input
              type="text"
              value={settings.xpath}
              onChange={(e) => setSettings(prev => ({ ...prev, xpath: e.target.value }))}
              placeholder="Enter XPath (e.g., //div[@id='content'])"
              className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700 text-gray-900 dark:text-white font-mono"
            />
            <p className="mt-1 text-xs text-gray-500 dark:text-gray-400">
              Works best with XML or well-formed HTML content.
            </p>
          </div>
        )}

        {settings.contentSelection === 'byteRange' && (
          <div className="mt-4 grid grid-cols-2 gap-4">
            <div>
              <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                Start Byte
              </label>
              <input
                type="number"
                value={settings.byteStart}
                onChange={(e) => setSettings(prev => ({ ...prev, byteStart: parseInt(e.target.value) }))}
                className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
              />
            </div>
            <div>
              <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                End Byte
              </label>
              <input
                type="number"
                value={settings.byteEnd}
                onChange={(e) => setSettings(prev => ({ ...prev, byteEnd: parseInt(e.target.value) }))}
                className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
              />
            </div>
          </div>
        )}
      </div>

      {/* Generation Controls */}
      <div className="flex justify-center">
        <button
          onClick={generateProof}
          disabled={!file || isGenerating || 
            (settings.contentSelection === 'pattern' && !settings.pattern) ||
            (settings.contentSelection === 'regex' && !settings.regex) ||
            (settings.contentSelection === 'xpath' && !settings.xpath)}
          className="inline-flex items-center px-6 py-3 border border-transparent text-base font-medium rounded-md text-white bg-blue-600 hover:bg-blue-700 disabled:bg-gray-400 disabled:cursor-not-allowed transition-colors"
        >
          <Play className="mr-2 h-5 w-5" />
          {isGenerating ? 'Generating...' : 'Generate Proof'}
        </button>
      </div>

      {/* Progress */}
      {isGenerating && (
        <div className="space-y-2">
          <Progress value={progress} className="w-full" />
          <p className="text-sm text-gray-600 dark:text-gray-400 text-center">
            {progress.toFixed(0)}% complete
          </p>
        </div>
      )}

      {/* Error */}
      {error && (
        <div className="flex items-center space-x-2 p-4 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-md">
          <AlertCircle className="h-5 w-5 text-red-500" />
          <p className="text-sm text-red-600 dark:text-red-400">{error}</p>
        </div>
      )}

      {/* Success */}
      {proof && (
        <div className="space-y-4">
          <div className="flex items-center space-x-2 p-4 bg-green-50 dark:bg-green-900/20 border border-green-200 dark:border-green-800 rounded-md">
            <CheckCircle className="h-5 w-5 text-green-500" />
            <p className="text-sm text-green-600 dark:text-green-400">
              Proof generated successfully!
            </p>
          </div>

          <div className="bg-gray-50 dark:bg-gray-800 rounded-lg p-6">
            <h4 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">
              Proof Details
            </h4>
            
            <div className="space-y-3 text-sm">
              <div>
                <span className="font-medium text-gray-700 dark:text-gray-300">Proof ID:</span>
                <span className="ml-2 font-mono text-gray-600 dark:text-gray-400">{proof.id}</span>
              </div>
              <div>
                <span className="font-medium text-gray-700 dark:text-gray-300">Content Hash:</span>
                <span className="ml-2 font-mono text-gray-600 dark:text-gray-400 break-all">
                  {proof.contentHash}
                </span>
              </div>
              <div>
                <span className="font-medium text-gray-700 dark:text-gray-300">Root Hash:</span>
                <span className="ml-2 font-mono text-gray-600 dark:text-gray-400 break-all">
                  {proof.rootHash}
                </span>
              </div>
              <div>
                <span className="font-medium text-gray-700 dark:text-gray-300">Security Level:</span>
                <span className="ml-2 text-gray-600 dark:text-gray-400">
                  {proof.metadata.securityLevel}-bit
                </span>
              </div>
            </div>

            <div className="flex space-x-3 mt-6">
              <button
                onClick={downloadProof}
                className="inline-flex items-center px-4 py-2 border border-gray-300 dark:border-gray-600 text-sm font-medium rounded-md text-gray-700 dark:text-gray-300 bg-white dark:bg-gray-700 hover:bg-gray-50 dark:hover:bg-gray-600 transition-colors"
              >
                <Download className="mr-2 h-4 w-4" />
                Download
              </button>
              <button
                onClick={copyProof}
                className="inline-flex items-center px-4 py-2 border border-gray-300 dark:border-gray-600 text-sm font-medium rounded-md text-gray-700 dark:text-gray-300 bg-white dark:bg-gray-700 hover:bg-gray-50 dark:hover:bg-gray-600 transition-colors"
              >
                <Copy className="mr-2 h-4 w-4" />
                Copy
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

export default ProofGeneration;

