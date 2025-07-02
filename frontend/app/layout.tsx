import './globals.css'
import type { Metadata } from 'next'
import { WalletContextProvider } from '@/components/WalletContextProvider'

export const metadata: Metadata = {
  title: 'Yield Aggregator Test Frontend',
  description: 'Frontend for testing LayerZero Yield Aggregator',
}

export default function RootLayout({
  children,
}: {
  children: React.ReactNode
}) {
  return (
    <html lang="en">
      <body className="font-sans">
        <WalletContextProvider>
          {children}
        </WalletContextProvider>
      </body>
    </html>
  )
}
