import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Book, Code, Terminal, Zap } from 'lucide-react'

export default function DocsPage() {
  return (
    <div className="container mx-auto px-4 sm:px-6 lg:px-8 py-8">
      <div className="max-w-4xl mx-auto">
        <div className="text-center mb-8">
          <h1 className="text-3xl lg:text-4xl font-bold mb-4">
            Documentation
          </h1>
          <p className="text-xl text-muted-foreground">
            Learn how to use zkIPFS-Proof for your zero-knowledge verification needs
          </p>
        </div>

        <div className="grid md:grid-cols-2 gap-6">
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center space-x-2">
                <Book className="w-5 h-5" />
                <span>Getting Started</span>
              </CardTitle>
              <CardDescription>
                Quick start guide and basic concepts
              </CardDescription>
            </CardHeader>
            <CardContent>
              <p className="text-sm text-muted-foreground">
                Coming soon - Comprehensive guide to get you started with zkIPFS-Proof.
              </p>
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle className="flex items-center space-x-2">
                <Code className="w-5 h-5" />
                <span>API Reference</span>
              </CardTitle>
              <CardDescription>
                Complete API documentation and examples
              </CardDescription>
            </CardHeader>
            <CardContent>
              <p className="text-sm text-muted-foreground">
                Coming soon - Detailed API reference with code examples and integration guides.
              </p>
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle className="flex items-center space-x-2">
                <Terminal className="w-5 h-5" />
                <span>CLI Usage</span>
              </CardTitle>
              <CardDescription>
                Command-line interface documentation
              </CardDescription>
            </CardHeader>
            <CardContent>
              <p className="text-sm text-muted-foreground">
                Coming soon - Complete CLI documentation with examples and best practices.
              </p>
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle className="flex items-center space-x-2">
                <Zap className="w-5 h-5" />
                <span>Advanced Topics</span>
              </CardTitle>
              <CardDescription>
                Deep dive into zero-knowledge proofs and IPFS
              </CardDescription>
            </CardHeader>
            <CardContent>
              <p className="text-sm text-muted-foreground">
                Coming soon - Advanced concepts, security considerations, and optimization techniques.
              </p>
            </CardContent>
          </Card>
        </div>
      </div>
    </div>
  )
}

