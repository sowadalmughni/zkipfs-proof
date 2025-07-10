import { useState, useEffect, useCallback } from 'react'
import toast from 'react-hot-toast'

// Mock Web3 implementation for development
// In production, this would integrate with actual Web3 libraries like ethers.js or wagmi

export function useWeb3() {
  const [isConnected, setIsConnected] = useState(false)
  const [account, setAccount] = useState(null)
  const [chainId, setChainId] = useState(null)
  const [isConnecting, setIsConnecting] = useState(false)

  // Check if wallet is available
  const isWalletAvailable = useCallback(() => {
    return typeof window !== 'undefined' && window.ethereum
  }, [])

  // Connect to wallet
  const connect = useCallback(async () => {
    if (!isWalletAvailable()) {
      toast.error('Please install MetaMask or another Web3 wallet')
      return
    }

    setIsConnecting(true)
    
    try {
      // Request account access
      const accounts = await window.ethereum.request({
        method: 'eth_requestAccounts'
      })

      if (accounts.length > 0) {
        setAccount(accounts[0])
        setIsConnected(true)
        
        // Get chain ID
        const chainId = await window.ethereum.request({
          method: 'eth_chainId'
        })
        setChainId(parseInt(chainId, 16))
        
        toast.success('Wallet connected successfully!')
      }
    } catch (error) {
      console.error('Failed to connect wallet:', error)
      
      if (error.code === 4001) {
        toast.error('Wallet connection rejected by user')
      } else {
        toast.error('Failed to connect wallet')
      }
    } finally {
      setIsConnecting(false)
    }
  }, [isWalletAvailable])

  // Disconnect wallet
  const disconnect = useCallback(() => {
    setAccount(null)
    setIsConnected(false)
    setChainId(null)
    toast.success('Wallet disconnected')
  }, [])

  // Switch network
  const switchNetwork = useCallback(async (targetChainId) => {
    if (!isWalletAvailable()) {
      toast.error('Wallet not available')
      return
    }

    try {
      await window.ethereum.request({
        method: 'wallet_switchEthereumChain',
        params: [{ chainId: `0x${targetChainId.toString(16)}` }]
      })
    } catch (error) {
      console.error('Failed to switch network:', error)
      toast.error('Failed to switch network')
    }
  }, [isWalletAvailable])

  // Get network name
  const getNetworkName = useCallback((chainId) => {
    const networks = {
      1: 'Ethereum Mainnet',
      5: 'Goerli Testnet',
      11155111: 'Sepolia Testnet',
      137: 'Polygon Mainnet',
      80001: 'Polygon Mumbai',
      42161: 'Arbitrum One',
      421613: 'Arbitrum Goerli',
      10: 'Optimism',
      420: 'Optimism Goerli',
      8453: 'Base',
      84531: 'Base Goerli'
    }
    return networks[chainId] || `Chain ${chainId}`
  }, [])

  // Listen for account changes
  useEffect(() => {
    if (!isWalletAvailable()) return

    const handleAccountsChanged = (accounts) => {
      if (accounts.length === 0) {
        disconnect()
      } else if (accounts[0] !== account) {
        setAccount(accounts[0])
        toast.success('Account changed')
      }
    }

    const handleChainChanged = (chainId) => {
      setChainId(parseInt(chainId, 16))
      toast.success(`Switched to ${getNetworkName(parseInt(chainId, 16))}`)
    }

    const handleDisconnect = () => {
      disconnect()
    }

    window.ethereum.on('accountsChanged', handleAccountsChanged)
    window.ethereum.on('chainChanged', handleChainChanged)
    window.ethereum.on('disconnect', handleDisconnect)

    return () => {
      if (window.ethereum.removeListener) {
        window.ethereum.removeListener('accountsChanged', handleAccountsChanged)
        window.ethereum.removeListener('chainChanged', handleChainChanged)
        window.ethereum.removeListener('disconnect', handleDisconnect)
      }
    }
  }, [account, disconnect, getNetworkName, isWalletAvailable])

  // Check if already connected on mount
  useEffect(() => {
    const checkConnection = async () => {
      if (!isWalletAvailable()) return

      try {
        const accounts = await window.ethereum.request({
          method: 'eth_accounts'
        })

        if (accounts.length > 0) {
          setAccount(accounts[0])
          setIsConnected(true)
          
          const chainId = await window.ethereum.request({
            method: 'eth_chainId'
          })
          setChainId(parseInt(chainId, 16))
        }
      } catch (error) {
        console.error('Failed to check wallet connection:', error)
      }
    }

    checkConnection()
  }, [isWalletAvailable])

  // Mock contract interaction functions
  const verifyProofOnChain = useCallback(async (proofData) => {
    if (!isConnected) {
      toast.error('Please connect your wallet first')
      return null
    }

    try {
      // Mock verification - in production this would call the actual smart contract
      toast.loading('Verifying proof on-chain...', { duration: 2000 })
      
      // Simulate network delay
      await new Promise(resolve => setTimeout(resolve, 2000))
      
      const mockResult = {
        isValid: Math.random() > 0.2, // 80% success rate for demo
        transactionHash: '0x' + Math.random().toString(16).substr(2, 64),
        gasUsed: Math.floor(Math.random() * 100000) + 50000,
        blockNumber: Math.floor(Math.random() * 1000000) + 18000000
      }

      if (mockResult.isValid) {
        toast.success('Proof verified successfully on-chain!')
      } else {
        toast.error('Proof verification failed')
      }

      return mockResult
    } catch (error) {
      console.error('On-chain verification failed:', error)
      toast.error('On-chain verification failed')
      return null
    }
  }, [isConnected])

  const getProofFromChain = useCallback(async (proofHash) => {
    if (!isConnected) {
      toast.error('Please connect your wallet first')
      return null
    }

    try {
      // Mock retrieval - in production this would query the smart contract
      toast.loading('Fetching proof from blockchain...', { duration: 1500 })
      
      await new Promise(resolve => setTimeout(resolve, 1500))
      
      const mockProof = {
        hash: proofHash,
        isValid: true,
        verifier: account,
        timestamp: Date.now() - Math.floor(Math.random() * 86400000), // Random time in last 24h
        contentHash: '0x' + Math.random().toString(16).substr(2, 64),
        securityLevel: 128,
        proofSystem: 'Risc0'
      }

      toast.success('Proof retrieved from blockchain!')
      return mockProof
    } catch (error) {
      console.error('Failed to fetch proof from chain:', error)
      toast.error('Failed to fetch proof from blockchain')
      return null
    }
  }, [isConnected, account])

  return {
    isConnected,
    account,
    chainId,
    isConnecting,
    isWalletAvailable: isWalletAvailable(),
    networkName: chainId ? getNetworkName(chainId) : null,
    connect,
    disconnect,
    switchNetwork,
    verifyProofOnChain,
    getProofFromChain
  }
}

