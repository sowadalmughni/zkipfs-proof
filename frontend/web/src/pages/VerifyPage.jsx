import { useState } from 'react'
import { motion } from 'framer-motion'
import { CheckCircle, Upload, AlertCircle, Shield, Hash, Clock, User } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Textarea } from '@/components/ui/textarea'
import { Badge } from '@/components/ui/badge'
import { Label } from '@/components/ui/label'
import toast from 'react-hot-toast'

export default function VerifyPage() {
  const [proofInput, setProofInput] = useState('')
  const [isVerifying, setIsVerifying] = useState(false)
  const [verificationResult, setVerificationResult] = useState(null)

  const verifyProof = async () => {
    if (!proofInput.trim()) {
      toast.error('Please enter a proof to verify')
      return
    }

    setIsVerifying(true)
    
    try {
      // Simulate verification process
      await new Promise(resolve => setTimeout(resolve, 2000))
      
      // Mock verification result
      const isValid = Math.random() > 0.2 // 80% success rate for demo
      const mockResult = {
        isValid,
        proofId: 'proof_' + Math.random().toString(36).substr(2, 9),
        timestamp: new Date().toISOString(),
        verifier: '0x742d35Cc6634C0532925a3b8D4C9db96590c6C87',
        contentHash: '0x' + Array.from({length: 64}, () => Math.floor(Math.random() * 16).toString(16)).join(''),
        securityLevel: 128,
        proofSystem: 'Risc0',
        fileName: 'document.pdf',
        fileSize: '2.4 MB',
        generationTime: new Date(Date.now() - Math.floor(Math.random() * 86400000)).toISOString()
      }

      setVerificationResult(mockResult)
      
      if (isValid) {
        toast.success('Proof verified successfully!')
      } else {
        toast.error('Proof verification failed')
      }
    } catch (error) {
      console.error('Verification failed:', error)
      toast.error('Verification failed')
    } finally {
      setIsVerifying(false)
    }
  }

  const formatHash = (hash) => {
    if (!hash) return ''
    return `${hash.slice(0, 10)}...${hash.slice(-8)}`
  }

  return (
    <div className="container mx-auto px-4 sm:px-6 lg:px-8 py-8">
      <div className="max-w-4xl mx-auto">
        {/* Header */}
        <div className="text-center mb-8">
          <h1 className="text-3xl lg:text-4xl font-bold mb-4">
            Verify Zero-Knowledge Proof
          </h1>
          <p className="text-xl text-muted-foreground">
            Verify the authenticity of any proof generated with zkIPFS-Proof
          </p>
        </div>

        <div className="grid lg:grid-cols-3 gap-8">
          {/* Main Content */}
          <div className="lg:col-span-2 space-y-6">
            {/* Proof Input */}
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center space-x-2">
                  <Upload className="w-5 h-5" />
                  <span>Enter Proof</span>
                </CardTitle>
                <CardDescription>
                  Paste your proof JSON or upload a proof file to verify its authenticity
                </CardDescription>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="space-y-2">
                  <Label htmlFor="proof-input">Proof JSON</Label>
                  <Textarea
                    id="proof-input"
                    placeholder="Paste your proof JSON here..."
                    value={proofInput}
                    onChange={(e) => setProofInput(e.target.value)}
                    rows={8}
                    className="font-mono text-sm"
                  />
                </div>

                <Button 
                  onClick={verifyProof} 
                  disabled={isVerifying || !proofInput.trim()}
                  className="w-full"
                  size="lg"
                >
                  {isVerifying ? (
                    <>
                      <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-white mr-2"></div>
                      Verifying...
                    </>
                  ) : (
                    <>
                      <CheckCircle className="w-4 h-4 mr-2" />
                      Verify Proof
                    </>
                  )}
                </Button>
              </CardContent>
            </Card>

            {/* Verification Result */}
            {verificationResult && (
              <motion.div
                initial={{ opacity: 0, y: 20 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ duration: 0.3 }}
              >
                <Card>
                  <CardHeader>
                    <CardTitle className="flex items-center space-x-2">
                      {verificationResult.isValid ? (
                        <CheckCircle className="w-5 h-5 text-green-600" />
                      ) : (
                        <AlertCircle className="w-5 h-5 text-red-600" />
                      )}
                      <span>Verification Result</span>
                    </CardTitle>
                    <CardDescription>
                      {verificationResult.isValid 
                        ? 'The proof has been successfully verified'
                        : 'The proof verification failed'
                      }
                    </CardDescription>
                  </CardHeader>
                  <CardContent className="space-y-4">
                    <div className="flex flex-wrap gap-2">
                      <Badge 
                        variant={verificationResult.isValid ? "default" : "destructive"}
                      >
                        {verificationResult.isValid ? "Valid" : "Invalid"}
                      </Badge>
                      <Badge variant="secondary">
                        <Shield className="w-3 h-3 mr-1" />
                        {verificationResult.securityLevel}-bit Security
                      </Badge>
                      <Badge variant="secondary">
                        <Hash className="w-3 h-3 mr-1" />
                        {verificationResult.proofSystem}
                      </Badge>
                    </div>

                    {verificationResult.isValid && (
                      <div className="grid grid-cols-1 md:grid-cols-2 gap-4 text-sm">
                        <div>
                          <Label className="text-muted-foreground">Proof ID</Label>
                          <p className="font-mono">{verificationResult.proofId}</p>
                        </div>
                        <div>
                          <Label className="text-muted-foreground">Content Hash</Label>
                          <p className="font-mono">{formatHash(verificationResult.contentHash)}</p>
                        </div>
                        <div>
                          <Label className="text-muted-foreground">File Name</Label>
                          <p>{verificationResult.fileName}</p>
                        </div>
                        <div>
                          <Label className="text-muted-foreground">File Size</Label>
                          <p>{verificationResult.fileSize}</p>
                        </div>
                        <div>
                          <Label className="text-muted-foreground">Generated</Label>
                          <p>{new Date(verificationResult.generationTime).toLocaleString()}</p>
                        </div>
                        <div>
                          <Label className="text-muted-foreground">Verifier</Label>
                          <p className="font-mono">{formatHash(verificationResult.verifier)}</p>
                        </div>
                      </div>
                    )}
                  </CardContent>
                </Card>
              </motion.div>
            )}
          </div>

          {/* Sidebar */}
          <div className="space-y-6">
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center space-x-2">
                  <CheckCircle className="w-5 h-5" />
                  <span>How Verification Works</span>
                </CardTitle>
              </CardHeader>
              <CardContent className="space-y-3 text-sm">
                <div className="flex items-start space-x-2">
                  <div className="w-6 h-6 bg-primary/10 rounded-full flex items-center justify-center flex-shrink-0 mt-0.5">
                    <span className="text-xs font-medium text-primary">1</span>
                  </div>
                  <p>Parse the proof JSON and extract verification parameters</p>
                </div>
                <div className="flex items-start space-x-2">
                  <div className="w-6 h-6 bg-primary/10 rounded-full flex items-center justify-center flex-shrink-0 mt-0.5">
                    <span className="text-xs font-medium text-primary">2</span>
                  </div>
                  <p>Validate the cryptographic proof using the specified proof system</p>
                </div>
                <div className="flex items-start space-x-2">
                  <div className="w-6 h-6 bg-primary/10 rounded-full flex items-center justify-center flex-shrink-0 mt-0.5">
                    <span className="text-xs font-medium text-primary">3</span>
                  </div>
                  <p>Check proof integrity and security parameters</p>
                </div>
                <div className="flex items-start space-x-2">
                  <div className="w-6 h-6 bg-primary/10 rounded-full flex items-center justify-center flex-shrink-0 mt-0.5">
                    <span className="text-xs font-medium text-primary">4</span>
                  </div>
                  <p>Return verification result with detailed information</p>
                </div>
              </CardContent>
            </Card>

            <Card>
              <CardHeader>
                <CardTitle>Example Proof</CardTitle>
                <CardDescription>
                  Try verifying with this sample proof
                </CardDescription>
              </CardHeader>
              <CardContent>
                <Button
                  variant="outline"
                  onClick={() => {
                    const sampleProof = {
                      id: "proof_sample123",
                      timestamp: new Date().toISOString(),
                      fileName: "sample.pdf",
                      fileSize: 1024000,
                      contentHash: "0x1234567890abcdef...",
                      proofData: "0xabcdef1234567890...",
                      securityLevel: 128,
                      proofSystem: "risc0"
                    }
                    setProofInput(JSON.stringify(sampleProof, null, 2))
                  }}
                  className="w-full"
                >
                  Load Sample Proof
                </Button>
              </CardContent>
            </Card>
          </div>
        </div>
      </div>
    </div>
  )
}

