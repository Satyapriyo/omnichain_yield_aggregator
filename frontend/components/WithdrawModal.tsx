import React, { useState } from "react";
import { YieldAggregatorClient } from "@/lib/yieldAggregatorClient";
import { UserPosition } from "./YieldAggregatorDashboard";

interface WithdrawModalProps {
  client: YieldAggregatorClient;
  isOpen: boolean;
  onClose: () => void;
  onSuccess: () => void;
  userPosition: UserPosition | null;
}

export const WithdrawModal: React.FC<WithdrawModalProps> = ({
  client,
  isOpen,
  onClose,
  onSuccess,
  userPosition,
}) => {
  const [amount, setAmount] = useState("");
  const [withdrawToChain, setWithdrawToChain] = useState("40168"); // Default to Solana
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState("");

  const maxWithdrawable = userPosition
    ? userPosition.totalDeposits + userPosition.totalYieldEarned
    : 0;

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError("");
    setIsLoading(true);

    try {
      const amountNum = parseFloat(amount);

      if (isNaN(amountNum) || amountNum <= 0) {
        throw new Error("Please enter a valid amount");
      }

      if (amountNum > maxWithdrawable) {
        throw new Error("Amount exceeds available balance");
      }

      // For demo purposes, we'll use the first available strategy
      // In a real implementation, you'd specify which strategy to withdraw from
      const strategies = await client.getStrategies();
      const strategy = strategies[0]; // Just use first strategy for demo

      if (!strategy) {
        throw new Error("No strategies available");
      }

      await client.withdraw(strategy.publicKey, amountNum);

      onSuccess();
      onClose();
    } catch (err) {
      setError(err instanceof Error ? err.message : "An error occurred");
    } finally {
      setIsLoading(false);
    }
  };

  const handleMaxClick = () => {
    setAmount(maxWithdrawable.toString());
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center p-4 z-50">
      <div className="bg-white dark:bg-gray-800 rounded-lg shadow-xl max-w-md w-full max-h-[90vh] overflow-y-auto">
        <div className="p-6">
          {/* Header */}
          <div className="flex justify-between items-center mb-6">
            <h2 className="text-2xl font-bold text-gray-900 dark:text-white">
              Withdraw Funds
            </h2>
            <button
              onClick={onClose}
              className="text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200"
            >
              ✕
            </button>
          </div>

          {/* Balance Info */}
          {userPosition && (
            <div className="bg-gray-50 dark:bg-gray-700 rounded-lg p-4 mb-6">
              <h3 className="font-semibold text-gray-900 dark:text-white mb-3">
                Available Balance
              </h3>
              <div className="grid grid-cols-2 gap-4 text-sm">
                <div>
                  <p className="text-gray-600 dark:text-gray-300">Principal</p>
                  <p className="font-semibold text-gray-900 dark:text-white">
                    ${userPosition.totalDeposits.toLocaleString()}
                  </p>
                </div>
                <div>
                  <p className="text-gray-600 dark:text-gray-300">
                    Yield Earned
                  </p>
                  <p className="font-semibold text-green-600 dark:text-green-400">
                    ${userPosition.totalYieldEarned.toLocaleString()}
                  </p>
                </div>
              </div>
              <div className="mt-3 pt-3 border-t border-gray-200 dark:border-gray-600">
                <div className="flex justify-between">
                  <span className="font-semibold text-gray-900 dark:text-white">
                    Total Available:
                  </span>
                  <span className="font-bold text-blue-600 dark:text-blue-400">
                    ${maxWithdrawable.toLocaleString()}
                  </span>
                </div>
              </div>
            </div>
          )}

          {/* Form */}
          <form onSubmit={handleSubmit} className="space-y-4">
            <div>
              <div className="flex justify-between items-center mb-2">
                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300">
                  Withdrawal Amount (USDC)
                </label>
                <button
                  type="button"
                  onClick={handleMaxClick}
                  className="text-sm text-blue-600 dark:text-blue-400 hover:underline"
                >
                  Max
                </button>
              </div>
              <input
                type="number"
                value={amount}
                onChange={(e) => setAmount(e.target.value)}
                placeholder="0.00"
                min="0"
                max={maxWithdrawable}
                step="0.01"
                className="input-field"
                required
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                Withdraw To Chain
              </label>
              <select
                value={withdrawToChain}
                onChange={(e) => setWithdrawToChain(e.target.value)}
                className="input-field"
                required
              >
                <option value="40168">Solana</option>
                <option value="1">Ethereum</option>
                <option value="137">Polygon</option>
                <option value="56">Binance Smart Chain</option>
                <option value="43114">Avalanche</option>
              </select>
              <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
                Cross-chain withdrawals may take longer to process
              </p>
            </div>

            {error && (
              <div className="bg-red-50 dark:bg-red-900 border border-red-200 dark:border-red-700 rounded-md p-3">
                <p className="text-sm text-red-600 dark:text-red-400">
                  {error}
                </p>
              </div>
            )}

            {/* Action Buttons */}
            <div className="flex gap-3 pt-4">
              <button
                type="button"
                onClick={onClose}
                className="flex-1 btn-secondary"
                disabled={isLoading}
              >
                Cancel
              </button>
              <button
                type="submit"
                className="flex-1 btn-primary"
                disabled={isLoading || !amount || maxWithdrawable === 0}
              >
                {isLoading ? "Processing..." : "Withdraw"}
              </button>
            </div>
          </form>

          {/* Info */}
          <div className="mt-6 p-4 bg-yellow-50 dark:bg-yellow-900 rounded-lg">
            <p className="text-sm text-yellow-800 dark:text-yellow-200">
              ⚠️ <strong>Important:</strong> Cross-chain withdrawals may take
              5-15 minutes to complete. Local Solana withdrawals are processed
              immediately.
            </p>
          </div>
        </div>
      </div>
    </div>
  );
};
