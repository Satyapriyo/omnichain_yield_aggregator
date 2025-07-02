'use client'

import dynamic from 'next/dynamic'
import { YieldAggregatorDashboard } from '@/components/YieldAggregatorDashboard'

const WalletMultiButton = dynamic(
  async () => (await import('@solana/wallet-adapter-react-ui')).WalletMultiButton,
  { ssr: false }
)

export default function Home() {
  return (
    <main className="min-h-screen bg-gradient-to-br from-blue-50 to-purple-50 dark:from-gray-900 dark:to-blue-900">
      <div className="container mx-auto px-4 py-8">
        <header className="flex justify-between items-center mb-8">
          <div>
            <h1 className="text-4xl font-bold text-gray-900 dark:text-white mb-2">
              LayerZero Yield Aggregator
            </h1>
            <p className="text-gray-600 dark:text-gray-300">
              Cross-chain yield farming and portfolio management
            </p>
          </div>
          <div className="flex items-center space-x-4">
            <WalletMultiButton />
          </div>
        </header>
        <YieldAggregatorDashboard />
      </div>
    </main>
  )
}
