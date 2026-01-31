import { useState, useCallback } from 'react'
import { AnimatePresence } from 'framer-motion'
import { useDropzone } from 'react-dropzone'
import { 
  Upload, 
  File, 
  CheckCircle, 
  AlertCircle, 
  Download,
  Copy,
  Share2,
  Settings,
  Info,
  Zap,
  Shield,
  Clock,
  Hash
} from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Progress } from '@/components/ui/progress'
import { Badge } from '@/components/ui/badge'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { Label } from '@/components/ui/label'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Textarea } from '@/components/ui/textarea'
import { Switch } from '@/components/ui/switch'
import toast from 'react-hot-toast'

export default function GeneratePage() {
  const [uploadedFile, setUploadedFile] = useState(null)
  const [isGenerating, setIsGenerating] = useState(false)
  const [generationProgress, setGenerationProgress] = useState(0)
  const [generatedProof, setGeneratedProof] = useState(null)
  const [proofSettings, setProofSettings] = useState({
    securityLevel: '128',
    proofSystem: 'risc0',
    contentSelection: 'full',
    selectionValue: '',
    includeMetadata: true,
    enableSharing: true
  })

  const onDrop = useCallback((acceptedFiles) => {
    const file = acceptedFiles[0]
    if (file) {
      setUploadedFile(file)
      setGeneratedProof(null)
      toast.success(`File "${file.name}" uploaded successfully`)
    }
  }, [])

  const { getRootProps, getInputProps, isDragActive } = useDropzone({
    onDrop,
    maxFiles: 1,
    maxSize: 50 * 1024 * 1024 * 1024, // 50GB
    accept: {
      '*/*': []
    }
  })

  // Validate Regex pattern
  const isValidRegex = (pattern) => {
    if (!pattern) return false
    try {
      new RegExp(pattern)
      return true
    } catch {
      return false
    }
  }

  const generateProof = async () => {
    if (!uploadedFile) {
      toast.error('Please upload a file first')
      return
    }

    // Validate Regex if selected
    if (proofSettings.contentSelection === 'regex') {
      if (!proofSettings.selectionValue) {
        toast.error('Please enter a regex pattern')
        return
      }
      if (!isValidRegex(proofSettings.selectionValue)) {
        toast.error('Invalid regex pattern')
        return
      }
    }

    setIsGenerating(true)
    setGenerationProgress(0)

    try {
      // Simulate proof generation process
      const steps = [
        { message: 'Reading file content...', progress: 10 },
        { message: 'Computing IPFS hash...', progress: 25 },
        { message: 'Generating ZK circuit inputs...', progress: 40 },
        { message: 'Running Risc0 prover...', progress: 70 },
        { message: 'Finalizing proof...', progress: 90 },
        { message: 'Proof generated successfully!', progress: 100 }
      ]

      for (const step of steps) {
        await new Promise(resolve => setTimeout(resolve, 1000))
        setGenerationProgress(step.progress)
        toast.loading(step.message, { duration: 800 })
      }

      // Mock generated proof
      const mockProof = {
        id: 'proof_' + Math.random().toString(36).substr(2, 9),
        timestamp: new Date().toISOString(),
        fileName: uploadedFile.name,
        fileSize: uploadedFile.size,
        fileHash: '0x' + Array.from({length: 64}, () => Math.floor(Math.random() * 16).toString(16)).join(''),
        contentHash: '0x' + Array.from({length: 64}, () => Math.floor(Math.random() * 16).toString(16)).join(''),
        rootHash: '0x' + Array.from({length: 64}, () => Math.floor(Math.random() * 16).toString(16)).join(''),
        proofData: '0x' + Array.from({length: 256}, () => Math.floor(Math.random() * 16).toString(16)).join(''),
        securityLevel: parseInt(proofSettings.securityLevel),
        proofSystem: proofSettings.proofSystem,
        // Add content selection metadata
        contentSelection: proofSettings.contentSelection,
        selectionValue: proofSettings.selectionValue,
        generationTime: Math.floor(Math.random() * 30) + 10, // 10-40 seconds
        zkCycles: Math.floor(Math.random() * 1000000) + 500000,
        shareableUrl: `https://zkipfs-proof.com/verify/${Math.random().toString(36).substr(2, 9)}`
      }

      setGeneratedProof(mockProof)
      toast.success('Proof generated successfully!')
    } catch (error) {
      console.error('Proof generation failed:', error)
      toast.error('Failed to generate proof')
    } finally {
      setIsGenerating(false)
    }
  }

  const copyToClipboard = (text, label) => {
    navigator.clipboard.writeText(text)
    toast.success(`${label} copied to clipboard`)
  }

  const downloadProof = () => {
    if (!generatedProof) return

    const proofJson = JSON.stringify(generatedProof, null, 2)
    const blob = new Blob([proofJson], { type: 'application/json' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = `${generatedProof.id}.json`
    document.body.appendChild(a)
    a.click()
    document.body.removeChild(a)
    URL.revokeObjectURL(url)
    toast.success('Proof downloaded successfully')
  }

  const formatFileSize = (bytes) => {
    if (bytes === 0) return '0 Bytes'
    const k = 1024
    const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB']
    const i = Math.floor(Math.log(bytes) / Math.log(k))
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
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
            Generate Zero-Knowledge Proof
          </h1>
          <p className="text-xl text-muted-foreground">
            Upload any file and generate a cryptographic proof of its authenticity
          </p>
        </div>

        <div className="grid lg:grid-cols-3 gap-8">
          {/* Main Content */}
          <div className="lg:col-span-2 space-y-6">
            {/* File Upload */}
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center space-x-2">
                  <Upload className="w-5 h-5" />
                  <span>Upload File</span>
                </CardTitle>
                <CardDescription>
                  Drag and drop your file here, or click to browse. Maximum file size: 50GB
                </CardDescription>
              </CardHeader>
              <CardContent>
                <div
                  {...getRootProps()}
                  className={`border-2 border-dashed rounded-lg p-8 text-center cursor-pointer transition-colors ${
                    isDragActive 
                      ? 'border-primary bg-primary/5' 
                      : 'border-muted-foreground/25 hover:border-primary/50'
                  }`}
                >
                  <input {...getInputProps()} />
                  {uploadedFile ? (
                    <div className="space-y-4">
                      <div className="flex items-center justify-center w-16 h-16 bg-primary/10 rounded-lg mx-auto">
                        <File className="w-8 h-8 text-primary" />
                      </div>
                      <div>
                        <p className="font-medium">{uploadedFile.name}</p>
                        <p className="text-sm text-muted-foreground">
                          {formatFileSize(uploadedFile.size)}
                        </p>
                      </div>
                      <Button
                        variant="outline"
                        onClick={(e) => {
                          e.stopPropagation()
                          setUploadedFile(null)
                          setGeneratedProof(null)
                        }}
                      >
                        Remove File
                      </Button>
                    </div>
                  ) : (
                    <div className="space-y-4">
                      <div className="flex items-center justify-center w-16 h-16 bg-muted rounded-lg mx-auto">
                        <Upload className="w-8 h-8 text-muted-foreground" />
                      </div>
                      <div>
                        <p className="text-lg font-medium">
                          {isDragActive ? 'Drop your file here' : 'Choose a file to upload'}
                        </p>
                        <p className="text-sm text-muted-foreground">
                          Supports all file types up to 50GB
                        </p>
                      </div>
                    </div>
                  )}
                </div>
              </CardContent>
            </Card>

            {/* Proof Generation */}
            {uploadedFile && (
              <Card>
                <CardHeader>
                  <CardTitle className="flex items-center space-x-2">
                    <Zap className="w-5 h-5" />
                    <span>Generate Proof</span>
                  </CardTitle>
                  <CardDescription>
                    Create a zero-knowledge proof for your uploaded file
                  </CardDescription>
                </CardHeader>
                <CardContent className="space-y-4">
                  {isGenerating ? (
                    <div className="space-y-4">
                      <div className="flex items-center space-x-2">
                        <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-primary"></div>
                        <span className="text-sm">Generating proof...</span>
                      </div>
                      <Progress value={generationProgress} className="w-full" />
                      <p className="text-sm text-muted-foreground">
                        This may take a few minutes depending on file size and complexity
                      </p>
                    </div>
                  ) : generatedProof ? (
                    <div className="space-y-4">
                      <div className="flex items-center space-x-2 text-green-600">
                        <CheckCircle className="w-5 h-5" />
                        <span className="font-medium">Proof generated successfully!</span>
                      </div>
                      <div className="grid grid-cols-2 gap-4 text-sm">
                        <div>
                          <span className="text-muted-foreground">Generation Time:</span>
                          <span className="ml-2 font-mono">{generatedProof.generationTime}s</span>
                        </div>
                        <div>
                          <span className="text-muted-foreground">ZK Cycles:</span>
                          <span className="ml-2 font-mono">{generatedProof.zkCycles.toLocaleString()}</span>
                        </div>
                      </div>
                    </div>
                  ) : (
                    <Button 
                      onClick={generateProof} 
                      className="w-full"
                      size="lg"
                    >
                      <Zap className="w-4 h-4 mr-2" />
                      Generate Proof
                    </Button>
                  )}
                </CardContent>
              </Card>
            )}

            {/* Generated Proof Details */}
            {generatedProof && (
              <Card>
                <CardHeader>
                  <CardTitle className="flex items-center space-x-2">
                    <Shield className="w-5 h-5" />
                    <span>Proof Details</span>
                  </CardTitle>
                  <CardDescription>
                    Your zero-knowledge proof has been generated successfully
                  </CardDescription>
                </CardHeader>
                <CardContent>
                  <Tabs defaultValue="overview" className="w-full">
                    <TabsList className="grid w-full grid-cols-3">
                      <TabsTrigger value="overview">Overview</TabsTrigger>
                      <TabsTrigger value="technical">Technical</TabsTrigger>
                      <TabsTrigger value="share">Share</TabsTrigger>
                    </TabsList>
                    
                    <TabsContent value="overview" className="space-y-4">
                      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                        <div className="space-y-2">
                          <Label className="text-sm font-medium">Proof ID</Label>
                          <div className="flex items-center space-x-2">
                            <code className="flex-1 p-2 bg-muted rounded text-sm font-mono">
                              {generatedProof.id}
                            </code>
                            <Button
                              variant="outline"
                              size="sm"
                              onClick={() => copyToClipboard(generatedProof.id, 'Proof ID')}
                            >
                              <Copy className="w-4 h-4" />
                            </Button>
                          </div>
                        </div>
                        
                        <div className="space-y-2">
                          <Label className="text-sm font-medium">File Hash</Label>
                          <div className="flex items-center space-x-2">
                            <code className="flex-1 p-2 bg-muted rounded text-sm font-mono">
                              {formatHash(generatedProof.fileHash)}
                            </code>
                            <Button
                              variant="outline"
                              size="sm"
                              onClick={() => copyToClipboard(generatedProof.fileHash, 'File Hash')}
                            >
                              <Copy className="w-4 h-4" />
                            </Button>
                          </div>
                        </div>
                      </div>

                      <div className="flex flex-wrap gap-2">
                        <Badge variant="secondary">
                          <Shield className="w-3 h-3 mr-1" />
                          {generatedProof.securityLevel}-bit Security
                        </Badge>
                        <Badge variant="secondary">
                          <Hash className="w-3 h-3 mr-1" />
                          {generatedProof.proofSystem.toUpperCase()}
                        </Badge>
                        <Badge variant="secondary">
                          <Clock className="w-3 h-3 mr-1" />
                          {new Date(generatedProof.timestamp).toLocaleString()}
                        </Badge>
                      </div>

                      <div className="flex space-x-2">
                        <Button onClick={downloadProof} className="flex-1">
                          <Download className="w-4 h-4 mr-2" />
                          Download Proof
                        </Button>
                        <Button 
                          variant="outline" 
                          onClick={() => copyToClipboard(JSON.stringify(generatedProof, null, 2), 'Proof JSON')}
                        >
                          <Copy className="w-4 h-4" />
                        </Button>
                      </div>
                    </TabsContent>

                    <TabsContent value="technical" className="space-y-4">
                      <div className="space-y-4">
                        <div>
                          <Label className="text-sm font-medium">Content Hash</Label>
                          <code className="block p-2 bg-muted rounded text-sm font-mono mt-1 break-all">
                            {generatedProof.contentHash}
                          </code>
                        </div>
                        
                        <div>
                          <Label className="text-sm font-medium">Root Hash</Label>
                          <code className="block p-2 bg-muted rounded text-sm font-mono mt-1 break-all">
                            {generatedProof.rootHash}
                          </code>
                        </div>
                        
                        <div>
                          <Label className="text-sm font-medium">Proof Data</Label>
                          <Textarea
                            value={generatedProof.proofData}
                            readOnly
                            className="font-mono text-xs"
                            rows={4}
                          />
                        </div>
                      </div>
                    </TabsContent>

                    <TabsContent value="share" className="space-y-4">
                      <div className="space-y-4">
                        <div>
                          <Label className="text-sm font-medium">Shareable Verification URL</Label>
                          <div className="flex items-center space-x-2 mt-1">
                            <code className="flex-1 p-2 bg-muted rounded text-sm">
                              {generatedProof.shareableUrl}
                            </code>
                            <Button
                              variant="outline"
                              size="sm"
                              onClick={() => copyToClipboard(generatedProof.shareableUrl, 'Verification URL')}
                            >
                              <Copy className="w-4 h-4" />
                            </Button>
                          </div>
                        </div>

                        <Button className="w-full">
                          <Share2 className="w-4 h-4 mr-2" />
                          Share Proof
                        </Button>
                      </div>
                    </TabsContent>
                  </Tabs>
                </CardContent>
              </Card>
            )}
          </div>

          {/* Settings Sidebar */}
          <div className="space-y-6">
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center space-x-2">
                  <Settings className="w-5 h-5" />
                  <span>Proof Settings</span>
                </CardTitle>
                <CardDescription>
                  Configure your proof generation parameters
                </CardDescription>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="space-y-2">
                  <Label htmlFor="security-level">Security Level</Label>
                  <Select 
                    value={proofSettings.securityLevel} 
                    onValueChange={(value) => setProofSettings(prev => ({ ...prev, securityLevel: value }))}
                  >
                    <SelectTrigger>
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="80">80-bit (Fast)</SelectItem>
                      <SelectItem value="128">128-bit (Recommended)</SelectItem>
                      <SelectItem value="192">192-bit (High Security)</SelectItem>
                      <SelectItem value="256">256-bit (Quantum Resistant)</SelectItem>
                    </SelectContent>
                  </Select>
                </div>

                <div className="space-y-2">
                  <Label htmlFor="proof-system">Proof System</Label>
                  <Select 
                    value={proofSettings.proofSystem} 
                    onValueChange={(value) => setProofSettings(prev => ({ ...prev, proofSystem: value }))}
                  >
                    <SelectTrigger>
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="risc0">Risc0 (Recommended)</SelectItem>
                      <SelectItem value="groth16">Groth16</SelectItem>
                      <SelectItem value="plonk">PLONK</SelectItem>
                    </SelectContent>
                  </Select>
                </div>

                <div className="space-y-2">
                  <Label htmlFor="content-selection">Content Selection</Label>
                  <Select 
                    value={proofSettings.contentSelection} 
                    onValueChange={(value) => setProofSettings(prev => ({ ...prev, contentSelection: value }))}
                  >
                    <SelectTrigger>
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="full">Full File</SelectItem>
                      <SelectItem value="range">Byte Range (e.g. 100-200)</SelectItem>
                      <SelectItem value="pattern">Pattern Match</SelectItem>
                      <SelectItem value="regex">Regular Expression</SelectItem>
                    </SelectContent>
                  </Select>
                </div>

                {proofSettings.contentSelection !== 'full' && (
                  <div className="space-y-2">
                    <Label htmlFor="selection-value">
                      {proofSettings.contentSelection === 'range' ? 'Byte Range (start-end)' : 
                       proofSettings.contentSelection === 'regex' ? 'Regex Pattern' : 'Search Pattern'}
                    </Label>
                    <div className="relative">
                      <input
                        id="selection-value"
                        type="text"
                        className="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50"
                        placeholder={
                          proofSettings.contentSelection === 'range' ? '1024-2048' :
                          proofSettings.contentSelection === 'regex' ? '^([a-z0-9_\\.-]+)@([\\da-z\\.-]+)\\.([a-z\\.]{2,6})$' :
                          'Secret content to find...'
                        }
                        value={proofSettings.selectionValue}
                        onChange={(e) => setProofSettings(prev => ({ ...prev, selectionValue: e.target.value }))}
                      />
                      {proofSettings.contentSelection === 'regex' && proofSettings.selectionValue && !isValidRegex(proofSettings.selectionValue) && (
                        <p className="absolute -bottom-5 left-0 text-xs text-red-500">
                          Invalid regular expression
                        </p>
                      )}
                    </div>
                  </div>
                )}

                <div className="flex items-center justify-between">
                  <Label htmlFor="include-metadata">Include Metadata</Label>
                  <Switch
                    id="include-metadata"
                    checked={proofSettings.includeMetadata}
                    onCheckedChange={(checked) => setProofSettings(prev => ({ ...prev, includeMetadata: checked }))}
                  />
                </div>

                <div className="flex items-center justify-between">
                  <Label htmlFor="enable-sharing">Enable Sharing</Label>
                  <Switch
                    id="enable-sharing"
                    checked={proofSettings.enableSharing}
                    onCheckedChange={(checked) => setProofSettings(prev => ({ ...prev, enableSharing: checked }))}
                  />
                </div>
              </CardContent>
            </Card>

            <Card>
              <CardHeader>
                <CardTitle className="flex items-center space-x-2">
                  <Info className="w-5 h-5" />
                  <span>How it Works</span>
                </CardTitle>
              </CardHeader>
              <CardContent className="space-y-3 text-sm">
                <div className="flex items-start space-x-2">
                  <div className="w-6 h-6 bg-primary/10 rounded-full flex items-center justify-center shrink-0 mt-0.5">
                    <span className="text-xs font-medium text-primary">1</span>
                  </div>
                  <p>Your file is processed locally - it never leaves your device</p>
                </div>
                <div className="flex items-start space-x-2">
                  <div className="w-6 h-6 bg-primary/10 rounded-full flex items-center justify-center shrink-0 mt-0.5">
                    <span className="text-xs font-medium text-primary">2</span>
                  </div>
                  <p>A cryptographic hash is computed using IPFS content addressing</p>
                </div>
                <div className="flex items-start space-x-2">
                  <div className="w-6 h-6 bg-primary/10 rounded-full flex items-center justify-center shrink-0 mt-0.5">
                    <span className="text-xs font-medium text-primary">3</span>
                  </div>
                  <p>Zero-knowledge proof is generated using Risc0 ZK-VM</p>
                </div>
                <div className="flex items-start space-x-2">
                  <div className="w-6 h-6 bg-primary/10 rounded-full flex items-center justify-center shrink-0 mt-0.5">
                    <span className="text-xs font-medium text-primary">4</span>
                  </div>
                  <p>Proof can be verified by anyone without revealing file content</p>
                </div>
              </CardContent>
            </Card>
          </div>
        </div>
      </div>
    </div>
  )
}

