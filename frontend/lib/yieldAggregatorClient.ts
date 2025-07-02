import { Connection, PublicKey } from '@solana/web3.js'

// Types for the yield aggregator
export interface YieldStrategy {
  publicKey: PublicKey
  name: string
  apy: number
  tvl: number
  riskLevel: 'Low' | 'Medium' | 'High'
  isActive: boolean
}

export interface UserPosition {
  strategy: PublicKey
  amount: number
  value: number
  pendingRewards: number
}

export interface ProtocolData {
  name: string
  tvl: number
  apy: number
  strategies: YieldStrategy[]
}

export class YieldAggregatorClient {
  private connection: Connection
  private walletPublicKey: PublicKey | null = null

  constructor(connection: Connection) {
    this.connection = connection
  }

  setWallet(publicKey: PublicKey | null) {
    this.walletPublicKey = publicKey
  }

  // Mock data for testing - replace with actual program calls
  async getStrategies(): Promise<YieldStrategy[]> {
    // Simulate network delay
    await new Promise(resolve => setTimeout(resolve, 1000))
    
    return [
      {
        publicKey: new PublicKey('11111111111111111111111111111112'),
        name: 'Solana Native Staking',
        apy: 7.2,
        tvl: 1250000,
        riskLevel: 'Low',
        isActive: true
      },
      {
        publicKey: new PublicKey('11111111111111111111111111111113'),
        name: 'Marinade Liquid Staking',
        apy: 8.5,
        tvl: 890000,
        riskLevel: 'Medium',
        isActive: true
      },
      {
        publicKey: new PublicKey('11111111111111111111111111111114'),
        name: 'Raydium LP Farming',
        apy: 15.3,
        tvl: 450000,
        riskLevel: 'High',
        isActive: true
      },
      {
        publicKey: new PublicKey('11111111111111111111111111111115'),
        name: 'Orca Whirlpools',
        apy: 12.8,
        tvl: 320000,
        riskLevel: 'Medium',
        isActive: false
      }
    ]
  }

  async getUserPositions(): Promise<UserPosition[]> {
    if (!this.walletPublicKey) return []
    
    // Simulate network delay
    await new Promise(resolve => setTimeout(resolve, 800))
    
    return [
      {
        strategy: new PublicKey('11111111111111111111111111111112'),
        amount: 100,
        value: 102.5,
        pendingRewards: 2.1
      },
      {
        strategy: new PublicKey('11111111111111111111111111111113'),
        amount: 50,
        value: 53.2,
        pendingRewards: 1.8
      }
    ]
  }

  async getProtocolData(): Promise<ProtocolData[]> {
    // Simulate network delay
    await new Promise(resolve => setTimeout(resolve, 1200))
    
    const strategies = await this.getStrategies()
    
    return [
      {
        name: 'Solana Ecosystem',
        tvl: 2560000,
        apy: 8.9,
        strategies: strategies.filter(s => s.name.includes('Solana') || s.name.includes('Marinade'))
      },
      {
        name: 'DeFi Protocols',
        tvl: 770000,
        apy: 14.1,
        strategies: strategies.filter(s => s.name.includes('Raydium') || s.name.includes('Orca'))
      }
    ]
  }

  async deposit(strategyPubkey: PublicKey, amount: number, token: 'SOL' | 'USDC' = 'SOL'): Promise<string> {
    if (!this.walletPublicKey) {
      throw new Error('Wallet not connected')
    }

    console.log(`Depositing ${amount} ${token} to strategy ${strategyPubkey.toString()}`)

    // Simulate transaction
    await new Promise(resolve => setTimeout(resolve, 2000))
    
    // In a real implementation, you would:
    // 1. Handle SOL vs USDC differently:
    //    - For SOL: Transfer native SOL directly
    //    - For USDC: Transfer SPL token (USDC mint)
    // 2. Create the appropriate deposit instruction based on token type
    // 3. Build and send the transaction
    // 4. Return the transaction signature
    
    return `mock_${token.toLowerCase()}_deposit_signature_` + Date.now()
  }

  async withdraw(strategyPubkey: PublicKey, amount: number): Promise<string> {
    if (!this.walletPublicKey) {
      throw new Error('Wallet not connected')
    }

    // Simulate transaction
    await new Promise(resolve => setTimeout(resolve, 2000))
    
    return 'mock_transaction_signature_' + Date.now()
  }

  async rebalance(fromStrategy: PublicKey, toStrategy: PublicKey, amount: number): Promise<string> {
    if (!this.walletPublicKey) {
      throw new Error('Wallet not connected')
    }

    // Simulate transaction
    await new Promise(resolve => setTimeout(resolve, 3000))
    
    return 'mock_rebalance_signature_' + Date.now()
  }

  async claimRewards(strategyPubkey: PublicKey): Promise<string> {
    if (!this.walletPublicKey) {
      throw new Error('Wallet not connected')
    }

    // Simulate transaction
    await new Promise(resolve => setTimeout(resolve, 1500))
    
    return 'mock_claim_signature_' + Date.now()
  }

  async getPortfolioValue(): Promise<number> {
    const positions = await this.getUserPositions()
    return positions.reduce((total, pos) => total + pos.value, 0)
  }

  async getTotalRewards(): Promise<number> {
    const positions = await this.getUserPositions()
    return positions.reduce((total, pos) => total + pos.pendingRewards, 0)
  }
}

// Singleton instance
let clientInstance: YieldAggregatorClient | null = null

export function getYieldAggregatorClient(connection?: Connection): YieldAggregatorClient {
  if (!clientInstance) {
    if (!connection) {
      throw new Error('Connection required to create client')
    }
    clientInstance = new YieldAggregatorClient(connection)
  }
  return clientInstance
}
