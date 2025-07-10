import React, { useState, useEffect } from 'react';
import { Search, Filter, Calendar, Hash, Shield, Eye, Download } from 'lucide-react';

const ProofExplorer = () => {
  const [proofs, setProofs] = useState([]);
  const [filteredProofs, setFilteredProofs] = useState([]);
  const [searchTerm, setSearchTerm] = useState('');
  const [filterType, setFilterType] = useState('all');
  const [sortBy, setSortBy] = useState('date');
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    // Simulate loading proofs from a database or API
    const mockProofs = [
      {
        id: 'proof_1640995200001',
        contentHash: 'a1b2c3d4e5f6789012345678901234567890123456789012345678901234567890',
        rootHash: 'b2c3d4e5f6789012345678901234567890123456789012345678901234567890a1',
        fileName: 'financial_report_2024.pdf',
        fileSize: 15728640, // 15MB
        securityLevel: 128,
        createdAt: '2024-01-01T12:00:00Z',
        contentPattern: 'Transaction ID: TX-2024-001',
        verified: true,
        verificationCount: 5
      },
      {
        id: 'proof_1640995200002',
        contentHash: 'c3d4e5f6789012345678901234567890123456789012345678901234567890a1b2',
        rootHash: 'd4e5f6789012345678901234567890123456789012345678901234567890a1b2c3',
        fileName: 'research_data.csv',
        fileSize: 52428800, // 50MB
        securityLevel: 256,
        createdAt: '2024-01-02T14:30:00Z',
        contentPattern: 'Patient ID: P-2024-0157',
        verified: true,
        verificationCount: 12
      },
      {
        id: 'proof_1640995200003',
        contentHash: 'e5f6789012345678901234567890123456789012345678901234567890a1b2c3d4',
        rootHash: 'f6789012345678901234567890123456789012345678901234567890a1b2c3d4e5',
        fileName: 'leaked_documents.txt',
        fileSize: 2097152, // 2MB
        securityLevel: 192,
        createdAt: '2024-01-03T09:15:00Z',
        contentPattern: 'Confidential: Project Codename Alpha',
        verified: false,
        verificationCount: 0
      }
    ];

    setTimeout(() => {
      setProofs(mockProofs);
      setFilteredProofs(mockProofs);
      setLoading(false);
    }, 1000);
  }, []);

  useEffect(() => {
    let filtered = proofs;

    // Apply search filter
    if (searchTerm) {
      filtered = filtered.filter(proof => 
        proof.fileName.toLowerCase().includes(searchTerm.toLowerCase()) ||
        proof.contentPattern.toLowerCase().includes(searchTerm.toLowerCase()) ||
        proof.id.toLowerCase().includes(searchTerm.toLowerCase())
      );
    }

    // Apply type filter
    if (filterType !== 'all') {
      filtered = filtered.filter(proof => {
        switch (filterType) {
          case 'verified':
            return proof.verified;
          case 'unverified':
            return !proof.verified;
          case 'high-security':
            return proof.securityLevel >= 256;
          default:
            return true;
        }
      });
    }

    // Apply sorting
    filtered.sort((a, b) => {
      switch (sortBy) {
        case 'date':
          return new Date(b.createdAt) - new Date(a.createdAt);
        case 'size':
          return b.fileSize - a.fileSize;
        case 'security':
          return b.securityLevel - a.securityLevel;
        case 'verifications':
          return b.verificationCount - a.verificationCount;
        default:
          return 0;
      }
    });

    setFilteredProofs(filtered);
  }, [proofs, searchTerm, filterType, sortBy]);

  const formatFileSize = (bytes) => {
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  };

  const formatDate = (dateString) => {
    return new Date(dateString).toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit'
    });
  };

  const truncateHash = (hash, length = 16) => {
    return `${hash.substring(0, length)}...${hash.substring(hash.length - 8)}`;
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center min-h-64">
        <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600"></div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
        <div>
          <h2 className="text-2xl font-bold text-gray-900 dark:text-white">Proof Explorer</h2>
          <p className="text-gray-600 dark:text-gray-400">
            Browse and explore zero-knowledge proofs
          </p>
        </div>
        <div className="text-sm text-gray-500 dark:text-gray-400">
          {filteredProofs.length} of {proofs.length} proofs
        </div>
      </div>

      {/* Search and Filters */}
      <div className="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-6">
        <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
          {/* Search */}
          <div className="md:col-span-2">
            <div className="relative">
              <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-gray-400" />
              <input
                type="text"
                placeholder="Search proofs by filename, content, or ID..."
                value={searchTerm}
                onChange={(e) => setSearchTerm(e.target.value)}
                className="w-full pl-10 pr-4 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-blue-500"
              />
            </div>
          </div>

          {/* Filter */}
          <div>
            <select
              value={filterType}
              onChange={(e) => setFilterType(e.target.value)}
              className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-blue-500"
            >
              <option value="all">All Proofs</option>
              <option value="verified">Verified</option>
              <option value="unverified">Unverified</option>
              <option value="high-security">High Security</option>
            </select>
          </div>

          {/* Sort */}
          <div>
            <select
              value={sortBy}
              onChange={(e) => setSortBy(e.target.value)}
              className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-blue-500"
            >
              <option value="date">Sort by Date</option>
              <option value="size">Sort by Size</option>
              <option value="security">Sort by Security</option>
              <option value="verifications">Sort by Verifications</option>
            </select>
          </div>
        </div>
      </div>

      {/* Proof List */}
      <div className="space-y-4">
        {filteredProofs.length === 0 ? (
          <div className="text-center py-12">
            <Search className="mx-auto h-12 w-12 text-gray-400 mb-4" />
            <h3 className="text-lg font-medium text-gray-900 dark:text-white mb-2">
              No proofs found
            </h3>
            <p className="text-gray-600 dark:text-gray-400">
              Try adjusting your search terms or filters
            </p>
          </div>
        ) : (
          filteredProofs.map((proof) => (
            <div
              key={proof.id}
              className="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-6 hover:shadow-md transition-shadow"
            >
              <div className="flex items-start justify-between">
                <div className="flex-1 min-w-0">
                  {/* Header */}
                  <div className="flex items-center space-x-3 mb-3">
                    <div className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${
                      proof.verified 
                        ? 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200'
                        : 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200'
                    }`}>
                      {proof.verified ? 'Verified' : 'Unverified'}
                    </div>
                    <div className="flex items-center text-sm text-gray-500 dark:text-gray-400">
                      <Shield className="h-4 w-4 mr-1" />
                      {proof.securityLevel}-bit
                    </div>
                    <div className="flex items-center text-sm text-gray-500 dark:text-gray-400">
                      <Calendar className="h-4 w-4 mr-1" />
                      {formatDate(proof.createdAt)}
                    </div>
                  </div>

                  {/* File Info */}
                  <div className="mb-3">
                    <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-1">
                      {proof.fileName}
                    </h3>
                    <p className="text-sm text-gray-600 dark:text-gray-400">
                      {formatFileSize(proof.fileSize)} • {proof.verificationCount} verification{proof.verificationCount !== 1 ? 's' : ''}
                    </p>
                  </div>

                  {/* Content Pattern */}
                  <div className="mb-4">
                    <p className="text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                      Proven Content:
                    </p>
                    <p className="text-sm text-gray-600 dark:text-gray-400 font-mono bg-gray-50 dark:bg-gray-700 px-2 py-1 rounded">
                      "{proof.contentPattern}"
                    </p>
                  </div>

                  {/* Hashes */}
                  <div className="grid grid-cols-1 md:grid-cols-2 gap-4 text-xs">
                    <div>
                      <div className="flex items-center text-gray-700 dark:text-gray-300 mb-1">
                        <Hash className="h-3 w-3 mr-1" />
                        Content Hash
                      </div>
                      <p className="font-mono text-gray-600 dark:text-gray-400 break-all">
                        {truncateHash(proof.contentHash)}
                      </p>
                    </div>
                    <div>
                      <div className="flex items-center text-gray-700 dark:text-gray-300 mb-1">
                        <Hash className="h-3 w-3 mr-1" />
                        Root Hash
                      </div>
                      <p className="font-mono text-gray-600 dark:text-gray-400 break-all">
                        {truncateHash(proof.rootHash)}
                      </p>
                    </div>
                  </div>
                </div>

                {/* Actions */}
                <div className="flex flex-col space-y-2 ml-4">
                  <button className="inline-flex items-center px-3 py-1.5 border border-gray-300 dark:border-gray-600 text-sm font-medium rounded-md text-gray-700 dark:text-gray-300 bg-white dark:bg-gray-700 hover:bg-gray-50 dark:hover:bg-gray-600 transition-colors">
                    <Eye className="h-4 w-4 mr-1" />
                    View
                  </button>
                  <button className="inline-flex items-center px-3 py-1.5 border border-gray-300 dark:border-gray-600 text-sm font-medium rounded-md text-gray-700 dark:text-gray-300 bg-white dark:bg-gray-700 hover:bg-gray-50 dark:hover:bg-gray-600 transition-colors">
                    <Download className="h-4 w-4 mr-1" />
                    Download
                  </button>
                </div>
              </div>
            </div>
          ))
        )}
      </div>
    </div>
  );
};

export default ProofExplorer;

