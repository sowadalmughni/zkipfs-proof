import React, { useState, useCallback } from 'react'
import { useDropzone } from 'react-dropzone'
import { 
  Upload, 
  File, 
  CheckCircle, 
  AlertCircle, 
  Play, 
  Trash2,
  Download,
  MoreVertical
} from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Progress } from '@/components/ui/progress'
import { Badge } from '@/components/ui/badge'
import toast from 'react-hot-toast'

export default function BatchProofGeneration() {
  const [files, setFiles] = useState([])
  const [isProcessing, setIsProcessing] = useState(false)
  const [globalProgress, setGlobalProgress] = useState(0)

  const onDrop = useCallback((acceptedFiles) => {
    const newFiles = acceptedFiles.map(file => ({
      id: Math.random().toString(36).substr(2, 9),
      file,
      status: 'pending', // pending, processing, completed, error
      progress: 0,
      result: null
    }))
    setFiles(prev => [...prev, ...newFiles])
    toast.success(`Added ${newFiles.length} files to queue`)
  }, [])

  const { getRootProps, getInputProps, isDragActive } = useDropzone({
    onDrop,
    maxSize: 50 * 1024 * 1024 * 1024, // 50GB
  })

  const removeFile = (id) => {
    setFiles(prev => prev.filter(f => f.id !== id))
  }

  const processQueue = async () => {
    setIsProcessing(true)
    setGlobalProgress(0)

    const pendingFiles = files.filter(f => f.status === 'pending')
    let processedCount = 0

    // Process files sequentially (to avoid overwhelming the local server)
    for (const fileItem of pendingFiles) {
      // Update status to processing
      setFiles(prev => prev.map(f => 
        f.id === fileItem.id ? { ...f, status: 'processing', progress: 0 } : f
      ))

      try {
        // 1. Submit Job
        const formData = new FormData();
        formData.append('file', fileItem.file);
        // Defaulting to simple pattern match or full file for batch for now. 
        // Ideally, we'd allow per-file configuration in the UI.
        formData.append('content_selection', 'pattern:Hello'); // TODO: Add UI for this
        formData.append('security_level', '128');

        // Check availability of backend
        try {
            await fetch('http://localhost:3000/health');
        } catch {
            throw new Error("Backend server is not running on http://localhost:3000");
        }

        const submitRes = await fetch('http://localhost:3000/generate', {
            method: 'POST',
            body: formData,
        });
        
        if (!submitRes.ok) throw new Error('Failed to submit job');
        const { job_id } = await submitRes.json();

        // 2. Poll Status
        let jobStatus = 'Pending';
        let result = null;
        
        while (jobStatus === 'Pending' || jobStatus === 'Processing') {
            await new Promise(resolve => setTimeout(resolve, 1000)); // Poll every 1s
            
            const statusRes = await fetch(`http://localhost:3000/status/${job_id}`);
            if (!statusRes.ok) throw new Error('Failed to check status');
            
            const statusData = await statusRes.json();
            jobStatus = statusData.status; // "Pending", "Processing", "Completed", "Failed"

            // Simulate progress update if backend doesn't provide fine-grained progress
            setFiles(prev => prev.map(f => 
                f.id === fileItem.id ? { ...f, progress: Math.min(f.progress + 10, 90) } : f
            ));

            if (jobStatus === 'Completed') {
                result = statusData.result;
            } else if (jobStatus === 'Failed') {
                throw new Error(statusData.result || 'Job failed');
            }
        }

        setFiles(prev => prev.map(f => 
          f.id === fileItem.id ? { ...f, status: 'completed', progress: 100, result } : f
        ))

      } catch (error) {
        console.error('Batch processing error:', error)
        setFiles(prev => prev.map(f => 
          f.id === fileItem.id ? { ...f, status: 'error', progress: 0 } : f
        ))
      }

      processedCount++
      setGlobalProgress((processedCount / pendingFiles.length) * 100)
    }

    setIsProcessing(false)
    toast.success('Batch processing completed!')
  }

  const downloadAll = () => {
    const completedFiles = files.filter(f => f.status === 'completed' && f.result)
    if (completedFiles.length === 0) return

    // In a real app, this might zip the proofs or trigger multiple downloads
    // For now, we'll just alert
    toast.success(`Downloading ${completedFiles.length} proofs...`)
    
    // Simulate download of first one as example
    const dataStr = JSON.stringify(completedFiles[0].result, null, 2);
    const dataBlob = new Blob([dataStr], { type: 'application/json' });
    const url = URL.createObjectURL(dataBlob);
    const link = document.createElement('a');
    link.href = url;
    link.download = `batch_proofs.json`; 
    link.click();
  }

  const clearCompleted = () => {
    setFiles(prev => prev.filter(f => f.status !== 'completed'))
  }

  return (
    <div className="space-y-6">
      {/* Upload Zone */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center space-x-2">
            <Upload className="w-5 h-5" />
            <span>Batch Upload</span>
          </CardTitle>
          <CardDescription>
            Drag and drop multiple files here to generate proofs in batches.
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
            <div className="space-y-4">
              <div className="flex items-center justify-center w-16 h-16 bg-muted rounded-lg mx-auto">
                <Upload className="w-8 h-8 text-muted-foreground" />
              </div>
              <div>
                <p className="text-lg font-medium">
                  {isDragActive ? 'Drop files here' : 'Choose files to upload'}
                </p>
                <p className="text-sm text-muted-foreground">
                  Supports multiple files selection
                </p>
              </div>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Control Bar */}
      {files.length > 0 && (
        <div className="flex items-center justify-between bg-card p-4 rounded-lg border shadow-sm">
          <div className="flex items-center space-x-4">
            <div className="text-sm text-muted-foreground">
              {files.length} file{files.length !== 1 && 's'} | 
              {files.filter(f => f.status === 'completed').length} completed
            </div>
            {isProcessing && (
               <Progress value={globalProgress} className="w-32 h-2" />
            )}
          </div>
          <div className="flex items-center space-x-2">
            <Button 
              variant="outline" 
              onClick={clearCompleted}
              disabled={isProcessing || !files.some(f => f.status === 'completed')}
            >
              Clear Completed
            </Button>
            <Button 
              onClick={processQueue} 
              disabled={isProcessing || !files.some(f => f.status === 'pending')}
            >
              <Play className="w-4 h-4 mr-2" />
              {isProcessing ? 'Processing...' : 'Start Batch'}
            </Button>
            {files.some(f => f.status === 'completed') && (
              <Button variant="secondary" onClick={downloadAll}>
                <Download className="w-4 h-4 mr-2" />
                Download All
              </Button>
            )}
          </div>
        </div>
      )}

      {/* File List */}
      <div className="space-y-3">
        {files.map((fileItem) => (
          <div 
            key={fileItem.id} 
            className="flex items-center justify-between p-4 bg-card rounded-lg border shadow-sm group"
          >
            <div className="flex items-center space-x-4 flex-1 min-w-0">
              <div className={`p-2 rounded-lg ${
                fileItem.status === 'completed' ? 'bg-green-100 text-green-600' : 
                fileItem.status === 'error' ? 'bg-red-100 text-red-600' :
                'bg-muted'
              }`}>
                {fileItem.status === 'completed' ? <CheckCircle className="w-5 h-5" /> :
                 fileItem.status === 'error' ? <AlertCircle className="w-5 h-5" /> :
                 <File className="w-5 h-5" />}
              </div>
              
              <div className="flex-1 min-w-0">
                <div className="flex items-center justify-between mb-1">
                  <p className="font-medium truncate">{fileItem.file.name}</p>
                  <span className="text-xs text-muted-foreground capitalize">
                    {fileItem.status}
                  </span>
                </div>
                {fileItem.status === 'processing' ? (
                  <Progress value={fileItem.progress} className="h-1.5" />
                ) : (
                  <div className="text-xs text-muted-foreground">
                    {(fileItem.file.size / 1024).toFixed(1)} KB
                  </div>
                )}
              </div>
            </div>

            <div className="ml-4 flex items-center space-x-2">
              <Button 
                variant="ghost" 
                size="icon" 
                onClick={() => removeFile(fileItem.id)}
                disabled={isProcessing}
                className="opacity-0 group-hover:opacity-100 transition-opacity"
              >
                <Trash2 className="w-4 h-4 text-red-500" />
              </Button>
            </div>
          </div>
        ))}
      </div>
    </div>
  )
}
