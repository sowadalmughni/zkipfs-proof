import { useState, useEffect } from 'react'
import { Toaster } from 'react-hot-toast'

// Components
import Header from './components/Header'
import Hero from './components/Hero'
import FileUpload from './components/FileUpload'
import ProofGeneration from './components/ProofGeneration'
import ProofVerification from './components/ProofVerification'
import Footer from './components/Footer'
import LoadingScreen from './components/LoadingScreen'

// Styles
import './App.css'

function App() {
  const [loading, setLoading] = useState(true)
  const [currentPage, setCurrentPage] = useState('home')

  useEffect(() => {
    // Simulate app initialization
    const timer = setTimeout(() => {
      setLoading(false)
    }, 1500)

    return () => clearTimeout(timer)
  }, [])

  if (loading) {
    return <LoadingScreen />
  }

  const renderPage = () => {
    switch (currentPage) {
      case 'generate':
        return (
          <div className="min-h-screen bg-gray-50 dark:bg-gray-900 py-12">
            <div className="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8">
              <div className="text-center mb-8">
                <h1 className="text-3xl font-bold text-gray-900 dark:text-white mb-4">
                  Generate Proof
                </h1>
                <p className="text-lg text-gray-600 dark:text-gray-400">
                  Create a zero-knowledge proof for your file content
                </p>
              </div>
              <div className="bg-white dark:bg-gray-800 rounded-lg shadow-lg p-8">
                <div className="space-y-8">
                  <FileUpload onFileSelect={(file) => console.log('File selected:', file)} />
                  <ProofGeneration />
                </div>
              </div>
            </div>
          </div>
        )
      case 'verify':
        return (
          <div className="min-h-screen bg-gray-50 dark:bg-gray-900 py-12">
            <div className="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8">
              <div className="text-center mb-8">
                <h1 className="text-3xl font-bold text-gray-900 dark:text-white mb-4">
                  Verify Proof
                </h1>
                <p className="text-lg text-gray-600 dark:text-gray-400">
                  Verify the authenticity of a zero-knowledge proof
                </p>
              </div>
              <div className="bg-white dark:bg-gray-800 rounded-lg shadow-lg p-8">
                <ProofVerification />
              </div>
            </div>
          </div>
        )
      default:
        return (
          <div>
            <Hero />
            <div className="py-16 bg-white dark:bg-gray-800">
              <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
                <div className="text-center">
                  <h2 className="text-3xl font-bold text-gray-900 dark:text-white mb-8">
                    Get Started
                  </h2>
                  <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
                    <div className="bg-gray-50 dark:bg-gray-700 rounded-lg p-8">
                      <h3 className="text-xl font-semibold text-gray-900 dark:text-white mb-4">
                        Generate Proof
                      </h3>
                      <p className="text-gray-600 dark:text-gray-400 mb-6">
                        Create a cryptographic proof that specific content exists in your file
                      </p>
                      <button
                        onClick={() => setCurrentPage('generate')}
                        className="w-full px-6 py-3 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
                      >
                        Start Generating
                      </button>
                    </div>
                    <div className="bg-gray-50 dark:bg-gray-700 rounded-lg p-8">
                      <h3 className="text-xl font-semibold text-gray-900 dark:text-white mb-4">
                        Verify Proof
                      </h3>
                      <p className="text-gray-600 dark:text-gray-400 mb-6">
                        Verify the authenticity of an existing zero-knowledge proof
                      </p>
                      <button
                        onClick={() => setCurrentPage('verify')}
                        className="w-full px-6 py-3 bg-green-600 text-white rounded-lg hover:bg-green-700 transition-colors"
                      >
                        Start Verifying
                      </button>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        )
    }
  }

  return (
    <div className="min-h-screen bg-white dark:bg-gray-900">
      <Header currentPage={currentPage} setCurrentPage={setCurrentPage} />
      <main>
        {renderPage()}
      </main>
      <Footer />
      <Toaster position="top-right" />
    </div>
  )
}

export default App

