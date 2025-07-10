import { motion } from 'framer-motion'
import { Link } from 'react-router-dom'
import { 
  Shield, 
  Upload, 
  CheckCircle, 
  Globe, 
  Zap, 
  Lock,
  ArrowRight,
  Github,
  ExternalLink,
  FileText,
  Users,
  Star,
  GitFork
} from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'

export default function HomePage() {
  const features = [
    {
      icon: Shield,
      title: "Zero-Knowledge Proofs",
      description: "Prove file authenticity without revealing content using cutting-edge Risc0 ZK-VM technology."
    },
    {
      icon: Upload,
      title: "Drag & Drop Interface",
      description: "Simple, intuitive file upload with real-time progress tracking and instant proof generation."
    },
    {
      icon: CheckCircle,
      title: "On-Chain Verification",
      description: "Verify proofs on multiple blockchains with gas-optimized smart contracts."
    },
    {
      icon: Globe,
      title: "IPFS Integration",
      description: "Decentralized storage with content addressing for permanent, tamper-proof file records."
    },
    {
      icon: Zap,
      title: "Lightning Fast",
      description: "Optimized proof generation with 7-10x performance improvements over traditional methods."
    },
    {
      icon: Lock,
      title: "Privacy First",
      description: "Your files never leave your device. Generate proofs locally with complete privacy."
    }
  ]

  const useCases = [
    {
      title: "Journalism",
      description: "Prove leaked documents are authentic without revealing sources or sensitive content.",
      icon: "📰"
    },
    {
      title: "Research",
      description: "Verify data integrity in scientific papers and reproducible research.",
      icon: "🔬"
    },
    {
      title: "Legal",
      description: "Establish evidence authenticity in legal proceedings with cryptographic guarantees.",
      icon: "⚖️"
    },
    {
      title: "Auditing",
      description: "Prove compliance and data integrity without exposing confidential information.",
      icon: "📊"
    }
  ]

  const stats = [
    { label: "Proofs Generated", value: "10,000+", icon: Shield },
    { label: "Files Verified", value: "50TB+", icon: FileText },
    { label: "Active Users", value: "1,200+", icon: Users },
    { label: "GitHub Stars", value: "2,500+", icon: Star }
  ]

  return (
    <div className="min-h-screen">
      {/* Hero Section */}
      <section className="relative overflow-hidden bg-gradient-to-br from-background via-background to-muted/20">
        <div className="absolute inset-0 bg-grid-pattern opacity-5"></div>
        <div className="container mx-auto px-4 sm:px-6 lg:px-8 py-20 lg:py-32">
          <div className="text-center max-w-4xl mx-auto">
            <motion.div
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.6 }}
            >
              <h1 className="text-4xl sm:text-5xl lg:text-6xl font-bold tracking-tight mb-6">
                Prove File Authenticity
                <span className="block bg-gradient-to-r from-primary to-primary/70 bg-clip-text text-transparent">
                  Without Revealing Content
                </span>
              </h1>
              
              <p className="text-xl text-muted-foreground mb-8 max-w-2xl mx-auto">
                Generate zero-knowledge proofs for any file using Risc0 ZK-VM. 
                Perfect for journalists, researchers, and anyone who needs to prove 
                data integrity while maintaining privacy.
              </p>

              <div className="flex flex-col sm:flex-row gap-4 justify-center items-center">
                <Button asChild size="lg" className="text-lg px-8 py-6">
                  <Link to="/generate" className="flex items-center space-x-2">
                    <Upload className="w-5 h-5" />
                    <span>Generate Proof</span>
                    <ArrowRight className="w-4 h-4" />
                  </Link>
                </Button>
                
                <Button variant="outline" size="lg" asChild className="text-lg px-8 py-6">
                  <Link to="/verify" className="flex items-center space-x-2">
                    <CheckCircle className="w-5 h-5" />
                    <span>Verify Proof</span>
                  </Link>
                </Button>
              </div>

              <div className="mt-8 flex items-center justify-center space-x-6 text-sm text-muted-foreground">
                <div className="flex items-center space-x-2">
                  <div className="w-2 h-2 bg-green-500 rounded-full"></div>
                  <span>Open Source</span>
                </div>
                <div className="flex items-center space-x-2">
                  <div className="w-2 h-2 bg-blue-500 rounded-full"></div>
                  <span>MIT Licensed</span>
                </div>
                <div className="flex items-center space-x-2">
                  <div className="w-2 h-2 bg-purple-500 rounded-full"></div>
                  <span>Privacy First</span>
                </div>
              </div>
            </motion.div>
          </div>
        </div>
      </section>

      {/* Stats Section */}
      <section className="py-16 bg-muted/30">
        <div className="container mx-auto px-4 sm:px-6 lg:px-8">
          <div className="grid grid-cols-2 lg:grid-cols-4 gap-8">
            {stats.map((stat, index) => (
              <motion.div
                key={stat.label}
                initial={{ opacity: 0, y: 20 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ duration: 0.6, delay: index * 0.1 }}
                className="text-center"
              >
                <div className="flex items-center justify-center w-12 h-12 bg-primary/10 rounded-lg mx-auto mb-4">
                  <stat.icon className="w-6 h-6 text-primary" />
                </div>
                <div className="text-3xl font-bold text-foreground mb-2">{stat.value}</div>
                <div className="text-sm text-muted-foreground">{stat.label}</div>
              </motion.div>
            ))}
          </div>
        </div>
      </section>

      {/* Features Section */}
      <section className="py-20">
        <div className="container mx-auto px-4 sm:px-6 lg:px-8">
          <div className="text-center mb-16">
            <h2 className="text-3xl lg:text-4xl font-bold mb-4">
              Powerful Features for Modern Privacy
            </h2>
            <p className="text-xl text-muted-foreground max-w-2xl mx-auto">
              Built with cutting-edge cryptography and designed for real-world use cases
            </p>
          </div>

          <div className="grid md:grid-cols-2 lg:grid-cols-3 gap-8">
            {features.map((feature, index) => (
              <motion.div
                key={feature.title}
                initial={{ opacity: 0, y: 20 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ duration: 0.6, delay: index * 0.1 }}
              >
                <Card className="h-full hover:shadow-lg transition-shadow">
                  <CardHeader>
                    <div className="flex items-center space-x-3">
                      <div className="flex items-center justify-center w-10 h-10 bg-primary/10 rounded-lg">
                        <feature.icon className="w-5 h-5 text-primary" />
                      </div>
                      <CardTitle className="text-xl">{feature.title}</CardTitle>
                    </div>
                  </CardHeader>
                  <CardContent>
                    <CardDescription className="text-base">
                      {feature.description}
                    </CardDescription>
                  </CardContent>
                </Card>
              </motion.div>
            ))}
          </div>
        </div>
      </section>

      {/* Use Cases Section */}
      <section className="py-20 bg-muted/30">
        <div className="container mx-auto px-4 sm:px-6 lg:px-8">
          <div className="text-center mb-16">
            <h2 className="text-3xl lg:text-4xl font-bold mb-4">
              Real-World Applications
            </h2>
            <p className="text-xl text-muted-foreground max-w-2xl mx-auto">
              See how zkIPFS-Proof is being used across different industries
            </p>
          </div>

          <div className="grid md:grid-cols-2 lg:grid-cols-4 gap-8">
            {useCases.map((useCase, index) => (
              <motion.div
                key={useCase.title}
                initial={{ opacity: 0, y: 20 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ duration: 0.6, delay: index * 0.1 }}
              >
                <Card className="text-center h-full hover:shadow-lg transition-shadow">
                  <CardHeader>
                    <div className="text-4xl mb-4">{useCase.icon}</div>
                    <CardTitle className="text-xl">{useCase.title}</CardTitle>
                  </CardHeader>
                  <CardContent>
                    <CardDescription className="text-base">
                      {useCase.description}
                    </CardDescription>
                  </CardContent>
                </Card>
              </motion.div>
            ))}
          </div>
        </div>
      </section>

      {/* CTA Section */}
      <section className="py-20">
        <div className="container mx-auto px-4 sm:px-6 lg:px-8">
          <div className="bg-gradient-to-r from-primary/10 via-primary/5 to-primary/10 rounded-2xl p-8 lg:p-12 text-center">
            <h2 className="text-3xl lg:text-4xl font-bold mb-4">
              Ready to Get Started?
            </h2>
            <p className="text-xl text-muted-foreground mb-8 max-w-2xl mx-auto">
              Join thousands of users who trust zkIPFS-Proof for their file verification needs
            </p>
            
            <div className="flex flex-col sm:flex-row gap-4 justify-center items-center mb-8">
              <Button asChild size="lg" className="text-lg px-8 py-6">
                <Link to="/generate" className="flex items-center space-x-2">
                  <Upload className="w-5 h-5" />
                  <span>Start Generating Proofs</span>
                  <ArrowRight className="w-4 h-4" />
                </Link>
              </Button>
              
              <Button variant="outline" size="lg" asChild className="text-lg px-8 py-6">
                <a 
                  href="https://github.com/sowadmim/zkipfs-proof" 
                  target="_blank" 
                  rel="noopener noreferrer"
                  className="flex items-center space-x-2"
                >
                  <Github className="w-5 h-5" />
                  <span>View on GitHub</span>
                  <ExternalLink className="w-4 h-4" />
                </a>
              </Button>
            </div>

            <div className="flex items-center justify-center space-x-8 text-sm text-muted-foreground">
              <div className="flex items-center space-x-2">
                <GitFork className="w-4 h-4" />
                <span>Fork on GitHub</span>
              </div>
              <div className="flex items-center space-x-2">
                <Star className="w-4 h-4" />
                <span>Star the Project</span>
              </div>
            </div>
          </div>
        </div>
      </section>
    </div>
  )
}

