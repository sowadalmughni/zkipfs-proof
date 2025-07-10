import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Shield, Users, Globe, Heart, Github, ExternalLink } from 'lucide-react'
import { Button } from '@/components/ui/button'

export default function AboutPage() {
  return (
    <div className="container mx-auto px-4 sm:px-6 lg:px-8 py-8">
      <div className="max-w-4xl mx-auto">
        <div className="text-center mb-12">
          <h1 className="text-3xl lg:text-4xl font-bold mb-4">
            About zkIPFS-Proof
          </h1>
          <p className="text-xl text-muted-foreground max-w-2xl mx-auto">
            An open-source project dedicated to making zero-knowledge file verification 
            accessible to everyone, from journalists to researchers to everyday users.
          </p>
        </div>

        <div className="space-y-8">
          {/* Mission */}
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center space-x-2">
                <Shield className="w-5 h-5" />
                <span>Our Mission</span>
              </CardTitle>
            </CardHeader>
            <CardContent>
              <p className="text-muted-foreground">
                We believe in a world where anyone can prove the authenticity of their data 
                without compromising privacy. zkIPFS-Proof combines cutting-edge zero-knowledge 
                cryptography with decentralized storage to create a trustless verification system 
                that protects both content and privacy.
              </p>
            </CardContent>
          </Card>

          {/* Technology */}
          <Card>
            <CardHeader>
              <CardTitle>Technology Stack</CardTitle>
              <CardDescription>
                Built with modern, battle-tested technologies
              </CardDescription>
            </CardHeader>
            <CardContent>
              <div className="grid md:grid-cols-2 gap-4">
                <div>
                  <h4 className="font-semibold mb-2">Zero-Knowledge Proofs</h4>
                  <p className="text-sm text-muted-foreground">
                    Powered by Risc0 ZK-VM for efficient and secure proof generation
                  </p>
                </div>
                <div>
                  <h4 className="font-semibold mb-2">Decentralized Storage</h4>
                  <p className="text-sm text-muted-foreground">
                    IPFS content addressing for permanent, tamper-proof file records
                  </p>
                </div>
                <div>
                  <h4 className="font-semibold mb-2">Blockchain Integration</h4>
                  <p className="text-sm text-muted-foreground">
                    Smart contracts for on-chain verification across multiple networks
                  </p>
                </div>
                <div>
                  <h4 className="font-semibold mb-2">Developer Tools</h4>
                  <p className="text-sm text-muted-foreground">
                    CLI, web interface, and APIs for seamless integration
                  </p>
                </div>
              </div>
            </CardContent>
          </Card>

          {/* Open Source */}
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center space-x-2">
                <Github className="w-5 h-5" />
                <span>Open Source</span>
              </CardTitle>
              <CardDescription>
                Transparency and community-driven development
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <p className="text-muted-foreground">
                zkIPFS-Proof is completely open source under the MIT license. 
                We believe that cryptographic tools should be transparent, auditable, 
                and accessible to everyone.
              </p>
              <Button asChild>
                <a
                  href="https://github.com/sowadmim/zkipfs-proof"
                  target="_blank"
                  rel="noopener noreferrer"
                  className="flex items-center space-x-2"
                >
                  <Github className="w-4 h-4" />
                  <span>View on GitHub</span>
                  <ExternalLink className="w-3 h-3" />
                </a>
              </Button>
            </CardContent>
          </Card>

          {/* Team */}
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center space-x-2">
                <Users className="w-5 h-5" />
                <span>Team</span>
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div className="flex items-center space-x-4">
                <div className="w-16 h-16 bg-gradient-to-br from-primary to-primary/70 rounded-full flex items-center justify-center">
                  <span className="text-xl font-bold text-primary-foreground">SA</span>
                </div>
                <div>
                  <h4 className="font-semibold">Sowad Al-Mughni</h4>
                  <p className="text-sm text-muted-foreground">Creator & Lead Developer</p>
                  <p className="text-sm text-muted-foreground">
                    Passionate about cryptography, privacy, and open-source software
                  </p>
                </div>
              </div>
            </CardContent>
          </Card>

          {/* Community */}
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center space-x-2">
                <Globe className="w-5 h-5" />
                <span>Community</span>
              </CardTitle>
              <CardDescription>
                Join our growing community of privacy advocates and developers
              </CardDescription>
            </CardHeader>
            <CardContent>
              <p className="text-muted-foreground mb-4">
                We're building more than just software - we're building a community 
                of people who believe in privacy, transparency, and the power of 
                cryptographic verification.
              </p>
              <div className="flex flex-wrap gap-2">
                <Button variant="outline" size="sm" asChild>
                  <a href="https://discord.gg/zkipfs-proof" target="_blank" rel="noopener noreferrer">
                    Discord
                  </a>
                </Button>
                <Button variant="outline" size="sm" asChild>
                  <a href="https://twitter.com/zkipfs_proof" target="_blank" rel="noopener noreferrer">
                    Twitter
                  </a>
                </Button>
                <Button variant="outline" size="sm" asChild>
                  <a href="https://github.com/sowadmim/zkipfs-proof/discussions" target="_blank" rel="noopener noreferrer">
                    Discussions
                  </a>
                </Button>
              </div>
            </CardContent>
          </Card>

          {/* Support */}
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center space-x-2">
                <Heart className="w-5 h-5 text-red-500" />
                <span>Support the Project</span>
              </CardTitle>
            </CardHeader>
            <CardContent>
              <p className="text-muted-foreground mb-4">
                zkIPFS-Proof is developed and maintained by volunteers. 
                Your support helps us continue building privacy-preserving tools for everyone.
              </p>
              <div className="flex flex-wrap gap-2">
                <Button variant="outline" size="sm">
                  ⭐ Star on GitHub
                </Button>
                <Button variant="outline" size="sm">
                  🐛 Report Issues
                </Button>
                <Button variant="outline" size="sm">
                  💡 Contribute Code
                </Button>
                <Button variant="outline" size="sm">
                  📖 Improve Docs
                </Button>
              </div>
            </CardContent>
          </Card>
        </div>
      </div>
    </div>
  )
}

