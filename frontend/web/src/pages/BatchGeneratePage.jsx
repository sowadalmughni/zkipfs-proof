import { Layers } from 'lucide-react'
import BatchProofGeneration from '../components/BatchProofGeneration'

export default function BatchGeneratePage() {
  return (
    <div className="container mx-auto px-4 sm:px-6 lg:px-8 py-8">
      <div className="max-w-4xl mx-auto">
        {/* Header */}
        <div className="text-center mb-8">
          <div className="flex items-center justify-center mb-4">
            <div className="w-12 h-12 bg-primary/10 rounded-xl flex items-center justify-center">
              <Layers className="w-6 h-6 text-primary" />
            </div>
          </div>
          <h1 className="text-3xl lg:text-4xl font-bold mb-4">
            Batch Proof Generation
          </h1>
          <p className="text-xl text-muted-foreground">
            Generate zero-knowledge proofs for multiple files efficiently
          </p>
        </div>

        <div className="grid lg:grid-cols-1 gap-8">
           <BatchProofGeneration />
        </div>
      </div>
    </div>
  )
}
