import React from 'react'
import { UserPosition } from './YieldAggregatorDashboard'

interface UserPositionCardProps {
  position: UserPosition
  onWithdraw: () => void
  onRebalance: () => void
  formatCurrency: (amount: number) => string
}

export const UserPositionCard: React.FC<UserPositionCardProps> = ({
  position,
  onWithdraw,
  onRebalance,
  formatCurrency,
}) => {
  const totalValue = position.totalDeposits + position.totalYieldEarned
  const yieldPercentage = position.totalDeposits > 0 
    ? (position.totalYieldEarned / position.totalDeposits) * 100 
    : 0

  return (
    <div className="card bg-gradient-to-r from-blue-50 to-purple-50 dark:from-blue-900 dark:to-purple-900 border-2 border-blue-200 dark:border-blue-700">
      <div className="flex justify-between items-start mb-6">
        <div>
          <h2 className="text-2xl font-bold text-gray-900 dark:text-white mb-2">
            Your Portfolio
          </h2>
          <p className="text-gray-600 dark:text-gray-300">
            {position.positionCount} active position{position.positionCount !== 1 ? 's' : ''}
          </p>
        </div>
        <div className="text-right">
          <p className="text-sm text-gray-600 dark:text-gray-300">Total Value</p>
          <p className="text-3xl font-bold text-blue-600 dark:text-blue-400">
            {formatCurrency(totalValue)}
          </p>
        </div>
      </div>

      {/* Stats Grid */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mb-6">
        <div className="bg-white dark:bg-gray-800 rounded-lg p-4 shadow-sm">
          <p className="text-sm text-gray-600 dark:text-gray-300 mb-1">Principal</p>
          <p className="text-xl font-semibold text-gray-900 dark:text-white">
            {formatCurrency(position.totalDeposits)}
          </p>
        </div>
        
        <div className="bg-white dark:bg-gray-800 rounded-lg p-4 shadow-sm">
          <p className="text-sm text-gray-600 dark:text-gray-300 mb-1">Yield Earned</p>
          <p className="text-xl font-semibold text-green-600 dark:text-green-400">
            {formatCurrency(position.totalYieldEarned)}
          </p>
          <p className="text-xs text-gray-500 dark:text-gray-400">
            +{yieldPercentage.toFixed(2)}%
          </p>
        </div>
        
        <div className="bg-white dark:bg-gray-800 rounded-lg p-4 shadow-sm">
          <p className="text-sm text-gray-600 dark:text-gray-300 mb-1">Last Activity</p>
          <p className="text-sm font-semibold text-gray-900 dark:text-white">
            {new Date(position.lastActivity).toLocaleDateString()}
          </p>
          <p className="text-xs text-gray-500 dark:text-gray-400">
            {new Date(position.lastActivity).toLocaleTimeString()}
          </p>
        </div>
      </div>

      {/* Action Buttons */}
      <div className="flex flex-col sm:flex-row gap-3">
        <button
          onClick={onWithdraw}
          disabled={totalValue === 0}
          className="flex-1 btn-primary disabled:bg-gray-400 disabled:cursor-not-allowed"
        >
          💰 Withdraw
        </button>
        <button
          onClick={onRebalance}
          disabled={position.positionCount < 2}
          className="flex-1 btn-secondary disabled:bg-gray-400 disabled:cursor-not-allowed"
        >
          ⚖️ Rebalance
        </button>
      </div>

      {position.positionCount < 2 && (
        <p className="text-xs text-gray-500 dark:text-gray-400 mt-2 text-center">
          You need at least 2 positions to rebalance
        </p>
      )}
    </div>
  )
}
